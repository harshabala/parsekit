# ParseDock

The fastest way to turn any folder of documents into LLM-ready files — zero terminal, zero cloud.

ParseDock is a lightweight macOS menu-bar app that batch-converts PDFs, Office docs, and images into clean Markdown, text, or JSON. Built for knowledge workers who need instant, private document preprocessing before feeding LLMs or building RAG indexes.

## Features

- **Menu-bar native** — lives in your tray, one click away
- **Batch processing** — drop a folder, get clean output in seconds
- **Smart Markdown** — document titles, page headers, and separators
- **OCR built-in** — powered by LiteParse v2’s native Tesseract engine, works offline
- **LiteParse v2** — Rust core for up to ~100× faster parsing on small docs
- **100% local** — your documents never leave your machine
- **Dark mode** — follows macOS system preference

## Supported Formats

| Input | Output |
|-------|--------|
| PDF | Markdown (.md), Plain Text (.txt), or JSON (.json) |
| Word (DOC, DOCX, ODT, RTF, …) | Same (via LibreOffice) |
| PowerPoint (PPT, PPTX, ODP, …) | Same (via LibreOffice) |
| Spreadsheets (XLS, XLSX, CSV, …) | Always JSON (.json) |
| Images (PNG, JPG, WEBP, SVG, …) | Same (via ImageMagick) |

## Prerequisites

- **macOS** 12.0+
- **Node.js** 20+ (frontend build tooling)
- **Rust** (Tauri app + LiteParse v2 sidecar)
- **LibreOffice** (optional, for Office document conversion)
- **ImageMagick** (optional, for image formats)

## Development

```bash
# Install dependencies
npm install

# Build the LiteParse v2 sidecar (skipped if already up to date)
npm run build:sidecar

# Run in development mode (rebuilds sidecar only when sources change)
npm run tauri dev

# Skip sidecar rebuild if you already built it (faster iteration)
npm run tauri:dev:fast
```

**First-time note:** The initial `build:sidecar` compiles LiteParse v2 and Tesseract (~10 minutes on a clean machine). Later runs are incremental and usually instant.

The `sidecar/` Node package is **dev-only** for testing the JSON protocol; the shipped app uses the Rust `parsedock-sidecar` binary.

## Release checklist

`src-tauri/binaries/` is not committed. Before packaging or distributing:

```bash
npm run build
npm run build:sidecar   # produces src-tauri/binaries/parsedock-sidecar-<host-triple>
npm run tauri build
```

Build on each target platform (Apple Silicon vs Intel Mac) so the correct host triple is embedded in the sidecar filename.

Output (Apple Silicon example):

- App: `src-tauri/target/release/bundle/macos/ParseDock.app`
- DMG: `src-tauri/target/release/bundle/dmg/ParseDock_0.1.1_aarch64.dmg` (exact name may vary by Tauri version)

### Install from DMG (unsigned / ad-hoc)

Release builds are not notarized. After opening the DMG, if macOS blocks the app:

1. Drag **ParseDock** to Applications.
2. Optional ad-hoc sign (reduces Gatekeeper friction for local use):

```bash
codesign --force --deep --sign - /Applications/ParseDock.app
xattr -cr /Applications/ParseDock.app
```

3. First launch: **System Settings → Privacy & Security → Open Anyway**, or right-click the app → **Open**.

Popover debug traces (`/tmp/parsedock-popover-trace.log`) are written only in **debug** builds, not in release.

## How It Works

ParseDock uses [LiteParse v2](https://github.com/run-llama/liteparse) by LlamaIndex for document parsing. LiteParse v2 is a Rust-native engine (custom PDFium + built-in Tesseract OCR) — no cloud APIs, no API keys, no data ever leaves your machine.

### Architecture

1. **Tauri v2** — native macOS app with system tray
2. **Svelte 5** — reactive UI in the popover panel
3. **Rust sidecar binary** — a native `parsedock-sidecar` executable linked against LiteParse v2 (bundled with the app, no Node.js required at runtime)
4. **Tauri Store** — persists settings and batch history

## Privacy

ParseDock is designed with privacy as a core principle:

- All processing happens locally on your machine
- No telemetry, no analytics, no tracking
- No network requests during parsing (optional first-run Tesseract language data may download when OCR is enabled)
- Your documents are never uploaded anywhere

## License

Apache-2.0 — see [LICENSE](LICENSE)

## Credits

- [LiteParse](https://github.com/run-llama/liteparse) by LlamaIndex — local document parsing engine
- [Tauri](https://tauri.app) — native app framework
- [Svelte](https://svelte.dev) — reactive UI framework
