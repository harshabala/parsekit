# Task 14 Report — Floating Progress HUD (opt-in)

**Branch:** `feat/agent-brief-tokens-tracker`  
**Date:** 2026-07-01

## Summary

Implemented an opt-in floating progress HUD for background parses. When **Settings → General → Floating progress HUD** is enabled, a second Tauri window (`progress-hud`, 280×90) appears during batches triggered from the global hotkey or Finder Quick Action (when ParseKit is already running). Default behavior is unchanged when the toggle is off.

## Changes

### Tauri

- **`tauri.conf.json`** — Added `progress-hud` window: 280×90, undecorated, transparent, `alwaysOnTop`, `skipTaskbar`, hidden at launch.
- **`capabilities/default.json`** — Granted permissions to `progress-hud` window.
- **`macos_popover.rs`** — `configure_hud_window` / `ensure_hud_window_configured` for floating NSWindow level.
- **`lib.rs`** — `show_progress_hud` (position bottom-right, show) and `hide_progress_hud` commands; hide HUD on app setup.
- **`macos_open_files.rs`** — Queue supports `{ background: true }` to emit `background-parse` instead of `open-files`.
- **`open-with-parsekit.sh`** — When `showFloatingHud` is on and ParseKit is running, queues background parse via app instead of headless sidecar.

### Frontend

- **`ProgressHud.svelte`** — Compact HUD: file fraction, progress bar, success/fail counts; on completion shows `+N tokens saved` and summary; auto-dismiss after 5s; clickable failed count reveals error + File Support link.
- **`ProgressHudApp.svelte`** — HUD window entry; listens for `hud-sync` events.
- **`progressHud.ts`** — State types, helpers, `emitTo` sync, show/hide IPC wrappers.
- **`main.ts`** — Mounts `ProgressHudApp` when window label is `progress-hud`.
- **`App.svelte`** — `showFloatingHud` setting; shows/syncs HUD only for `isBackgroundBatch` (hotkey / background quick action); listens for `hud-open-file-support`.
- **`SettingsScreen.svelte`** — Toggle `showFloatingHud` (default false) in General tab.
- **`index.css`** — `.progress-hud-*` styles.
- **`locales/en.json`, `zh.json`, `es.json`** — HUD + settings strings.
- **`progressHud.test.ts`** — Unit tests for HUD progress helpers.

### Build fix (pre-existing)

- **`clipboard_convert.rs`** — Added `use tauri::Manager` so `cargo build` succeeds on this branch.

## Settings storage

Key: `showFloatingHud` in `settings.json`  
Default: `false`

## Verification

```bash
npm run check   # 0 errors
npm run test    # 47/47 passed
cargo build     # success
```

### Manual smoke (macOS)

1. **Toggle off (default)** — Press ⌃⇧M with files selected: popover stays hidden, no HUD, completion notification only.
2. **Toggle on** — Enable **Floating progress HUD** in Settings → General.
3. Press ⌃⇧M — HUD appears bottom-right with progress; on finish shows `+N tokens saved` + summary; auto-hides after 5s.
4. Batch with errors — Click failed count in HUD; error reason shown; dependency errors link to File Support (opens main window).
5. Run Parse from popover UI — HUD does **not** appear (foreground batch unchanged).

## Commit

`feat(hud): opt-in floating progress panel for background parses`