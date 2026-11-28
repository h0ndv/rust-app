# Architecture

## 1. What the app is

A Windows desktop client that reads three separate academic systems for a Santo Tomás
student and presents them in one interface:

- **Moodle** for courses, assignments, resources, announcements and grade totals.
- **Ellucian Experience** for the class list and the weekly schedule.
- **Banner 9** for official grades and GPA.

The app is read-only. It never submits, uploads, edits or deletes anything in those
platforms. All data lives in a local SQLite file; there is no central server, no
telemetry and no cloud account.

The app does not have a name yet. `rust-app` is the placeholder used by the repository, the bundle identifier and the binary.

## 2. Goals and non-goals

**Goals**

- One Microsoft login covers all three sources.
- "What is due" answered in seconds, with overdue items first.
- Courses, tasks, resources, schedule and grades without opening a browser.
- Background sync that never blocks the interface.
- Works offline with the last snapshot.
- Downloads Moodle attachments and opens them in the in-app WebView viewer.

**Non-goals**

- Attendance. Not exposed by any verified source.
- Submitting, uploading or editing anything.
- Recomputing averages or mixing Moodle and Banner grades.
- Storing passwords.
- Central server, accounts, ads or telemetry.
- Mobile apps, macOS or Linux in the first version.

## 3. Layers

```text
Dioxus UI (src/, WebAssembly)
   │  Transport::invoke(command, payload) / listen("sync://…")
   ▼
contracts            Serialize/Deserialize DTOs shared by both sides
   ▼
commands/            Tauri boundary: DTOs in, DTOs out, `{kind, message}` errors
   │
   ├── sync/         orchestrator, novelty rules, archiving, download queue
   ├── db/           SQLite pool, repositories, schema, course linking
   ├── auth/         login windows, cookie bridge, source health
   ├── http/         rate-limited reqwest client
   └── moodle/  ellucian/  banner/     one adapter per source
```

The interface never talks to Moodle, Ellucian or Banner. It only calls commands, and
Rust owns every decision. Both sides share one set of types in `crates/contracts`, so a
command rename breaks the build instead of the running app.

## 4. Module map

| Path                      | Responsibility                                                             |
| ------------------------- | -------------------------------------------------------------------------- |
| `crates/contracts/`       | DTOs, commands payloads and `AppError` wire types shared by both sides     |
| `src/`                    | Dioxus app: router, context, transport, theme, formatting, i18n            |
| `src/components/`         | Shell, sidebar, headers, toasts and the component primitives               |
| `src/features/`           | One module per screen; renders from a DTO, holds no global state           |
| `tests/`                  | Dioxus SSR tests for the interface                                         |
| `locales/`                | `en.yml` and `es.yml`, loaded at compile time by `rust-i18n`               |
| `assets/`                 | Tailwind sources, generated `app.css`, self-hosted Inter and Geist Mono    |
| `src-tauri/src/lib.rs`    | Builder, setup, DB bootstrap, command registry                             |
| `src-tauri/src/state.rs`  | `AppState { pool, repo, sync, downloads_fallback }`                        |
| `src-tauri/src/error.rs`  | `AppError` with `kind` and a technical message                             |
| `src-tauri/src/events.rs` | Injected event emitter type shared by auth, sync and downloads             |
| `src-tauri/src/http/`     | `HttpClient` (timeout, 4 in flight), `RateLimiter` (6/s, 600/run)          |
| `src-tauri/src/auth/`     | `Source`, `SessionState`, `SessionManager`, login windows, cookies, health |
| `src-tauri/src/moodle/`   | AJAX client, HTML/JSON parsers, models                                     |
| `src-tauri/src/ellucian/` | Query descriptors, parsers, GraphQL-style client                           |
| `src-tauri/src/banner/`   | Grades client, `eval_json` fallback, parsers                               |
| `src-tauri/src/db/`       | `connect`, `migrate`, models, `Repo`, `linking`                            |
| `src-tauri/src/sync/`     | `SyncOrchestrator`, `diff`, `downloads`, source traits                     |
| `src-tauri/src/commands/` | Commands and DTOs consumed by the UI                                       |
| `src-tauri/migrations/`   | Versioned SQLite schema                                                    |
| `src-tauri/tests/`        | Integration tests and real fixtures                                        |

