# Privacy

ParseKit is designed so your documents never leave your Mac.

## What stays on your device

- **Documents and converted output.** Parsing and OCR run locally in the bundled `parsekit-sidecar` process. Output is written only to the folder you choose.
- **Settings.** Stored by the Tauri store plugin in `settings.json` in the app's Application Support directory (output folder, language, theme, shortcut, recent job counts).
- **Local metrics.** `token-stats.json` (also in Application Support) holds counters such as files converted and estimated tokens saved. It contains file *types* and numbers, not file names or contents. Settings lets you reset it.
- **Error reports.** Only written when you click to save one, to a folder you control, and contain file names and error text for failed files. Review before sharing.

## What ParseKit never does

- No telemetry, analytics, crash uploads, or accounts.
- No API keys; nothing is sent to any AI service. (You may paste output into one yourself.)
- No logging of document contents. Diagnostics (console and sidecar stderr) carry error messages and file names only.

## The only network use

| Purpose | Destination | When |
|---------|-------------|------|
| Update check and download | `github.com/harshabala/parsekit/releases` | When you open the app / press "Check for updates" |
| OCR language packs | Tesseract language data host (only for non-English OCR) | First use of that language |

Both are optional; the app works fully offline for conversion.

## Deleting your data

Quit ParseKit and delete its Application Support folder (`~/Library/Application Support/com.harshabala.parsekit`) and, if desired, the converted output folder.
