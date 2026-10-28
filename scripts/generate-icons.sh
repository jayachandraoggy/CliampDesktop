#!/usr/bin/env bash
# Regenerate Freedesktop hicolor PNGs from assets/icons/hertz.svg
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SVG="$ROOT/assets/icons/hertz.svg"
OUT="$ROOT/assets/icons/hicolor"
SIZES=(16 24 32 48 64 128 256 512)

for size in "${SIZES[@]}"; do
  dir="$OUT/${size}x${size}/apps"
  mkdir -p "$dir"
  dest="$dir/hertz.png"
  if command -v rsvg-convert >/dev/null 2>&1; then
    rsvg-convert -w "$size" -h "$size" "$SVG" -o "$dest"
  elif python3 -c "import cairosvg" 2>/dev/null; then
    python3 -c "import cairosvg; cairosvg.svg2png(url='$SVG', write_to='$dest', output_width=$size, output_height=$size)"
  else
    echo "Need rsvg-convert or Python cairosvg (pip install cairosvg)" >&2
    exit 1
  fi
  echo "Wrote $dest"
done
