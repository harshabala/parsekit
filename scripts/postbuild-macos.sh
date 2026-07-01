#!/usr/bin/env bash
# Sign the Tauri-built .app without rewriting Mach-O binaries (no cat/strip in place).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP="$ROOT/src-tauri/target/release/bundle/macos/ParseKit.app"
VERSION="$(node -p "require('$ROOT/package.json').version")"
DMG_DIR="$ROOT/src-tauri/target/release/bundle/dmg"
DMG_OUT="$DMG_DIR/ParseKit_${VERSION}_aarch64.dmg"

if [[ ! -d "$APP" ]]; then
  echo "error: $APP not found — run npm run tauri build first" >&2
  exit 1
fi

echo "== postbuild-macos: ParseKit v${VERSION} =="

echo "[1/6] Stage bundle outside target/ (avoids FinderInfo on bundle wrapper) ..."
STAGE_DIR="$(mktemp -d)"
STAGE_APP="$STAGE_DIR/ParseKit.app"
ditto --norsrc "$APP" "$STAGE_APP"

echo "[2/6] Strip extended attributes on staged bundle ..."
xattr -cr "$STAGE_APP" || true
xattr -d com.apple.FinderInfo "$STAGE_APP" 2>/dev/null || true
xattr -d 'com.apple.fileprovider.fpfs#P' "$STAGE_APP" 2>/dev/null || true

echo "[3/6] Ad-hoc sign staged bundle as a unit (--deep) ..."
codesign --force --deep --sign - "$STAGE_APP"

echo "[4/6] Verify signature (strict) ..."
codesign --verify --deep --strict --verbose=2 "$STAGE_APP"
codesign -dv --verbose=2 "$STAGE_APP" 2>&1

# Styled DMG (background art + drag-to-Applications layout via create-dmg).
if [[ -d "$DMG_DIR" ]]; then
  echo "[5/6] Build styled DMG (ParseKit installer window) ..."
  bash "$ROOT/scripts/dmg/build-dmg.sh" "$STAGE_APP"
  echo "DMG written: $DMG_OUT"
  DMG_MOUNT="$(mktemp -d /tmp/parsekit-dmg-verify.XXXXXX)"
  hdiutil attach -nobrowse -readonly -mountpoint "$DMG_MOUNT" "$DMG_OUT" >/dev/null
  codesign --verify --deep --strict --verbose=2 "$DMG_MOUNT/ParseKit.app"
  codesign -dv --verbose=2 "$DMG_MOUNT/ParseKit.app" 2>&1 | head -12
  hdiutil detach "$DMG_MOUNT" -quiet
  rmdir "$DMG_MOUNT"
fi

echo "[6/7] Install signed bundle into bundle/macos (mv, no Mach-O edits) ..."
rm -rf "$APP"
mv "$STAGE_APP" "$APP"
rmdir "$STAGE_DIR"

CLI_SRC="$ROOT/src-tauri/target/release/parsekit-cli"
CLI_DST="$APP/Contents/MacOS/parsekit-cli"
if [[ -f "$CLI_SRC" ]]; then
  echo "[7/7] Bundle CLI + optional /usr/local/bin symlink ..."
  cp "$CLI_SRC" "$CLI_DST"
  chmod +x "$CLI_DST"
  codesign --force --sign - "$CLI_DST" 2>/dev/null || true
  if [[ -w /usr/local/bin ]]; then
    ln -sf "$CLI_DST" /usr/local/bin/parsekit
    echo "Linked /usr/local/bin/parsekit -> $CLI_DST"
  else
    echo "note: /usr/local/bin not writable — skip CLI symlink (use $CLI_DST directly)"
  fi
else
  echo "[7/7] warn: $CLI_SRC missing — build with: cargo build --release --bin parsekit-cli" >&2
fi

echo "postbuild-macos: OK"