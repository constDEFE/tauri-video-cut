#!/usr/bin/env bash
# SPDX-License-Idenifier: Apache-2.0
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BUILD_DIR="${BUILD_DIR:-$SCRIPT_DIR/build}"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUT="$REPO_ROOT/legal/generated"

mkdir -p "$OUT/ffmpeg" "$OUT/mpv"

# --- FFmpeg ---
if [[ -d "$BUILD_DIR/ffmpeg-source" ]]; then
  cp "$BUILD_DIR/ffmpeg-source/ffbuild/config.log" "$OUT/ffmpeg/configure.log" 2>/dev/null || true
  cp "$BUILD_DIR/ffmpeg-source/ffbuild/config.mak" "$OUT/ffmpeg/" 2>/dev/null || true
  cp "$BUILD_DIR/ffmpeg-source/COPYING.GPLv2"      "$OUT/ffmpeg/" 2>/dev/null || true
fi
if [[ -x "$BUILD_DIR/ffmpeg-dist/bin/ffmpeg.exe" ]]; then
  "$BUILD_DIR/ffmpeg-dist/bin/ffmpeg.exe" -version > "$OUT/ffmpeg/ffmpeg-version.txt" 2>/dev/null || true
fi
cp "$BUILD_DIR/2-build-ff.sh" "$OUT/ffmpeg/build-script.sh" 2>/dev/null || true

# --- mpv ---
if [[ -d "$BUILD_DIR/mpv-tmp" ]]; then
  cp "$BUILD_DIR/mpv-tmp/meson-logs/meson-log.txt" "$OUT/mpv/" 2>/dev/null || true
fi
if [[ -d "$BUILD_DIR/mpv-source" ]]; then
  for f in Copyright LICENSE LICENSE.* COPYING*; do
    [[ -f "$BUILD_DIR/mpv-source/$f" ]] && cp -f "$BUILD_DIR/mpv-source/$f" "$OUT/mpv/"
  done
fi
cp "$BUILD_DIR/3-build-mpv.sh" "$OUT/mpv/build-script.sh" 2>/dev/null || true

# --- Build environment ---
{
  echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  uname -a
  gcc --version 2>/dev/null | head -1 || true
} > "$OUT/build-env.txt"

echo "Done → $OUT"
