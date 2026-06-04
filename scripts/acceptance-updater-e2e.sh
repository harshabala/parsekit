#!/usr/bin/env bash
# Acceptance: (A) codesign verify published tar.gz  (B) in-app Install & Restart → 0.2.2
set -euo pipefail

VERSION="${1:-0.2.2}"
TAG="v${VERSION}"
REPO="harshabala/parsedock"
TARBALL="ParseKit_${VERSION}_aarch64.app.tar.gz"
URL="https://github.com/${REPO}/releases/download/${TAG}/${TARBALL}"
WORKDIR="$(mktemp -d)"
trap 'rm -rf "$WORKDIR"' EXIT

echo "=== A: codesign verify published ${TARBALL} ==="
curl -fsSL -o "$WORKDIR/$TARBALL" "$URL"
tar -xzf "$WORKDIR/$TARBALL" -C "$WORKDIR"
echo "--- codesign --verify --deep --strict ---"
codesign --verify --deep --strict "$WORKDIR/ParseKit.app"
echo "codesign verify: exit 0"
codesign -dv --verbose=2 "$WORKDIR/ParseKit.app" 2>&1 | head -12

echo ""
echo "=== B: E2E Install & Restart (gold banner) ==="
bash "$(dirname "$0")/kill-parsekit.sh" 2>/dev/null || true

# Ensure an older build with updater is in /Applications (0.2.0 or 0.2.1).
INSTALLED_VER="$(defaults read /Applications/ParseKit.app/Contents/Info CFBundleShortVersionString 2>/dev/null || echo none)"
echo "Installed /Applications version: ${INSTALLED_VER}"
if [[ "$INSTALLED_VER" == "$VERSION" ]]; then
  echo "error: /Applications already at ${VERSION}; install 0.2.0 or 0.2.1 DMG first" >&2
  exit 1
fi

open -a /Applications/ParseKit.app
echo "Waiting for background update check (12s) ..."
sleep 12

CLICKED=0
if osascript <<'APPLESCRIPT' 2>/dev/null; then
  tell application "ParseKit" to activate
  delay 0.5
  tell application "System Events"
    tell process "ParseKit"
      set frontmost to true
      repeat with w in windows
        repeat with b in buttons of w
          if name of b contains "Install" then
            click b
            exit repeat
          end if
        end repeat
      end repeat
    end tell
  end tell
APPLESCRIPT
  CLICKED=1
  echo "Clicked Install & Restart via AppleScript"
fi

if [[ "$CLICKED" -eq 0 ]]; then
  echo "AppleScript could not find Install button (menu-bar webview). Trying AX tree dump ..."
  osascript -e 'tell application "System Events" to tell process "ParseKit" to get name of every window' 2>/dev/null || true
  echo "Open the tray popover manually if needed; retrying button search for 30s ..."
  for _ in $(seq 1 15); do
    sleep 2
    if osascript <<'APPLESCRIPT' 2>/dev/null; then
      tell application "System Events"
        tell process "ParseKit"
          repeat with w in windows
            repeat with b in buttons of w
              if name of b contains "Install" then
                click b
                return "ok"
              end if
            end repeat
          end repeat
        end tell
      end tell
APPLESCRIPT
      CLICKED=1
      echo "Clicked Install & Restart"
      break
    fi
  done
fi

if [[ "$CLICKED" -eq 0 ]]; then
  echo "error: could not automate Install & Restart — open tray, click Install & Restart, then re-run checks below" >&2
  exit 1
fi

echo "Waiting for download, install, restart (90s max) ..."
for _ in $(seq 1 45); do
  sleep 2
  if pgrep -xq parsekit 2>/dev/null || pgrep -xq ParseKit 2>/dev/null; then
    NEW_VER="$(defaults read /Applications/ParseKit.app/Contents/Info CFBundleShortVersionString 2>/dev/null || echo unknown)"
    if [[ "$NEW_VER" == "$VERSION" ]]; then
      echo "pgrep: $(pgrep -l parsekit 2>/dev/null || pgrep -l ParseKit)"
      echo "CFBundleShortVersionString: ${NEW_VER}"
      exit 0
    fi
  fi
done

NEW_VER="$(defaults read /Applications/ParseKit.app/Contents/Info CFBundleShortVersionString 2>/dev/null || echo unknown)"
echo "pgrep: $(pgrep -l parsekit 2>/dev/null || pgrep -l ParseKit || echo none)"
echo "CFBundleShortVersionString: ${NEW_VER}"
if [[ "$NEW_VER" == "$VERSION" ]] && pgrep -xq parsekit 2>/dev/null; then
  exit 0
fi
echo "error: E2E failed — expected ${VERSION} running" >&2
exit 1