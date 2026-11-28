# Development

## 1. Prerequisites

- Windows 10 or 11 with the WebView2 runtime (installed by default on current builds).
- Rust 1.99+ with `rustfmt` and `clippy` (`rustup component add rustfmt clippy`) and the
  WebAssembly target (`rustup target add wasm32-unknown-unknown`).
- `dx` 0.7.10, the Dioxus CLI (`cargo install dioxus-cli --version 0.7.10 --locked`).
- `cargo-tauri` 2.x (`cargo install tauri-cli --version "^2" --locked`).
- The Tailwind standalone binary, v4.3.5, at `tools/tailwindcss.exe`. It is not tracked;
  download it from the Tailwind CSS releases page and keep the same version.

## 2. Setup

```powershell
cargo tauri dev
```

`tauri dev` calls `scripts/dev.ps1`, which compiles `assets/tailwind.css` into
`assets/app.css`, starts Tailwind in watch mode and runs `dx serve` for the WebView.
The first build compiles around 500 crates and takes several minutes.

To work on the interface without the desktop shell:

```powershell
dx serve
```

Stop `tauri dev` before running `cargo test`: both write to the same target directory.

## 3. Commands

| Command                                                       | What it does                                 |
| ------------------------------------------------------------- | -------------------------------------------- |
| `cargo tauri dev`                                             | Desktop app with hot reload                  |
| `dx serve`                                                     | Web bundle with hot reload                   |
| `dx build`                                                     | Web bundle under `target/dx`                 |
| `dx build --release`                                           | Optimised WebAssembly bundle                 |
| `dx bundle --release`                                          | Bundle copied to `dist/public`               |
| `cargo tauri build`                                            | Installers under `target/release/bundle`     |
| `cargo test -p rust-app-ui`                                   | 191 interface tests                          |
| `cd src-tauri; cargo test`                                    | 105 backend tests                            |
| `cargo test -p rust-app-contracts`                            | 6 wire-format tests                          |
| `cargo test -p rust-app-ui --test e2e -- --ignored`           | Windows WebDriver smoke (needs tooling)      |
| `wasm-pack test --headless --chrome --test wasm_smoke --features web` | Browser smoke for the Dioxus runtime |
| `cargo fmt --all --check`                                      | Rust formatting, the workspace               |
| `cargo clippy --workspace --all-targets`                      | Rust lints                                   |
| `cargo check -p rust-app-ui --target wasm32-unknown-unknown --features web` | Wasm-only build check     |
| `tools\tailwindcss.exe -i assets\tailwind.css -o assets\app.css --minify` | Regenerate the CSS            |

## 4. Definition of done

Before calling a task finished, from a clean tree:

```powershell
cargo fmt --all --check
cargo test -p rust-app-ui
cargo test -p rust-app-contracts
cd src-tauri; cargo test; cd ..
cargo clippy --workspace --all-targets
cargo check -p rust-app-ui --target wasm32-unknown-unknown --features web
```

All green. A failing test you did not cause still gets reported by name.

## 5. Formatting

- **Rust:** `rustfmt` with the edition from `Cargo.toml`. `src-tauri/rustfmt.toml` pins
  the backend options (max width 100, field-init shorthand), so `cargo fmt` behaves the
  same outside the workspace; the interface crate uses the defaults.
- **Tailwind:** class strings live inside Rust attributes; there is no automatic class
  sorter. `assets/app.css` is generated, minified and **committed**, because
  `Dioxus.toml` embeds `assets/` as the asset directory. Regenerate it with the command
  above whenever a class changes, and never edit it by hand.
- `assets/tailwind.css` is the source: the theme tokens, the fonts, the vendored
  `tw-animate.css` and `shadcn-tailwind.css`, and the `@source "../src/**/*.rs"` line
  that tells Tailwind where the class names are.

## 6. Testing

- No test touches the network. HTML and JSON live in `src-tauri/tests/fixtures/` and
  were captured from the real account.
- Parser tests read fixtures with `include_str!`.
- Sync tests use fake sources (`tests/sync_orchestrator.rs`) and an in-memory database.
- Interface tests render with `dioxus-ssr` and inject a fake `Transport`, so a screen
  can be asserted without Tauri. Native builds use `UnsupportedTransport`; the real
  `TauriTransport` only compiles for `wasm32`.
- `crates/contracts/tests/wire.rs` pins the JSON shape shared by both sides.

Current totals: 191 interface tests, 105 backend tests and 6 contract tests.

## 7. Conventions

- Interface defaults to English, with Spanish as the second locale. Code, identifiers and commits in English.
- Conventional Commits, English, with bullets that describe the real scope. One commit
  per task.
- Comments only when they explain a constraint that the signature cannot.
- Errors carry a `kind` (`network`, `unauthorized`, `forbidden`, `parse`,
  `not_found`, `rate_limited`, `internal`) and a technical message in English;
  the interface localizes by `kind`.
- The interface never calls `invoke` outside `src/transport.rs`; screens go through
  `Api`, and the payloads are the types from `contracts`.

## 8. Known traps

