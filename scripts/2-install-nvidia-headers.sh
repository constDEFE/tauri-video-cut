#!/bin/bash
set -e

echo "=== [2/5] Installing NVidia codec headers ==="

if [ "$MSYSTEM" != "MINGW64" ]; then
  echo "ERROR: This script must be run inside an MSYS2 MINGW64 terminal shell"
  exit 1
fi

WORKSPACE_DIR=$(pwd)
NV_CODEC_HEADERS_REPO_DIR="$WORKSPACE_DIR/nvidia-codec-headers-source"
FFMPEG_BUILD_DIR="$WORKSPACE_DIR/ffmpeg-dist"

cd "$NV_CODEC_HEADERS_REPO_DIR"
echo "Installing codec headers..."
make PREFIX="${FFMPEG_BUILD_DIR}"
make install PREFIX="${FFMPEG_BUILD_DIR}"

echo "=== Installation complete ==="