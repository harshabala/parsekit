# ParseKit

The fastest way to turn any folder of documents into LLM-ready files — zero terminal, zero cloud.

ParseKit — a toolkit for parsing documents. A lightweight macOS menu-bar app that batch-converts PDFs, Office docs, and images into clean Markdown, text, or JSON. Built for knowledge workers who need instant, private document preprocessing before feeding LLMs or building RAG indexes.

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

The `sidecar/` Node package is **dev-only** for testing the JSON protocol; the shipped app uses the Rust `parsekit-sidecar` binary.

## Release checklist

`src-tauri/binaries/` is not committed. Before packaging or distributing:

```bash
npm run release:macos   # build + scripts/postbuild-macos.sh (sign, strict verify, DMG)
```

Build on each target platform (Apple Silicon vs Intel Mac) so the correct host triple is embedded in the sidecar filename.

Output (Apple Silicon example):

- App: `src-tauri/target/release/bundle/macos/ParseKit.app`
- DMG: `src-tauri/target/release/bundle/dmg/ParseKit_<version>_aarch64.dmg`

### Install from DMG (ad-hoc signed, not notarized)

The release `.app` is ad-hoc signed with sealed resources (`codesign --verify --deep --strict` passes on the build artifact). Gatekeeper may still require a one-time approval for downloaded DMGs.

1. Drag **ParseKit** to **Applications**.
2. Clear Finder xattrs the installer adds (does not modify Mach-O):

```bash
xattr -cr /Applications/ParseKit.app
xattr -d com.apple.FinderInfo /Applications/ParseKit.app 2>/dev/null || true
```

3. **First launch:** Right-click **ParseKit** → **Open** → confirm, or use **Privacy & Security → Open Anyway**.
4. Use the **ParseKit** icon in the **menu bar** (top-right). ParseKit is menu-bar-only (`LSUIElement`); it does not remain in the Dock.

Popover debug traces (`/tmp/parsekit-popover-trace.log`) are written only in **debug** builds, not in release.

## How It Works

ParseKit uses [LiteParse v2](https://github.com/run-llama/liteparse) by LlamaIndex for document parsing. LiteParse v2 is a Rust-native engine (custom PDFium + built-in Tesseract OCR) — no cloud APIs, no API keys, no data ever leaves your machine.

### Architecture

1. **Tauri v2** — native macOS app with system tray
2. **Svelte 5** — reactive UI in the popover panel
3. **Rust sidecar binary** — a native `parsekit-sidecar` executable linked against LiteParse v2 (bundled with the app, no Node.js required at runtime)
4. **Tauri Store** — persists settings and batch history

## Privacy

ParseKit is designed with privacy as a core principle:

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
