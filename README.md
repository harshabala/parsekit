<p align="center">
  <img src="assets/branding/app-icon.png" alt="ParseKit icon" width="96" height="96">
</p>

<h1 align="center">ParseKit</h1>

<h3 align="center">Turn documents into AI-ready Markdown</h3>

<p align="center">
  <a href="https://github.com/harshabala/parsekit/releases/latest/download/ParseKit_0.2.2_aarch64.dmg"><strong>Download for Mac (Apple Silicon)</strong></a>
  &nbsp;·&nbsp;
  <a href="docs/INSTALL.md">Install guide</a>
</p>

ParseKit is a native macOS menu-bar app that converts PDFs, Office files, spreadsheets, and images into clean Markdown, plain text, or JSON — entirely on your Mac.

---

## Why ParseKit?

Large language models work best with clean, structured text.

Unfortunately, PDFs, Office documents, and scanned files often require unnecessary parsing before the model can understand your content.

ParseKit converts those documents into structured Markdown locally on your Mac, making them easier to use with ChatGPT, Claude, Gemini, Codex, and other AI tools.

Everything happens on-device.

No cloud upload.

No subscriptions.

No API keys.

---

## Features

- Local-first processing
- Native macOS app
- AI-ready Markdown
- Plain Text export
- JSON export
- OCR for scanned documents
- Office document support
- PDF support
- Spreadsheet support
- Fast batch conversion
- Privacy-first — no telemetry

---

## Why Markdown?

Markdown is a clean, structured format that both humans and language models understand well.

Converting documents once before sharing them with an AI assistant helps:

- eliminate repeated parsing
- preserve document hierarchy
- create cleaner context
- improve prompt quality
- reduce unnecessary formatting overhead

ParseKit focuses on cleaner context and lower parsing overhead — not a promise that Markdown always uses fewer tokens.

---

## Get ParseKit

**You do not need `git clone`.** End users install the DMG:

1. **[Download the DMG](https://github.com/harshabala/parsekit/releases/latest/download/ParseKit_0.2.2_aarch64.dmg)** (macOS 12+, Apple Silicon)
2. Open it → drag **ParseKit** to **Applications**
3. Open from Applications → look for the icon in your **menu bar** (top-right)

First-launch security steps: **[docs/INSTALL.md](docs/INSTALL.md)**

---

## Privacy

Everything runs locally.

Your files never leave your Mac.

No cloud processing.

No telemetry.

No tracking.

---

## For developers

```bash
git clone https://github.com/harshabala/parsekit.git
cd parsekit
npm install
npm run build:sidecar
npm run tauri dev
```

Release notes: **[docs/RELEASING.md](docs/RELEASING.md)**

---

## Credits

Created and crafted by [Harsha Balakrishnan](https://github.com/harshabala).

Development help from Claude (Anthropic), Grok (xAI), and Gemini (Google) coding agents — see **[docs/ACKNOWLEDGMENTS.md](docs/ACKNOWLEDGMENTS.md)**.

Powered by [LiteParse v2](https://github.com/run-llama/liteparse) · [Tauri](https://tauri.app) · [Svelte](https://svelte.dev)

Apache-2.0 — see [LICENSE](LICENSE)