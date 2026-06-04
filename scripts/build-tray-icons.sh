#!/usr/bin/env bash
# Build macOS template tray icons from menubar-icon.png (black glyph, transparent background).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SOURCE="$ROOT/assets/branding/menubar-icon.png"
TRAY="$ROOT/src-tauri/icons/tray"

command -v magick >/dev/null || { echo "error: ImageMagick (magick) required" >&2; exit 1; }

if [[ ! -f "$SOURCE" ]]; then
  echo "error: $SOURCE not found" >&2
  exit 1
fi

mkdir -p "$TRAY"

# Pad to square, resize to menu-bar sizes; force black glyph on transparent alpha.
for spec in "36:icon@2x.png" "18:icon.png"; do
  size="${spec%%:*}"
  out="${spec##*:}"
  magick "$SOURCE" \
    -alpha on \
    -background none \
    -gravity center \
    -extent "${size}x${size}" \
    -resize "${size}x${size}!" \
    -colorspace Gray \
    -negate \
    -channel RGB \
    -fill black \
    -colorize 100 \
    -channel A \
    +channel \
    PNG32:"$TRAY/$out"
done

echo "Wrote $TRAY/icon.png (18) and $TRAY/icon@2x.png (36)"

# --- Verification (required) ---
sips -g hasAlpha "$TRAY/icon.png" "$TRAY/icon@2x.png" | grep hasAlpha

python3 << 'PY'
from struct import unpack
import zlib
from pathlib import Path

def check(path: Path) -> None:
    data = path.read_bytes()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise SystemExit(f"{path}: not PNG")
    w = h = 0
    idat = b""
    i = 8
    while i < len(data):
        ln = unpack(">I", data[i : i + 4])[0]
        typ = data[i + 4 : i + 8]
        chunk = data[i + 8 : i + 8 + ln]
        i += 12 + ln
        if typ == b"IHDR":
            w, h = unpack(">II", chunk[:8])
            ctype = chunk[9]
        elif typ == b"IDAT":
            idat += chunk
        elif typ == b"IEND":
            break
    bpp = {6: 4, 2: 3, 3: 1, 4: 2}.get(ctype, 4)
    if bpp != 4:
        raise SystemExit(f"{path}: expected RGBA, got ctype {ctype}")
    raw = zlib.decompress(idat)
    stride = w * bpp + 1

    def alpha_at(x: int, y: int) -> int:
        off = y * stride + 1 + x * bpp
        return raw[off + 3]

    corners = [(0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1)]
    alphas = [alpha_at(x, y) for x, y in corners]
    print(f"{path.name} {w}x{h} corner alpha: {alphas}")
    if any(a != 0 for a in alphas):
        raise SystemExit(f"{path}: corner pixels must be transparent (alpha=0)")

print("Alpha checks: OK")
PY

identify "$TRAY/icon.png" "$TRAY/icon@2x.png"