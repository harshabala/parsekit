#!/usr/bin/env bash
# Square template tray icons from assets/branding/menubar-icon-src.png
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/assets/branding/menubar-icon-src.png"
TRAY="$ROOT/src-tauri/icons/tray"
WORK="$(mktemp /tmp/parsedock-tray-square.XXXXXX.png)"

command -v magick >/dev/null || { echo "error: ImageMagick (magick) required" >&2; exit 1; }

mkdir -p "$TRAY"

magick "$SRC" -trim +repage -background none -gravity center \
  -extent '%[fx:max(w,h)*1.24]x%[fx:max(w,h)*1.24]' "$WORK"

magick "$WORK" -resize 22x22! PNG32:"$TRAY/icon.png"
magick "$WORK" -resize 44x44! PNG32:"$TRAY/icon@2x.png"

rm -f "$WORK"
echo "Tray icons written: $TRAY/icon.png (22) $TRAY/icon@2x.png (44)"
identify "$TRAY/icon.png" "$TRAY/icon@2x.png"