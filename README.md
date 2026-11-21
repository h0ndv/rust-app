# rust-app

Desktop client that gathers **Moodle**, **Ellucian Experience** and **Banner 9** in one
interface: courses, tasks, resources, schedule and grades. Everything is stored locally
in SQLite. The app only reads; it never submits, uploads or edits anything upstream.

The app does not have a name yet; `rust-app` is the placeholder used by the repository and the bundle.

## Requirements

- Windows 10 or 11 with the WebView2 runtime
- Rust 1.99+ with `rustfmt` and `clippy`
- Bun 1.4+

## Run it

```powershell
bun install
bun run tauri dev
```

## Test it

```powershell
bun run check
cd src-tauri; cargo test
```

## Documentation

| Document                                       | Contents                                                     |
| ---------------------------------------------- | ------------------------------------------------------------ |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Layers, authentication, sync pipeline, data model, decisions |
| [`docs/SOURCES.md`](docs/SOURCES.md)           | Verified endpoints and payloads of the three sources         |
| [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md)   | Commands, formatting, testing and known traps                |

## Structure

| Path                    | Contents                                               |
| ----------------------- | ------------------------------------------------------ |
| `src/`                  | React interface                                        |
| `src-tauri/src/`        | Rust backend: auth, adapters, database, sync, commands |
| `src-tauri/migrations/` | Versioned SQLite schema                                |
| `src-tauri/tests/`      | Integration tests and real fixtures                    |

## Data and privacy

The database lives in `%APPDATA%\com.santotomas.rustapp\panel.db` and downloads go to
`%USERPROFILE%\Downloads\rust-app`. Cookies stay in the WebView2 profile and are never
copied into the database or the logs. There is no central server, no telemetry and no
cloud account.