| Symptom                                                    | Cause                                                                       | Fix                                                                    |
| ---------------------------------------------------------- | --------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| App freezes with `Responding=False` when opening a window  | Window created from a sync command; the main thread blocks in `build()`     | Make the command `async`                                               |
| `cookies_for_url` never returns                            | Called from the main thread or a window event handler                       | `spawn_blocking` inside an `async` command                             |
| `STATUS_ENTRYPOINT_NOT_FOUND` in tests                     | `AppHandle::emit` pulls Tauri and `WebView2Loader.dll` into the test binary | Inject `EventEmitter` from `lib.rs`                                    |
| Sync stalls on a course                                    | `reqwest`'s timeout does not cover the body; a large PDF blocks forever     | `http::read_text` / `read_bytes`, and resources are not fetched at all |
| `attempt to multiply with overflow`                        | `step * 100` with `step: u8`                                                | Compute in `u32`                                                       |
| Duplicate course when a source fails                       | Banner creates `banner:CRN` next to `moodle:ID`                             | Resolve the CRN against the database before upserting                  |
| `migration 1 was previously applied but has been modified` | A database created by an older schema                                       | Move the database aside; never edit an applied migration               |
| `FileName` escapes the downloads folder                    | Only separators were replaced                                               | `safe_name` keeps the last path component                              |
| Moodle answers HTML instead of JSON                        | Session expired                                                             | Detect login `errorcode` and return `Unauthorized`                     |
| `dist/public` missing when `cargo tauri build` runs        | `dx build` writes under `target/dx/<app>/<profile>/web/public`               | Use `dx bundle` (or `scripts/build.ps1`), which copies to `out_dir`    |
| A class added in Rust has no CSS                           | Tailwind scanned the files before the class existed                         | Regenerate `assets/app.css`; `dx serve` only watches, it does not scan  |
| The interface cannot find a class or the fonts at runtime  | `app.css` refers to `./fonts/*.woff2` and the folder was not bundled         | Keep the folder `asset!` next to the stylesheet `asset!` in `src/app.rs` |
| Wasm release fails inside wasm-bindgen                     | `strip = true` removes the custom sections it reads                         | `[profile.wasm-release]` overrides `strip` to `false`                   |
| Interface test hangs or panics on `__TAURI__`              | Native test binary has no Tauri IPC                                          | Inject `FakeTransport`; never call the real bridge in a native test     |

**Rust target directory:** `tauri dev`, `dx` and `cargo test` write to the same `target`
directory. Running them at once poisons the incremental cache and produces
`E0460`/`E0786` ("found possibly newer version of crate"). Stop the app before running
cargo, and note that `[profile.dev] incremental = false` is set in `Cargo.toml` for this
reason.

## 9. Data locations

| What              | Where                                                           |
| ----------------- | --------------------------------------------------------------- |
| Database          | `%APPDATA%\com.santotomas.rustapp\panel.db` (WAL mode)          |
| Downloads         | `%USERPROFILE%\Downloads\rust-app`                              |
| Exports           | `<downloads_dir>\rust-app-export-<timestamp>.json`              |
| Corrupt DB backup | `%APPDATA%\com.santotomas.rustapp\panel.db.corrupt-<timestamp>` |
| Cookies           | WebView2 profile, never copied out                              |
| Logs              | stderr; `tracing` honours `RUST_LOG`                            |

## 10. Status

**Working:** desktop shell, SQLite schema, HTTP client with rate limiting, the three
source adapters, course linking, the sync orchestrator, the commands the UI consumes,
the onboarding with per-source connection, cached startup with a reconnect banner,
Novedades with acknowledgment and all task statuses, the ten data screens in Dioxus, the
English/Spanish interface and the persisted settings.

**Missing:** the manual checklist in §11 on a clean machine. The automated layer passes,
including the wasm smoke; only the WebDriver e2e still needs its drivers.

**Known gaps:** Moodle totals (the per-user report does not exist), Banner GPA (not
persisted), announcements (no course exposed a recognizable news forum) and section
fetches that could run four at a time.

## 11. Windows end-to-end checklist

The automated layer runs from the repository. The manual layer needs a clean
Windows 10/11 machine or VM with WebView2, the installer and the real Microsoft
account. **None of the manual rows below has been executed by the developer**;
do not mark one as passed from a machine that already has the app profile.

### 11.1 Automated layer

Checked on 2026-11-25 on the development machine:

| Check                    | Command                                                                  | Result                    |
| ------------------------ | ------------------------------------------------------------------------ | ------------------------- |
| Formatting               | `cargo fmt --all --check`                                                | pass                      |
| Interface                | `cargo test -p rust-app-ui`                                              | 191 passed, 1 ignored     |
| Backend                  | `cargo test -p rust-app`                                                 | 105 passed                |
| Contracts                | `cargo test -p rust-app-contracts`                                       | 6 passed                  |
| Lints                    | `cargo clippy --workspace --all-targets`                                 | pass, no warnings         |
| Wasm build               | `cargo check -p rust-app-ui --target wasm32-unknown-unknown --features web` | pass                   |
| Wasm tests build         | `cargo build --tests --test wasm_smoke --target wasm32-unknown-unknown --features web` | pass |
| Wasm smoke runs          | `wasm-pack test --headless --chrome --test wasm_smoke --features web`    | 1 passed                  |
| Desktop e2e compiles     | `cargo check -p rust-app-ui --test e2e`                                  | pass                      |

