#!/usr/bin/env bash
# Sign the release .app and rebuild the DMG so Gatekeeper + Finder installs work reliably.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP="$ROOT/src-tauri/target/release/bundle/macos/ParseDock.app"
DMG_OUT="$ROOT/src-tauri/target/release/bundle/dmg/ParseDock_0.1.3_aarch64.dmg"
VERSION="$(node -p "require('$ROOT/package.json').version")"
DMG_OUT="$ROOT/src-tauri/target/release/bundle/dmg/ParseDock_${VERSION}_aarch64.dmg"

if [[ ! -d "$APP" ]]; then
  echo "error: $APP not found — run npm run tauri build first" >&2
  exit 1
fi

echo "Cloning bundle without Finder metadata ..."
APP_CLEAN="${APP}.unsigned"
rm -rf "$APP_CLEAN"
ditto --norsrc "$APP" "$APP_CLEAN"
rm -rf "$APP"
mv "$APP_CLEAN" "$APP"
xattr -cr "$APP" 2>/dev/null || true

strip_macos_bin() {
  local src="$1"
  local tmp
  tmp="$(mktemp)"
  cat "$src" > "$tmp"
  chmod +x "$tmp"
  xattr -cr "$tmp" 2>/dev/null || true
  mv "$tmp" "$src"
}
strip_macos_bin "$APP/Contents/MacOS/parsedock-sidecar"
strip_macos_bin "$APP/Contents/MacOS/parsedock"

echo "Codesigning (best-effort; provenance xattrs can block ad-hoc sign on some systems) ..."
codesign --force --sign - "$APP/Contents/MacOS/parsedock-sidecar" 2>/dev/null || true
codesign --force --sign - "$APP/Contents/MacOS/parsedock" 2>/dev/null || true
echo "If Gatekeeper blocks launch: right-click ParseDock → Open once, or run: xattr -cr /Applications/ParseDock.app"

DMG_DIR="$(dirname "$DMG_OUT")"
mkdir -p "$DMG_DIR"
RM_DMG="$DMG_OUT"
[[ -f "$RM_DMG" ]] && rm -f "$RM_DMG"

echo "Creating DMG at $DMG_OUT ..."
hdiutil create \
  -volname "ParseDock" \
  -srcfolder "$APP" \
  -ov \
  -format UDZO \
  "$RM_DMG"

echo "Done: $RM_DMG"