## 5. Runtime and threading rules

These are not style preferences. Each one exists because the alternative hung the app.

- **Commands that open windows are `async`.** Creating a `WebviewWindow` from a sync
  command blocks the main thread inside `build()` and leaves the process
  `Responding=False`.
- **`cookies_for_url` always runs inside `spawn_blocking`**, never on the main thread
  and never from a window event handler.
- **The library does not import Tauri's `Emitter`.** Events are injected as
  `EventEmitter = Arc<dyn Fn(&str, Value)>` from `lib.rs`. `AppHandle::emit`
  monomorphises Tauri into the crate and pulls `WebView2Loader.dll` into test binaries,
  which then fail to start with `STATUS_ENTRYPOINT_NOT_FOUND`.
- **One runtime.** Everything runs on `tauri::async_runtime`; background loops use
  `tauri::async_runtime::spawn`, not `tokio::spawn` from `setup` (that panics with
  "there is no reactor running").
- **Body reads have their own timeout.** `reqwest`'s `timeout` covers headers, not the
  body. `http::read_text` and `http::read_bytes` wrap the read.

## 6. Authentication

The WebView2 profile on disk is the single source of truth for cookies. They are never
copied to SQLite and never logged.

### Flow

1. The student presses connect. `SessionManager::connect_all` walks
   **Ellucian → Moodle → Banner** because Ellucian starts the SAML chain and the
   Microsoft session it leaves behind is reused by the other two.
2. Each source opens the shared `auth` window, navigates, waits for its final page and
   reads the cookies with `cookies_for_url`.
3. The session is confirmed with a real request (Moodle `/my/`, a minimal GraphQL
   query, Banner `/term`) before the source is marked healthy.
4. The window hides and closes. Cookies stay in the profile.

### State machine

`Disconnected → Connecting(source) → Connected | Partial`, plus `Expired(source)` when
a source that used to work starts rejecting requests. Any failure makes the session
`Partial`; the other sources keep working.

### Session lifetime

`MoodleSession`, `ustc-exp.sid` and `JSESSIONID` are **session cookies without an
expiry date**. WebView2 drops them when the process ends, so every app restart starts
without a session. The Microsoft cookies do survive (`ESTSAUTHPERSISTENT`,
`AADSSOTILES`), which makes reconnection a click with no password and no MFA.

The app must treat "no session" as a normal startup state, not as an error. Every session
transition emits `session://changed` through the injected emitter. The app opens with the
cached snapshot and a reconnect banner; onboarding is only the first run with no local data.

### Health

Each source is checked every 15 minutes and when the main window regains focus, with a
60-second debounce. A health check is a cheap authenticated request; an expired session
returns `false`, not an error.

## 7. Sync pipeline

`SyncOrchestrator::run(trigger)` executes nine steps. Every source block has its own
`Result`, so a failure is recorded and the run continues.

1. Reset the request budget, close stale `running` runs as `failed`, open a `sync_runs` row.
2. Health of the three sources → `source_status`.
3. Moodle: enrolled courses and the calendar (used as a due-date fallback for quizzes).
4. Moodle: one page per section, then activities, assignments, attachments,
   announcements and the course total.
5. Ellucian: classes and the schedule for those classes.
6. Banner: academic periods, levels and grades for the current period.
7. Link catalogs (`link_courses`, `course_rows`, `course_groups`) and upsert them.
8. Persist Banner grades and the Ellucian schedule against the linked courses.
9. Mark synced courses, archive stale items and close the run as `ok`, `partial` or
   `failed`.

### Rules that matter

