# Architecture

ParseKit is a macOS menu-bar app: a Tauri 2 shell (Rust) hosting a Svelte 5 + TypeScript UI, with document parsing isolated in a separate Rust sidecar binary.

```
+--------------------------- ParseKit.app ----------------------------+
|  Webview (Svelte 5, TS)  <--IPC-->  Tauri core (src-tauri/src/lib.rs) |
|   src/App.svelte                     tray, popover, windows, settings  |
|   src/components/*                   path validation, updater, trash   |
|   src/lib/*  (pure logic, tested)                                      |
|        |                                                               |
|        | shell plugin: spawn sidecar, stdin/stdout JSON lines          |
|        v                                                               |
|  parsekit-sidecar (src/bin/parsekit-sidecar.rs)                        |
|   LiteParse (pdfium + Tesseract OCR) -> Markdown / text / JSON         |
+------------------------------------------------------------------------+
   parsekit-cli (src/bin/parsekit-cli.rs): same engine for scripts/agents
```

## Components

| Path | Role |
|------|------|
| `src/App.svelte`, `OnboardingApp.svelte`, `ProgressHudApp.svelte` | Entry points for the three windows (`main`, `onboarding`, `progress-hud`) |
| `src/components/` | UI components (drop zone, settings, history, scoreboard, update banner) |
| `src/lib/sidecar.ts` | Spawns the sidecar, streams JSON-line events, maps raw errors to friendly messages, cancels by killing the child |
| `src/lib/store.ts` | Settings persistence (Tauri store plugin, `settings.json`) with a timeout guard |
| `src/lib/i18n.svelte.ts`, `src/locales/{en,es,zh}.json` | UI strings; `en` is the fallback |
| `src-tauri/src/lib.rs` | Tauri setup, IPC commands, tray/popover, path validation (`validate_user_path`), updater |
| `src-tauri/src/sidecar_helpers.rs` | Output paths/format, token-savings computation (shared by sidecar and CLI) |
| `src-tauri/src/token_stats.rs`, `token_count.rs` | Local token-savings counters (`token-stats.json`) |
| `src-tauri/src/clipboard_*.rs`, `global_hotkey.rs` | Clipboard-to-clipboard conversion and global shortcut |
| `src-tauri/src/macos_*.rs` | macOS-only: popover, notifications, Finder open-files, trash |
| `src-tauri/src/bin/parsekit-sidecar.rs` | Batch parser. Files are parsed one at a time (the native engine is not safe to run concurrently); page-level parallelism uses `workers`. 300 s timeout per file |
| `src-tauri/src/bin/parsekit-cli.rs` | `parsekit convert` command line |
| `sidecar/` | Legacy Node sidecar, development-only reference for the protocol |
| `scripts/` | Build, packaging, DMG, release and benchmark scripts |

## Sidecar protocol

The UI writes one JSON config line to stdin (`files`, `outputDir`, `format`, `ocrEnabled`, `ocrLanguage`, `workers`). The sidecar replies with JSON lines on stdout: `start`, `progress` (`parsing | completed | error | skipped`), `token_savings`, `done`, or a fatal `error`. Per-file failures are `progress` events with `status: "error"` and never abort the batch. The authoritative description is the doc comment at the top of `src/lib/sidecar.ts`.

## Data

All state is local: `settings.json` and `token-stats.json` in `~/Library/Application Support/com.harshabala.parsekit`. See [PRIVACY.md](PRIVACY.md).

## Security boundaries

Documents are untrusted and parsed only in the sidecar process. The webview has a strict CSP and a minimal Tauri capability set (see [SECURITY.md](SECURITY.md)). Filesystem access is mediated by Rust commands that canonicalize and validate paths.

## Build and release

`npm run build:sidecar` builds `src-tauri/binaries/parsekit-sidecar-<triple>`, which Tauri bundles as an `externalBin`. `npm run tauri build` produces the `.app` and DMG. Releases are described in [docs/RELEASING.md](docs/RELEASING.md).
