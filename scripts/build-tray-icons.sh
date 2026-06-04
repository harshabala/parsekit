#!/usr/bin/env bash
# Build macOS template tray icons: solid black glyph on transparent, standard 22/44pt canvas.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TRAY="$ROOT/src-tauri/icons/tray"

command -v magick >/dev/null || { echo "error: ImageMagick (magick) required" >&2; exit 1; }

mkdir -p "$TRAY"

# Four bold parse-line bands on a full menu-bar canvas (larger = easier to click).
draw_glyph() {
  local size="$1"
  local out="$2"
  local m=3
  local w=$((size - m))
  local h=$((size >= 40 ? 5 : 4))
  local gap=$((size >= 40 ? 3 : 2))
  local y1=$m
  local y2=$((y1 + h + gap))
  local y3=$((y2 + h + gap))
  local y4=$((y3 + h + gap))
  magick -size "${size}x${size}" xc:none \
    -fill '#000000' \
    -draw "roundrectangle ${m},${y1} ${w},$((y1 + h)) 2,2" \
    -draw "roundrectangle ${m},${y2} ${w},$((y2 + h)) 2,2" \
    -draw "roundrectangle ${m},${y3} ${w},$((y3 + h)) 2,2" \
    -draw "roundrectangle ${m},${y4} ${w},$((y4 + h)) 2,2" \
    PNG32:"$out"
}

# Standard macOS status-item sizes (logical 22pt + @2x).
draw_glyph 22 "$TRAY/icon.png"
draw_glyph 44 "$TRAY/icon@2x.png"

echo "Wrote $TRAY/icon.png (22) and $TRAY/icon@2x.png (44)"

sips -g hasAlpha "$TRAY/icon.png" "$TRAY/icon@2x.png" | grep hasAlpha

python3 << 'PY'
from struct import unpack
import zlib
from pathlib import Path

def check(path: Path) -> None:
    data = path.read_bytes()
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
    raw = zlib.decompress(idat)
    stride = w * bpp + 1

    def alpha_at(x: int, y: int) -> int:
        off = y * stride + 1 + x * bpp
        return raw[off + 3]

    corners = [(0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1)]
    alphas = [alpha_at(x, y) for x, y in corners]
    visible = sum(1 for y in range(h) for x in range(w) if alpha_at(x, y) > 128)
    print(f"{path.name} {w}x{h} corner alpha: {alphas} visible pixels: {visible}")
    if visible < 40:
        raise SystemExit(f"{path}: glyph too faint ({visible} visible pixels)")
    if any(a != 0 for a in alphas):
        raise SystemExit(f"{path}: corner pixels must be transparent (alpha=0)")

print("Alpha checks: OK")
PY

identify "$TRAY/icon.png" "$TRAY/icon@2x.png"