- **Baseline per course.** If a course has no activities yet, everything found in that
  run is stored as already acknowledged. Later runs leave `acknowledged_at` NULL for
  new items and write an event in `events`.
- **Nothing is deleted.** An item that stops appearing is archived after 7 days unseen,
  and only if its course synced successfully in the last hour.
- **Budget.** 6 requests per second, 600 per run, 4 in flight. Exhausting the budget
  ends the run as `partial` without losing data.
- **Cancellation.** A `CancellationToken` is checked between courses. Cancelling marks
  the run `partial` with the error `cancelada`.
- **Interrupted runs.** A run that dies with the process leaves `running`; the next run
  closes it as `failed`.

### Measured cost

8 courses, 238 requests, about 6 minutes on the real account. The design target is 60
seconds. The known improvement is fetching section pages four at a time and reusing
the course page that is currently requested twice.

## 8. Data model

SQLite in WAL mode at `%APPDATA%\com.santotomas.rustapp\panel.db`, with
`synchronous=NORMAL` and the WAL capped at 8 MB. `PRAGMA optimize` and a truncating
checkpoint run after every sync.

| Table             | Holds                                                         |
| ----------------- | ------------------------------------------------------------- |
| `course_groups`   | One row per course code (`IGR-00167`)                         |
| `courses`         | One row per real section; holds both Moodle id and Banner CRN |
| `sections`        | Units inside a course page                                    |
| `activities`      | Assignments, quizzes, forums; status, due dates, hash         |
| `files`           | Attachments and resources with `local_path` and hash          |
| `grade_items`     | Banner and Moodle grade rows, kept separate by `source`       |
| `grade_totals`    | One total per course, term and source                         |
| `schedule_events` | Weekly blocks with room, building and instructor              |
| `announcements`   | Forum posts                                                   |
| `events`          | Local novelty log used by Novedades                           |
| `sync_runs`       | Result, duration, request count and per-source errors         |
| `source_status`   | Last health check per source                                  |
| `settings`        | Key/value preferences                                         |
| `search_index`    | FTS5 over activities, files and announcements                 |

**No foreign keys on purpose.** The sync writes each source before the union, so a
child row can legitimately arrive before its course. Integrity comes from the upsert
order, and nothing is deleted on a source failure.

`last_seen_at` exists only to support the 7-day archiving rule; `first_seen_at` and
`acknowledged_at` drive the "new" badge.

## 9. Course linking

A course may exist in Moodle, in Banner, or in both. The app never hides one.

1. **CRN**: the prefix of the Moodle shortname (`30061.202620`) against Banner's
   `courseReferenceNumber` (`30061`). This survives course renames.
2. **Code and section**: `(IGR-00167-00G)` inside the Moodle full name against
   Banner's `subjectCode`, `courseNumber` and `section`.

The term must match when both sides carry one. Grouping is by code, so lecture and lab
appear under the same card with two sections.

## 10. Interface

The interface is Rust: Dioxus 0.7 with client-side rendering, compiled to WebAssembly
and embedded in the Tauri window. There is no Node toolchain; the only JavaScript is the
wasm-bindgen glue that `dx` generates.

- **Entry point.** `src/lib.rs` declares the modules and, for `wasm32`, launches `App`.
  `src/main.rs` starts the same app for `dx serve`. `src/app.rs` builds the context,
  runs four startup probes (`get_settings`, `has_local_data`, `session_state`,
  `recovery_notice`) and decides between onboarding and the shell.
- **Routing.** `src/router.rs` derives `Routable`; `AppShell` is the layout and every
  screen is a feature module under `src/features/`. Paths mirror the Spanish UI
  (`/ramos`, `/tareas`, `/recursos`, `/notas`, `/horario`, `/novedades`, `/buscar`,
  `/ajustes`) and the course route carries an optional focus.
- **State.** One `AppContext` in Dioxus context holds the `Api`, the session, the sync
  view, the language, the toast queue and a `data_version` counter. Screens read it with
  `use_app()`. A finished sync bumps `data_version`, and every screen refetches on the
  change.
