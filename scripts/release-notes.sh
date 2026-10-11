#!/usr/bin/env bash
# Print GitHub release notes for the given version (or package.json version).
# Usage: bash scripts/release-notes.sh [version]
# If RELEASE_NOTES env is set and non-empty, prints that instead (CI override).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION="${1:-$(node -p "require('$ROOT/package.json').version")}"
VERSION="${VERSION#v}"

if [[ -n "${RELEASE_NOTES:-}" ]]; then
  printf '%s\n' "$RELEASE_NOTES"
  exit 0
fi

# Prefer a versioned notes file when present.
NOTES_FILE="$ROOT/docs/releases/v${VERSION}.md"
if [[ -f "$NOTES_FILE" ]]; then
  cat "$NOTES_FILE"
  exit 0
fi

# Fallback: friendly default (never dump raw commit subjects as the whole body).
cat <<EOF
## ParseKit v${VERSION}

Menu-bar app that turns documents into clean Markdown — **on your Mac**, offline.

### Install

1. Download **\`ParseKit_${VERSION}_aarch64.dmg\`** below (Apple Silicon, macOS 12+)
2. Open the DMG → drag **ParseKit** to **Applications**
3. Open from Applications → icon appears in the **menu bar**

First launch blocked by macOS? Run once:

\`\`\`bash
xattr -cr /Applications/ParseKit.app
\`\`\`

Full guide: [docs/INSTALL.md](https://github.com/harshabala/parsekit/blob/master/docs/INSTALL.md)

### What's included

- Local PDF / Office / image → Markdown, plain text, or JSON
- OCR for scans · Finder Quick Action · global hotkey
- In-app updates (Install & Restart when a banner appears)

### Assets

| File | Use |
| --- | --- |
| \`ParseKit_${VERSION}_aarch64.dmg\` | **New install** — drag to Applications |
| \`ParseKit_${VERSION}_aarch64.app.tar.gz\` | Auto-updater (you don't need this manually) |
| \`parsekit-latest.json\` | Updater manifest |

**Requirements:** Apple Silicon Mac. PDF works immediately; Word/PowerPoint need LibreOffice; images need ImageMagick.

Questions or bugs → [open an issue](https://github.com/harshabala/parsekit/issues)
EOF
