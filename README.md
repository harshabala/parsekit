# ParseDock

The fastest way to turn any folder of documents into LLM-ready files — zero terminal, zero cloud.

ParseDock is a lightweight macOS menu-bar app that batch-converts PDFs, Office docs, and images into clean Markdown, text, or JSON. Built for knowledge workers who need instant, private document preprocessing before feeding LLMs or building RAG indexes.

## Features

- **Menu-bar native** — lives in your tray, one click away
- **Batch processing** — drop a folder, get clean output in seconds
- **Smart Markdown** — document titles, page headers, and separators
- **OCR built-in** — powered by Tesseract.js, works offline
- **100% local** — your documents never leave your machine
- **Dark mode** — follows macOS system preference

## Supported Formats

| Input | Output |
|-------|--------|
| PDF | Markdown (.md) |
| DOCX, DOC | Plain Text (.txt) |
| PPTX, PPT | JSON (.json) |
| XLSX, XLS | |
| PNG, JPG, JPEG | |
| TIFF, BMP | |

## Prerequisites

- **macOS** 12.0+
- **Node.js** 20+ (for document parsing engine)
- **Rust** (for development only)
- **LibreOffice** (optional, for Office document conversion)

## Development

```bash
# Install dependencies
npm install
cd sidecar && npm install && cd ..

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## How It Works

ParseDock uses [LiteParse](https://github.com/run-llama/liteparse) by LlamaIndex for document parsing. LiteParse runs entirely locally using PDF.js and Tesseract.js — no cloud APIs, no API keys, no data ever leaves your machine.

### Architecture

1. **Tauri v2** — native macOS app with system tray
2. **Svelte 5** — reactive UI in the popover panel
3. **Node.js sidecar** — runs LiteParse for document processing
4. **Tauri Store** — persists settings and batch history

## Privacy

ParseDock is designed with privacy as a core principle:

- All processing happens locally on your machine
- No telemetry, no analytics, no tracking
- No network requests (except optional first-run Tesseract model download)
- Your documents are never uploaded anywhere

## License

Apache-2.0 — see [LICENSE](LICENSE)

## Credits

- [LiteParse](https://github.com/run-llama/liteparse) by LlamaIndex — local document parsing engine
- [Tauri](https://tauri.app) — native app framework
- [Svelte](https://svelte.dev) — reactive UI framework
