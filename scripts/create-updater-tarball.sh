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

echo "[1/4] Verify signed .app (strict) ..."
xattr -cr "$APP" 2>/dev/null || true
codesign --verify --deep --strict --verbose=2 "$APP"

echo "[2/4] Remove stale Tauri auto-generated updater archives ..."
MACOS_DIR="$(dirname "$APP")"
rm -f "$MACOS_DIR"/*.app.tar.gz "$MACOS_DIR"/*.app.tar.gz.sig

echo "[3/4] tar.gz from signed bundle ..."
mkdir -p "$(dirname "$OUT")"
rm -f "$OUT"
(
  cd "$(dirname "$APP")"
  tar -czf "$OUT" "$(basename "$APP")"
)

echo "[4/4] Verify extracted .app from tarball ..."
VERIFY_DIR="$(mktemp -d)"
trap 'rm -rf "$VERIFY_DIR"' EXIT
tar -xzf "$OUT" -C "$VERIFY_DIR"
codesign --verify --deep --strict --verbose=2 "$VERIFY_DIR/ParseKit.app"
codesign -dv --verbose=2 "$VERIFY_DIR/ParseKit.app" 2>&1 | head -8

echo "create-updater-tarball: OK → $OUT"