The commands below run both smoke layers; the desktop one still needs tools that
are not installed on this machine:

```powershell
# Browser smoke: mounts a small Dioxus component, renders text, clicks it.
wasm-pack test --headless --chrome --test wasm_smoke --features web

# Desktop smoke: launches the built app, asserts the nine sidebar links and /notas.
cargo test -p rust-app-ui --test e2e -- --ignored --nocapture
```

The wasm test mounts a local `Counter`, not `App`: `App` builds a
`TauriTransport` that calls `invoke`, and a browser test has no `__TAURI__`
bridge. The isolated component still exercises the real Dioxus web renderer,
DOM mounting and event delegation. The exact wasm build step
(`cargo build --tests --test wasm_smoke --target wasm32-unknown-unknown
--features web`) passes.

- **Wasm smoke:** pass. `wasm-pack` 0.15.1 fetched chromedriver
  158.0.8127.88 and Chrome 158.0.8127.37 ran the test in 0.05 s. The original
  blocker was `wasm-pack` itself missing, not the driver.
- **Desktop e2e:** blocked by `tauri-driver` and `msedgedriver` missing. The
  test is `#[ignore]` and never installs the MSI. It expects
  `target/release/rust-app.exe` (override with `RUST_APP_E2E_BINARY`) and an
  app profile that already completed onboarding; otherwise the app opens on
  onboarding and no sidebar exists.
- **Profile safety:** launching the release app uses the real
  `%APPDATA%\com.santotomas.rustapp` SQLite database and the real WebView2
  profile, so the run **may write** settings, session state, window state and
  sync rows. Use a disposable Windows account/VM, or back up and restore both
  folders. Setting `APPDATA`/`LOCALAPPDATA` is not enough because Tauri resolves
  the Windows known folders. Never run it against the daily profile.

### 11.2 Manual layer

| #  | Check                       | Pass criteria                                                                                                   | Status |
| -- | --------------------------- | --------------------------------------------------------------------------------------------------------------- | ------ |
| 1  | Install the MSI             | `target/release/bundle/msi/rust-app_0.1.0_x64_en-US.msi` installs, adds a Start menu entry and uninstall entry, and launches. Repeat with `nsis/rust-app_0.1.0_x64-setup.exe` if that build is shipped. | [ ] |
| 2  | Real onboarding             | All three sources (Moodle, Ellucian, Banner) connect with the real account. Password and MFA are typed only in the Microsoft window. | [ ] |
| 3  | First sync                  | The sync finishes without a frozen window, reports requests and per-source counts, and the badge turns into "Up to date". | [ ] |
| 4  | Cached restart              | Close the app, disconnect the network, reopen it: cached data is visible and the session banner announces the missing session. | [ ] |
| 5  | Ten screens                 | Home, Courses, Course detail, Tasks, Resources, Grades, Schedule, News, Search and Settings each open without a panic and show data or a localized empty note. | [ ] |
| 6  | English and Spanish         | Switching in Settings updates the sidebar, all screens, dates, numbers and grades without a reload, and survives a restart. | [ ] |
| 7  | Light, dark and system      | All three themes are readable; `system` follows the Windows app theme when it changes. | [ ] |
| 8  | Downloads                   | A resource downloads into `%USERPROFILE%\Downloads\rust-app`, gets the "Downloaded" marker and survives a restart. | [ ] |
| 9  | In-app viewer               | Opening a resource link shows the in-app window; closing it returns to the app. A rejected URL shows a localized toast. | [ ] |
| 10 | Search                      | A query finds courses, tasks and resources; an empty query shows the empty state; Enter runs the search. | [ ] |
| 11 | News mark-read              | Acknowledging an item updates the badge and stays acknowledged after a restart. | [ ] |
| 12 | Export                      | Export writes `<downloads_dir>\rust-app-export-<timestamp>.json` and the file is valid JSON with the expected collections. | [ ] |
| 13 | Clear downloads             | Confirming removes the downloaded files from disk, keeps the local catalogue and reports the count. | [ ] |
| 14 | Dead keys `áéíóúñ`          | Typing with a dead-key keyboard in Search and Settings produces the accented characters, with no dropped tilde or doubled key. | [ ] |
| 15 | Tab order and focus         | Tab walks sidebar, header, content and the Settings form in visual order, the focus ring is visible, there is no trap, and Enter/Space activate the focused control. | [ ] |
| 16 | DPI 125% and 150%           | Text is not clipped, the sidebar keeps full height, there is no horizontal scroll and dialogs stay on screen at both settings. | [ ] |
| 17 | Session-expiry banner       | With an expired source the banner names Moodle, Ellucian or Banner, "Reconnect" reopens the login, and cached data stays usable. | [ ] |

