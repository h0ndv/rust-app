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
- Downloads Moodle attachments and opens them with the system app.

**Non-goals**

- Attendance. Not exposed by any verified source.
- Submitting, uploading or editing anything.
- Recomputing averages or mixing Moodle and Banner grades.
- Storing passwords.
- Central server, accounts, ads or telemetry.
- Mobile apps, macOS or Linux in the first version.

## 3. Layers

```text
React UI (src/)
   │  invoke(command, payload) / listen("sync://…")
   ▼
commands/            Tauri boundary: DTOs in, DTOs out, errors as strings
   │
   ├── sync/         orchestrator, novelty rules, archiving, download queue
   ├── db/           SQLite pool, repositories, schema, course linking
   ├── auth/         login windows, cookie bridge, source health
   ├── http/         rate-limited reqwest client
   └── moodle/  ellucian/  banner/     one adapter per source
```

The interface never talks to Moodle, Ellucian or Banner. It only calls commands, and
Rust owns every decision. That keeps the three integration quirks in one language and
keeps the UI replaceable.

## 4. Module map

| Path                      | Responsibility                                                             |
| ------------------------- | -------------------------------------------------------------------------- |
| `src/`                    | React screens, components and the Tauri client                             |
| `src-tauri/src/lib.rs`    | Builder, setup, DB bootstrap, command registry                             |
| `src-tauri/src/state.rs`  | `AppState { pool, repo, sync }`                                            |
| `src-tauri/src/error.rs`  | `AppError` with `kind` and a Spanish message                               |
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

The app must treat "no session" as a normal startup state, not as an error.

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

## 10. Commands and DTOs

`commands/` exposes 20 commands. Reads return DTOs from `commands/dto.rs`; the UI never
sees database models.

| Group    | Commands                                                                                                                            |
| -------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| Session  | `connect(source?)`, `session_state`                                                                                                 |
| Sync     | `start_sync`, `cancel_sync`, `sync_status`                                                                                          |
| Reads    | `get_dashboard`, `list_courses`, `get_course`, `list_tasks`, `list_resources`, `list_grades`, `get_schedule`, `list_news`, `search` |
| Settings | `get_settings`, `save_settings`                                                                                                     |
| Actions  | `open_in_app`, `download_file`, `clear_data`                                                                                        |

Filters (`TaskFilter`, `ResourceFilter`) travel lowercase so the JSON stays
`"overdue"`, `"new"`.

Events emitted to the UI: `sync://progress`, `sync://finished`, `download://finished`,
`download://error`.

## 11. Decisions and tradeoffs

| Decision                                                                | Alternative                         | Why                                                                 | Cost if wrong                                           |
| ----------------------------------------------------------------------- | ----------------------------------- | ------------------------------------------------------------------- | ------------------------------------------------------- |
| One trait per source (`MoodleSource`, `EllucianSource`, `BannerSource`) | A single `Sources` trait            | A new source is additive; a test fakes only what it needs           | Three traits instead of one                             |
| `SyncDeps` struct                                                       | Five constructor arguments          | New dependencies do not change signatures                           | One more struct                                         |
| Injected `EventEmitter`                                                 | `AppHandle` inside the orchestrator | Keeps Tauri out of the library and test binaries                    | One closure in `lib.rs`                                 |
| `async-trait`                                                           | Hand-written `BoxFuture`            | `dyn` needs boxing anyway; the macro removes 13 signatures of noise | One proc-macro dependency                               |
| No foreign keys                                                         | `REFERENCES` + `ON DELETE CASCADE`  | Sync writes sources before the union                                | Orphan rows that archiving already tolerates            |
| `last_seen_at` column                                                   | Reuse `first_seen_at`               | 7-day archiving needs the last time an item was seen                | One migration                                           |
| Resources are files, not pages                                          | Fetch `mod/resource/view.php`       | The page redirects to the PDF; reading it as text hung the sync     | Attachment metadata comes from the link instead         |
| Baseline per course                                                     | Global first run                    | A new course added mid-semester should not flood Novedades          | A newly added course never marks its first items as new |

## 12. Extension points

**Add a data source**

1. Define `XSource` in `sync/sources.rs` and implement it on `LiveSources`.
2. Add a field to `SyncDeps`.
3. Add one block to `run_inner` with its own `Result`.
4. Add the tables it needs in a new migration; never edit an applied migration.
5. Add a `Source` variant only if it needs its own login and cookie.

**Add a screen**

1. Add the DTO in `commands/dto.rs`.
2. Add the read command in `commands/data.rs`.
3. Register it in `generate_handler!`.
4. Add the route in `src/app/router.tsx` and the Coss components for it.

**Add a command**

Keep it thin: read `AppState`, call a service, map `AppError` to `String`.
