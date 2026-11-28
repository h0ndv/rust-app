# rust-app

Desktop client that gathers **Moodle**, **Ellucian Experience** and **Banner 9** in one
interface: courses, tasks, resources, schedule and grades. Everything is stored locally
in SQLite. The app only reads; it never submits, uploads or edits anything upstream.

The app does not have a name yet; `rust-app` is the placeholder used by the repository and the bundle.

## Requirements

- Windows 10 or 11 with the WebView2 runtime
- Rust 1.99+ with `rustfmt`, `clippy` and the `wasm32-unknown-unknown` target
- `dx` 0.7.10 (`cargo install dioxus-cli --version 0.7.10 --locked`)
- `cargo-tauri` 2.x (`cargo install tauri-cli --version "^2" --locked`)
- The Tailwind standalone binary in `tools/tailwindcss.exe` (v4.3.5, not tracked)

## Run it

```powershell
cargo tauri dev
```

The script compiles the Tailwind CSS, starts the watcher and serves the Dioxus interface
inside the Tauri window. To work on the interface alone, run `dx serve`.

## Test it

```powershell
cargo test -p rust-app-ui
cargo test -p rust-app-contracts
cd src-tauri; cargo test
```

191 interface tests, 6 contract tests and 105 backend tests.

## Build it

```powershell
cargo tauri build
```

The installers land in `target\release\bundle` (`msi` and `nsis`). `dx bundle --release`
alone copies the web bundle to `dist\public`, the directory Tauri embeds.

## Documentation

| Document                                       | Contents                                                     |
| ---------------------------------------------- | ------------------------------------------------------------ |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Layers, authentication, sync pipeline, data model, decisions |
| [`docs/SOURCES.md`](docs/SOURCES.md)           | Verified endpoints and payloads of the three sources         |
| [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md)   | Commands, formatting, testing and known traps                |

## Structure

| Path                    | Contents                                                    |
| ----------------------- | ----------------------------------------------------------- |
| `crates/contracts/`     | DTOs shared by the backend and the interface                |
| `src/`                  | Dioxus interface: router, state, components, ten screens    |
| `tests/`                | Dioxus SSR tests for the interface                          |
| `locales/`              | English and Spanish catalogs                                |
| `assets/`               | Tailwind sources, generated `app.css`, self-hosted fonts    |
| `src-tauri/src/`        | Rust backend: auth, adapters, database, sync, commands      |
| `src-tauri/migrations/` | Versioned SQLite schema                                     |
| `src-tauri/tests/`      | Integration tests and real fixtures                         |

## Data and privacy

The database lives in `%APPDATA%\com.santotomas.rustapp\panel.db` and downloads go to
`%USERPROFILE%\Downloads\rust-app`. Cookies stay in the WebView2 profile and are never
copied into the database or the logs. There is no central server, no telemetry and no
cloud account.
