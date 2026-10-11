# Installing ParseKit

**Fast path:** [Download the DMG](https://github.com/harshabala/parsekit/releases/latest/download/ParseKit_0.2.11_aarch64.dmg) → open it → drag to Applications → open from Applications.

You do **not** need `git clone` or Terminal for a normal install.

## Requirements

- macOS 12 (Monterey) or newer  
- **Apple Silicon** Mac (M1–M4) — Apple menu → About This Mac  
- ~200 MB free disk  

## 1. Download

1. Open [Releases → Latest](https://github.com/harshabala/parsekit/releases/latest)
2. Download **`ParseKit_0.2.11_aarch64.dmg`**
3. Find it in **Downloads**

## 2. Install

1. Double-click the `.dmg`
2. **Drag ParseKit → Applications** (not the Desktop)
3. Eject the DMG
4. Open **ParseKit** from Applications

## 3. First launch (Gatekeeper)

ParseKit isn’t notarized with Apple yet. You only do this once.

**Easiest:** in Applications, **right-click ParseKit → Open → Open**.

**Or Terminal:**

```bash
xattr -cr /Applications/ParseKit.app
```

**Or:** System Settings → Privacy & Security → **Open Anyway**.

## 4. Find it

ParseKit lives in the **menu bar** (top-right), not the Dock.  
If you don’t see it, click the `›` overflow chevron on the menu bar.

## Optional converters

| Format | Needs |
| --- | --- |
| PDF | Nothing extra |
| Word / PowerPoint | [LibreOffice](https://www.libreoffice.org/download/) |
| Images (PNG, JPG…) | `brew install imagemagick` |

In the app: **Settings → File Support** shows what’s installed. Hit **Recheck** after installing.

## Finder Quick Action

**Settings → Finder → Install Finder Quick Action**, then right-click a file → **Quick Actions → Parse to Markdown with ParseKit**.

## Updating

Use the in-app gold banner (**Install & Restart**) when it appears.  
Or download the latest DMG and replace the app. Settings are kept separately.

## Still stuck?

[Open an issue](https://github.com/harshabala/parsekit/issues) with:

- macOS version  
- Apple Silicon or Intel  
- Which step failed and any error text  
