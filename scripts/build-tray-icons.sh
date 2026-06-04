#!/usr/bin/env bash
# Build macOS template tray icons: solid black glyph, fully transparent background.
# Uses ImageMagick draw (reliable) — menubar PNGs often lack a real alpha mask at 18pt.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TRAY="$ROOT/src-tauri/icons/tray"

command -v magick >/dev/null || { echo "error: ImageMagick (magick) required" >&2; exit 1; }

mkdir -p "$TRAY"

# Four bold parse-line bands — solid #000 on transparent for NSStatusItem template tinting.
draw_glyph() {
  local size="$1"
  local out="$2"
  local m=2
  local w=$((size - m))
  local h=3
  if [[ "$size" -ge 32 ]]; then
    h=4
    m=3
    w=$((size - m))
  fi
  local y1=$((m + 1))
  local y2=$((y1 + h + 2))
  local y3=$((y2 + h + 2))
  local y4=$((y3 + h + 2))
  magick -size "${size}x${size}" xc:none \
    -fill '#000000' \
    -draw "roundrectangle ${m},${y1} ${w},$((y1 + h)) 1,1" \
    -draw "roundrectangle ${m},${y2} ${w},$((y2 + h)) 1,1" \
    -draw "roundrectangle ${m},${y3} ${w},$((y3 + h)) 1,1" \
    -draw "roundrectangle ${m},${y4} ${w},$((y4 + h)) 1,1" \
    PNG32:"$out"
}

draw_glyph 18 "$TRAY/icon.png"
draw_glyph 36 "$TRAY/icon@2x.png"

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
    visible = sum(1 for y in range(h) for x in range(w) if alpha_at(x, y) > 128)
    print(f"{path.name} {w}x{h} corner alpha: {alphas} visible pixels: {visible}")
    if visible < 20:
        raise SystemExit(f"{path}: glyph too faint ({visible} visible pixels)")
    if any(a != 0 for a in alphas):
        raise SystemExit(f"{path}: corner pixels must be transparent (alpha=0)")

print("Alpha checks: OK")
PY

identify "$TRAY/icon.png" "$TRAY/icon@2x.png"