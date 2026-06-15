# ParseKit

The fastest way to turn any folder of documents into LLM-ready files — zero terminal, zero cloud.

ParseKit — a toolkit for parsing documents. A lightweight macOS menu-bar app that batch-converts PDFs, Office docs, and images into clean Markdown, text, or JSON. Built for knowledge workers who need instant, private document preprocessing before feeding LLMs or building RAG indexes.

## Features

- **Menu-bar native** — lives in your tray, one click away
- **Batch processing** — drop a folder, get clean output in seconds
- **Smart Markdown** — document titles, page headers, and separators
- **OCR built-in** — powered by LiteParse v2’s native Tesseract engine, works offline
- **LiteParse v2** — Rust core for up to ~100× faster parsing on small docs
- **On-device parsing** — no cloud upload; optional one-time Tesseract language data download when OCR is on
- **Automatic updates** — checks GitHub Releases on launch; in-app banner to install (menu-bar friendly, no system dialog)
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
npm run release:macos    # build + scripts/postbuild-macos.sh (sign, strict verify, DMG)
npm run publish:macos    # release build + sign updater bundle + upload to GitHub Releases
```

Build on each target platform (Apple Silicon vs Intel Mac) so the correct host triple is embedded in the sidecar filename.

### Automatic updates

ParseKit includes the [Tauri v2 updater](https://v2.tauri.app/plugin/updater/). On launch (and via **Settings → Updates → Check for updates**), the app fetches a manifest from GitHub Releases:

`https://github.com/harshabala/parsekit/releases/latest/download/parsekit-latest.json`

When a newer version is available, a gold banner appears in the popover: **Install & Restart** or **Later**. Updates download a signed `.app.tar.gz` and replace the app bundle (not the DMG installer flow).

The updater tarball is built **after** `postbuild-macos.sh` seals the `.app` with `codesign --deep` (`createUpdaterArtifacts` is **off** so Tauri does not emit a pre-sign tar.gz).

**Publishing a release (maintainers):**

1. Generate a signing key once (keep private key local, never commit):

   ```bash
   npx tauri signer generate -w ~/.tauri/parsekit.key -f --ci -p ''
   ```

2. The public key is already in `src-tauri/tauri.conf.json` (`plugins.updater.pubkey`).

3. Build and upload:

   ```bash
   export TAURI_SIGNING_PRIVATE_KEY="$HOME/.tauri/parsekit.key"
   export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
   RELEASE_NOTES="ParseKit v0.2.1 — …" npm run publish:macos
   ```

   This uploads the DMG, `ParseKit_<version>_aarch64.app.tar.gz`, and `parsekit-latest.json` to the `v<version>` GitHub release. The repo must be **public** so clients can download assets without authentication.

**Note:** GitHub serves `latest.json` as 404; the manifest must be named **`parsekit-latest.json`**.

Output (Apple Silicon example):

- App: `src-tauri/target/release/bundle/macos/ParseKit.app`
- DMG: `src-tauri/target/release/bundle/dmg/ParseKit_<version>_aarch64.dmg`

### Install from DMG (ad-hoc signed, not notarized)

The release `.app` is ad-hoc signed with sealed resources (`codesign --verify --deep --strict` passes on the build artifact). Gatekeeper may still require a one-time approval for downloaded DMGs.

The DMG opens a guided installer window: **“Drag ParseKit to Applications”** with a frosted layout, arrow cue, and icon labels (same idea as modern app installers). Finder cannot animate that window live — the guidance is baked into the background art.

1. Drag **ParseKit** to **Applications** (not Desktop or Downloads). Quit any copy running from the DMG, then open **ParseKit** from Applications only.
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

- **On-device parsing — no cloud upload.** Files are read and written only on your Mac.
- No telemetry, no analytics, no tracking
- No network during parsing except optional Tesseract language data (OCR) and optional update checks when you use automatic updates
- Your documents are never uploaded anywhere

## License

Apache-2.0 — see [LICENSE](LICENSE)

## Credits

- [LiteParse](https://github.com/run-llama/liteparse) by LlamaIndex — local document parsing engine
- [Tauri](https://tauri.app) — native app framework
- [Svelte](https://svelte.dev) — reactive UI framework