- **Transport.** `src/transport.rs` defines `async_trait Transport`. The `wasm32`
  implementation calls `__TAURI__.core.invoke` (the window is created with
  `withGlobalTauri`), serializes args with `serde-wasm-bindgen`'s JSON-compatible mode
  because the default emits ES maps Tauri cannot read, and decodes rejections into
  `UiError`. Native builds use `UnsupportedTransport`; tests inject a fake.
- **Events.** `src/events.rs` subscribes to three Tauri events (`session://changed`,
  `sync://progress`, `sync://finished`). Each subscription returns a `Listener` that
  keeps the JS closure alive and calls the unlisten function before releasing it, so a
  pending `listen` can never hand Tauri a freed callback.
- **Copy and locale.** `rust-i18n` compiles `locales/en.yml` and `locales/es.yml`.
  `settings.language` drives the catalog, the document language and date formatting.
  Rust sends stable English codes and serialized `AppError` objects, so rewording a
  screen never needs a Rust change.
- **Theme.** `src/theme.rs` applies `settings.theme` at boot and listens to
  `prefers-color-scheme` for the `system` value; `dark` is a class on the root.
- **Assets.** `src/app.rs` registers `asset!("/assets/app.css")` and the fonts folder;
  `assets/app.css` is generated by the standalone Tailwind binary from
  `assets/tailwind.css` and the class strings in `src/**/*.rs`.
- **CSP.** `src-tauri/tauri.conf.json` keeps `'self'`, `data:` images,
  `wasm-unsafe-eval`, inline styles and the IPC endpoints, and blocks every other
  origin. The browser window shares the authenticated WebView2 profile, so the policy
  is also the security boundary for source pages opened in it.

## 11. Commands and DTOs

`commands/` exposes 30 commands. Reads return DTOs re-exported by `commands/dto.rs` from
`crates/contracts`; the UI never sees database models and the backend never sees
interface types.

| Group    | Commands                                                                                                                                              |
| -------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| Session  | `connect(source?)`, `session_state`, `refresh_session`                                                                                                |
| Sync     | `start_sync`, `cancel_sync`, `sync_status`                                                                                                            |
| Reads    | `get_dashboard`, `has_local_data`, `list_courses`, `get_course`, `list_tasks`, `list_resources`, `list_grades`, `get_schedule`, `list_news`, `search` |
| Settings | `get_settings`, `save_settings`, `set_language`, `set_window_theme`                                                                                   |
| Actions  | `acknowledge`, `open_in_app`, `download_file`, `export_data`, `clear_downloads`, `recovery_notice`, `clear_data`, `notification_permission`, `request_notification_permission`, `send_notification` |

Filters (`TaskFilter`, `ResourceFilter`) travel lowercase so the JSON stays
`"overdue"`, `"new"`. `no_data` is spelled out in the enum because the lowercase rename
would produce `nodata`.

Novedades reads unread rows straight from `activities`, `files` and `announcements`;
`acknowledge` updates the entity and the `events` novelty log in one call, and the
interface bumps `data_version` so the badge clears on every screen.

The interface language lives in `settings.language` and defaults to `"en"`. Adding a
language is a `locales/<code>.yml` file plus a `Language` variant in `src/i18n.rs`; the
contracts and the commands stay untouched.

The other settings are read where they are used: the scheduler reads
`settings.sync_interval_minutes` every minute, the download worker reads
`settings.downloads_dir` for each item, the interface applies `settings.theme` at boot
and on save, and notifications honor `settings.notifications`.

Events emitted by the backend: `session://changed`, `sync://progress`,
`sync://finished`, `download://finished`, `download://error`. The interface listens to
all five; `download_file` only queues the job in the download worker and returns, then
the UI waits for the matching `download://finished` or `download://error` before it
refreshes the row or shows the failure toast.

