#!/bin/bash
set -e

echo "=== [4/5] Compiling Custom FFmpeg & mpv Player ==="

if [ "$MSYSTEM" != "MINGW64" ]; then
  echo "ERROR: This script must be run inside an MSYS2 MINGW64 terminal shell"
  exit 1
fi

WORKSPACE_DIR=$(pwd)
FFMPEG_BUILD_DIR="$WORKSPACE_DIR/ffmpeg-dist"
MPV_REPO_DIR="$WORKSPACE_DIR/mpv-source"
MPV_TEMP_DIR="$WORKSPACE_DIR/mpv-tmp"
MPV_BUILD_DIR="$WORKSPACE_DIR/mpv-dist"

echo "Wiping old build artifacts..."
rm -rf "$MPV_TEMP_DIR"

# Force pkg-config to read our lean custom build configuration manifests first
export PKG_CONFIG_PATH="$FFMPEG_BUILD_DIR/lib/pkgconfig:$PKG_CONFIG_PATH"

cd "$MPV_REPO_DIR"

# ======Settings Guide======
# Dlibmpv - emit libmpv.dll
# Ddefault_library - emit binaries so could be shared
# Dcplayer - emit exe file
#
# Dlua - lua scripting
# Djavascript - javascript scripting
# Dwayland - linux related
# Dvulkan - Vulcan support
# Dd3d11 - Direct3D11 support
#
# Dgl - OpenGL
# Dgl-win32 - OpenGL
# Dgl-dxinterop - OpenGL
# Dplain-gl - OpenGL
# Degl-angle-lib - OpenGL
# Degl-angle-win32 - OpenGL
#
# Dvapoursynth - filters
# Dlcms2 - accurate colors
# Drubberband - playback speed (0.5x-2x)
# Duchardet - subtitle encoding detection
# Dopenal - only need if `--ao=openal`
#
# Dlibbluray - blueray
# Ddvdnav - dvd
# Dcdda - cd
echo "Configuring ultra-lean mpv engine layout..."
meson setup "$MPV_TEMP_DIR" \
  --prefix="$MPV_BUILD_DIR" \
  -Dlibmpv=true \
  -Ddefault_library=shared \
  -Dcplayer=false \
  \
  -Dcaca=disabled \
  -Dd3d11=enabled \
  -Dlua=disabled \
  -Djavascript=disabled \
  \
  -Dgl=disabled \
  -Dgl-win32=disabled \
  -Dgl-dxinterop=disabled \
  -Dplain-gl=disabled \
  -Degl-angle-lib=disabled \
  -Degl-angle-win32=disabled \
  \
  -Dvapoursynth=disabled\
  -Drubberband=disabled \
  -Duchardet=disabled \
  -Dopenal=disabled \
  \
  -Ddvdnav=disabled \
  -Dlibbluray=disabled \
  -Dcdda=disabled \
  -Dwayland=disabled

echo "Compiling mpv binaries and shared library targets..."
meson compile -C "$MPV_TEMP_DIR"

echo "Installing mpv layers to workspace target..."
meson install -C "$MPV_TEMP_DIR"

echo "=== Build Complete. Staged assets located inside: $MPV_BUILD_DIR/bin ==="