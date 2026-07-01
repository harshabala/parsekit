# Installing ParseKit on your Mac

**Fast path:** [Download the DMG](https://github.com/harshabala/parsekit/releases/latest/download/ParseKit_0.2.4_aarch64.dmg) → open it → drag to Applications → open from Applications.

You do **not** need `git clone` or Terminal for a normal install. This guide covers the details macOS doesn't explain well.

## Before you start

- **macOS 12 (Monterey) or newer**
- **Apple Silicon Mac** (M1, M2, M3, M4). Check: Apple menu → **About This Mac**. If it says "Chip: Apple M…", you're good.
- About 200 MB of disk space

## Step 1 — Download

1. Open **[github.com/harshabala/parsekit/releases/latest](https://github.com/harshabala/parsekit/releases/latest)** in Safari or Chrome.
2. Under **Assets**, click **`ParseKit_0.2.4_aarch64.dmg`**.
3. Wait for the download to finish. It'll land in your **Downloads** folder.

## Step 2 — Open the installer

1. Double-click the `.dmg` file in Downloads.
2. A Finder window opens with two icons: **ParseKit** and **Applications**, plus a frosted background with an arrow between them.
3. **Drag the ParseKit icon onto the Applications folder icon.** Don't drop it on the Desktop. Don't double-click ParseKit inside the DMG window.

```mermaid
flowchart LR
  DMG["Open the .dmg"] --> DRAG["Drag ParseKit → Applications"]
  DRAG --> EJECT["Eject the DMG"]
  EJECT --> OPEN["Open from Applications"]
```

4. Eject the DMG (right-click it on the Desktop → **Eject**, or drag it to Trash in the sidebar).
5. If ParseKit was running from the DMG, quit it first (menu bar icon → Quit).

## Step 3 — First launch (the annoying part)

ParseKit isn't notarized with Apple yet. macOS Gatekeeper will probably block it once. You only need to do this once.

### Option A — Right-click Open (easiest)

1. Open **Finder → Applications**.
2. Find **ParseKit**.
3. **Right-click** (or Control-click) → **Open**.
4. Click **Open** in the dialog that says the developer can't be verified.
5. ParseKit starts. Look for its icon in the **menu bar** at the top-right of your screen.

### Option B — System Settings

1. Try opening ParseKit normally. macOS blocks it.
2. Open **System Settings → Privacy & Security**.
3. Scroll down. You should see a message about ParseKit being blocked. Click **Open Anyway**.
4. Confirm when prompted.

### Option C — Terminal one-liner

If Finder is being weird after the drag-and-drop, paste this into **Terminal** (Applications → Utilities → Terminal):

```bash
xattr -cr /Applications/ParseKit.app
xattr -d com.apple.FinderInfo /Applications/ParseKit.app 2>/dev/null || true
```

Then try Option A again. ParseKit also has a **Copy fix command** button under Settings if you need this later.

## Step 4 — Find ParseKit

ParseKit is a **menu bar app**. It does not appear in the Dock.

Look at the top-right of your screen, near the Wi-Fi and battery icons. ParseKit's icon should be there.

If you don't see it:

- Click the **`›`** or **`˅`** chevron on the left side of the menu bar. macOS hides overflow icons there.
- Make sure you're opening ParseKit from **Applications**, not from the DMG or Downloads.

Click the icon to open the panel.

## Optional — LibreOffice (for Word and PowerPoint)

PDFs work immediately. Word and PowerPoint files need LibreOffice, which is free:

1. Download from **[libreoffice.org](https://www.libreoffice.org/download/download-libreoffice/)**.
2. Install it the normal way (drag to Applications).
3. In ParseKit, open **Settings**. Under **Optional converters**, LibreOffice should show as installed. If not, hit **Recheck**.

## Optional — Finder Quick Action (right-click PDFs)

ParseKit can add a Finder shortcut. In the app: **Settings → Finder → Install Finder Quick Action**.

After that, right-click a file in Finder:

> **Quick Actions** → **Parse to Markdown with ParseKit**

**Important:** the menu item includes the word **ParseKit**. If you only see **"Convert to Markdown"** with no ParseKit name, that's a different tool — not this app.

**How it behaves:**
- If you've set an **output folder** in ParseKit → parses in the background, macOS notification when done
- If no output folder yet → ParseKit opens with the file ready to parse

Enable the action in **System Settings → Keyboard → Keyboard Shortcuts → Services** (or **Finder → Quick Actions**) if it doesn't show up.

## Optional — ImageMagick (for PNG, JPG, etc.)

If you parse images:

```bash
brew install imagemagick
```

Don't have Homebrew? Install it from **[brew.sh](https://brew.sh)**, then run the command above. Hit **Recheck** in ParseKit Settings.

## Updating

ParseKit checks for updates on launch. When a gold banner appears, click **Install & Restart**.

If that fails, download the latest DMG from [Releases](https://github.com/harshabala/parsekit/releases/latest) and repeat this guide. Your settings are stored separately and should carry over.

## Still stuck?

Open an issue at **[github.com/harshabala/parsekit/issues](https://github.com/harshabala/parsekit/issues)** and include:

- Your macOS version (Apple menu → About This Mac)
- Whether you're on Apple Silicon or Intel
- What step failed and any error message you saw