## 12. Decisions and tradeoffs

| Decision                                                                | Alternative                          | Why                                                                 | Cost if wrong                                                  |
| ----------------------------------------------------------------------- | ------------------------------------ | ------------------------------------------------------------------- | -------------------------------------------------------------- |
| One trait per source (`MoodleSource`, `EllucianSource`, `BannerSource`) | A single `Sources` trait             | A new source is additive; a test fakes only what it needs           | Three traits instead of one                                    |
| `SyncDeps` struct                                                       | Five constructor arguments           | New dependencies do not change signatures                           | One more struct                                                |
| Injected `EventEmitter`                                                 | `AppHandle` inside the orchestrator  | Keeps Tauri out of the library and test binaries                    | One closure in `lib.rs`                                        |
| `async-trait`                                                           | Hand-written `BoxFuture`             | `dyn` needs boxing anyway; the macro removes 13 signatures of noise | One proc-macro dependency                                      |
| No foreign keys                                                         | `REFERENCES` + `ON DELETE CASCADE`   | Sync writes sources before the union                                | Orphan rows that archiving already tolerates                   |
| `last_seen_at` column                                                   | Reuse `first_seen_at`                | 7-day archiving needs the last time an item was seen                | One migration                                                  |
| Resources are files, not pages                                          | Fetch `mod/resource/view.php`        | The page redirects to the PDF; reading it as text hung the sync     | Attachment metadata comes from the link instead                |
| Baseline per course                                                     | Global first run                     | A new course added mid-semester should not flood Novedades          | A newly added course never marks its first items as new        |
| Dioxus in Rust                                                          | React plus Vite                      | One language for the whole app, no Node toolchain, shared DTOs      | Larger wasm bundle and a smaller component ecosystem            |
| `Transport` trait                                                       | Call `invoke` from every screen      | Tests render with a fake and stay offline                           | One indirection per command                                    |
| `crates/contracts`                                                      | Duplicate DTOs in both crates        | A wire mismatch breaks the build instead of a screen                | One crate and one dependency edge                               |
| `rust-i18n` catalogs                                                    | Rust-localized messages              | UI copy and date formatting change with the interface               | A catalog per language                                         |
| CSP plus a three-host allowlist                                         | Trust every URL the adapters produce | The browser window shares the authenticated WebView2 profile        | A source link outside those hosts is blocked by design         |
| Deep links carry the course group and an optional focus                 | Resolve ids inside the route         | Search and Novedades already join the source tables                 | A hit without a course stays unlinked                          |
| JSON snapshot of the durable tables                                     | Dump the SQLite file                 | A portable export does not need the WAL or the FTS index            | The search index and run logs are excluded; there is no import |
| Clear only the recorded download paths                                  | Wipe the downloads folder            | The folder is user-configurable and may be broad                    | A file not recorded in the database is left alone              |
| Move a corrupt database aside                                           | Delete it and start over             | The old file is the only way to recover data                        | A backup file stays beside the database                        |

## 13. Extension points

**Add a data source**

1. Define `XSource` in `sync/sources.rs` and implement it on `LiveSources`.
2. Add a field to `SyncDeps`.
3. Add one block to `run_inner` with its own `Result`.
4. Add the tables it needs in a new migration; never edit an applied migration.
5. Add a `Source` variant only if it needs its own login and cookie.

**Add a screen**

1. Add the DTO in `crates/contracts/src/lib.rs`.
2. Add the read command in `commands/data.rs` and register it in `generate_handler!`.
3. Add the method to `src/api.rs` and the module under `src/features/`.
4. Add the route to `src/router.rs`; the sidebar entry is one row in
   `src/components/sidebar.rs`.
5. Add the copy to `locales/en.yml` and `locales/es.yml`, then regenerate
   `assets/app.css` if the screen introduces new classes.

**Add a command**

Keep it thin: read `AppState`, call a service, return `AppError` when it fails; it
serializes as `{kind, message}`.
