#!/usr/bin/env bash
# Create ParseKit_<version>_aarch64.app.tar.gz from a post-postbuild signed .app.
# Fails if codesign --verify does not pass on the source app or extracted copy.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION="$(node -p "require('$ROOT/package.json').version")"
APP="${1:-$ROOT/src-tauri/target/release/bundle/macos/ParseKit.app}"
OUT="${2:-$ROOT/src-tauri/target/release/bundle/dmg/ParseKit_${VERSION}_aarch64.app.tar.gz}"

if [[ ! -d "$APP" ]]; then
  echo "error: app bundle not found: $APP" >&2
  exit 1
fi

echo "== create-updater-tarball: $(basename "$APP") → $(basename "$OUT") =="

echo "[1/5] Verify signed .app in bundle/macos (strict) ..."
xattr -cr "$APP" 2>/dev/null || true
codesign --verify --deep --strict --verbose=2 "$APP"

echo "[2/5] Remove stale Tauri auto-generated updater archives ..."
MACOS_DIR="$(dirname "$APP")"
rm -f "$MACOS_DIR"/*.app.tar.gz "$MACOS_DIR"/*.app.tar.gz.sig

echo "[3/5] Stage clean copy + re-seal (avoids FinderInfo in tarball) ..."
mkdir -p "$(dirname "$OUT")"
rm -f "$OUT"
TAR_STAGE="$(mktemp -d)"
VERIFY_DIR=""
cleanup() { rm -rf "$TAR_STAGE" "$VERIFY_DIR"; }
trap cleanup EXIT

ditto --norsrc "$APP" "$TAR_STAGE/ParseKit.app"
xattr -cr "$TAR_STAGE/ParseKit.app" 2>/dev/null || true
xattr -d com.apple.FinderInfo "$TAR_STAGE/ParseKit.app" 2>/dev/null || true
codesign --force --deep --sign - "$TAR_STAGE/ParseKit.app"
codesign --verify --deep --strict --verbose=2 "$TAR_STAGE/ParseKit.app"

echo "[4/5] tar.gz (COPYFILE_DISABLE, no mac metadata) ..."
COPYFILE_DISABLE=1 tar --no-mac-metadata -czf "$OUT" -C "$TAR_STAGE" ParseKit.app 2>/dev/null \
  || COPYFILE_DISABLE=1 tar -czf "$OUT" -C "$TAR_STAGE" ParseKit.app

echo "[5/5] Verify extracted .app from tarball (no post-extract xattr strip) ..."
VERIFY_DIR="$(mktemp -d)"
tar -xzf "$OUT" -C "$VERIFY_DIR"
codesign --verify --deep --strict --verbose=2 "$VERIFY_DIR/ParseKit.app"
codesign -dv --verbose=2 "$VERIFY_DIR/ParseKit.app" 2>&1 | head -8

echo "create-updater-tarball: OK → $OUT"