# Development

## 1. Prerequisites

- Windows 10 or 11 with the WebView2 runtime (installed by default on current builds).
- Rust 1.99+ with `rustfmt` and `clippy` (`rustup component add rustfmt clippy`).
- Bun 1.4+ for the frontend. Avoid `npm` in PowerShell: the ExecutionPolicy blocks
  `npm.ps1`.

## 2. Setup

```powershell
bun install
bun run tauri dev
```

The first build compiles around 530 crates and takes several minutes. Stop `tauri dev`
before running `cargo test`: both write to the same target directory.

## 3. Commands

| Command                      | What it does                       |
| ---------------------------- | ---------------------------------- |
| `bun run dev`                | Vite only                          |
| `bun run tauri dev`          | Desktop app with hot reload        |
| `bun run build`              | Type check and bundle the frontend |
| `bun run test -- --run`      | Frontend tests once                |
| `bun run typecheck`          | `tsc --noEmit`                     |
| `bun run format`             | Prettier over `src/`               |
| `bun run format:check`       | Prettier in check mode             |
| `bun run check`              | Format check, type check and tests |
| `cargo test`                 | Rust tests, run from `src-tauri/`  |
| `cargo fmt`                  | Format Rust                        |
| `cargo clippy --all-targets` | Rust lints                         |

## 4. Definition of done

Before calling a task finished, from a clean tree:

```powershell
cd src-tauri; cargo fmt --check; cargo test; cargo clippy --all-targets
cd ..; bun run check
```

All four green. A failing test you did not cause still gets reported by name.

## 5. Formatting

- **Rust:** `rustfmt` with the edition from `Cargo.toml`. `rustfmt.toml` pins it so the
  formatter behaves the same outside `cargo fmt`.
- **TypeScript:** Prettier, with `prettier-plugin-tailwindcss` so class names sort
  consistently. `.prettierignore` skips build output, `bun.lock` and `src-tauri/`.
- `src/components/ui/` and the Coss composer helpers
  (`src/hooks/use-composer-editor.ts`, `src/hooks/use-media-query.ts`,
  `src/lib/composer-*.ts`) are excluded in `.prettierignore`: those files are copied
  from the registry with 2-space indentation and double quotes, and are replaced when
  the registry updates. The rest of `src/` follows the house style (4 spaces, single
  quotes, 80 columns).

## 6. Testing

- No test touches the network. HTML and JSON live in `src-tauri/tests/fixtures/` and
  were captured from the real account.
- Parser tests read fixtures with `include_str!`.
- Sync tests use fake sources (`tests/sync_orchestrator.rs`) and an in-memory database.
- Frontend tests use Vitest with `jsdom`.

Current totals: 64 Rust tests and 5 frontend tests.

## 7. Conventions

- Interface in Spanish; code, identifiers and commits in English.
- Conventional Commits, English, with bullets that describe the real scope. One commit
  per task.
- Comments only when they explain a constraint that the signature cannot.
- Errors carry a `kind` (`network`, `unauthorized`, `parse`, `not_found`,
  `rate_limited`, `internal`) and a message in Spanish for the UI.

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

**Rust target directory:** `tauri dev` and `cargo test` write to the same `target`
directory. Running both at once poisons the incremental cache and produces
`E0460`/`E0786` ("found possibly newer version of crate"). Stop the app before running
cargo, and note that `[profile.dev] incremental = false` is set in `Cargo.toml` for this
reason.

## 9. Data locations

| What      | Where                                                  |
| --------- | ------------------------------------------------------ |
| Database  | `%APPDATA%\com.santotomas.rustapp\panel.db` (WAL mode) |
| Downloads | `%USERPROFILE%\Downloads\PanelRamos`                   |
| Cookies   | WebView2 profile, never copied out                     |
| Logs      | stderr; `tracing` honours `RUST_LOG`                   |

## 10. Status

**Working:** desktop shell, SQLite schema, HTTP client with rate limiting, the three
source adapters, course linking, the sync orchestrator, the commands the UI consumes,
and the onboarding with per-source connection.

**Missing:** the data screens, the Windows installer with updates, and the end-to-end
checklist on a clean machine.

**Known gaps:** Moodle totals (the per-user report does not exist), Banner GPA (not
persisted), announcements (no course exposed a recognizable news forum) and section
fetches that could run four at a time.
