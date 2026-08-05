#!/bin/bash
set -e

echo "=== [4/5] Compiling Custom mpv Player ==="

if [ "$MSYSTEM" != "UCRT64" ]; then
  echo "ERROR: This script must be run inside an MSYS2 UCRT64 terminal shell"
  exit 1
fi

WORKSPACE_DIR=$(pwd)
FFMPEG_BUILD_DIR="$WORKSPACE_DIR/ffmpeg-dist"
MPV_REPO_DIR="$WORKSPACE_DIR/mpv-source"
MPV_TEMP_DIR="$WORKSPACE_DIR/mpv-tmp"
MPV_BUILD_DIR="$WORKSPACE_DIR/mpv-dist"

echo "Wiping old build artifacts..."
rm -rf "$MPV_TEMP_DIR"

export PKG_CONFIG_PATH="$FFMPEG_BUILD_DIR/lib/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"

cd "$MPV_REPO_DIR"

# see `./mpv-source/meson.options`
echo "Configuring ultra-lean mpv engine layout..."
meson setup "$MPV_TEMP_DIR" \
  --prefix="$MPV_BUILD_DIR" \
  -Dlibmpv=true \
  -Ddefault_library=shared \
  -Dcplayer=false \
  -Dbuild-date=false \
  -Dtests=false \
  -Dfuzzers=false \
  \
  -Dd3d11=enabled \
  -Dshaderc=enabled \
  -Dspirv-cross=enabled \
  -Ddirect3d=disabled \
  -Dgl=disabled \
  -Dgl-win32=disabled \
  -Dgl-dxinterop=disabled \
  -Dplain-gl=disabled \
  -Degl-angle=disabled \
  -Degl-angle-lib=disabled \
  -Degl-angle-win32=disabled \
  -Dvulkan=disabled \
  \
  -Dd3d-hwaccel=enabled \
  -Dd3d9-hwaccel=enabled \
  -Dcuda-hwaccel=disabled \
  -Dcuda-interop=disabled \
  -Dvaapi=disabled \
  -Dvdpau=disabled \
  \
  -Dlua=disabled \
  -Djavascript=disabled \
  -Dlibarchive=disabled \
  -Diconv=disabled \
  -Duchardet=disabled \
  -Djpeg=disabled \
  -Dlcms2=disabled \
  -Dzimg=disabled \
  -Dcplugins=disabled \
  -Ddvbin=disabled \
  -Dcdda=disabled \
  -Ddvdnav=disabled \
  -Dlibbluray=disabled \
  -Dvapoursynth=disabled \
  -Drubberband=disabled \
  -Dopenal=disabled \
  \
  -Dwasapi=enabled \
  -Dalsa=disabled \
  -Dpulse=disabled \
  -Djack=disabled \
  -Dpipewire=disabled \
  -Doss-audio=disabled \
  -Dsndio=disabled \
  -Dsdl2-audio=disabled \
  -Dsdl2-video=disabled \
  -Dsdl2-gamepad=disabled \
  \
  -Dcaca=disabled \
  -Dwayland=disabled \
  -Dx11=disabled \
  -Dxv=disabled \
  -Ddrm=disabled \
  -Dgbm=disabled \
  -Dsixel=disabled \
  \
  -Dwin32-smtc=disabled \
  -Dmanpage-build=disabled \
  -Dhtml-build=disabled \
  -Dpdf-build=disabled

echo "Compiling mpv binaries and shared library targets..."
meson compile -C "$MPV_TEMP_DIR"

echo "Installing mpv layers to workspace target..."
meson install -C "$MPV_TEMP_DIR"

echo "=== Build Complete. Staged assets located inside: $MPV_BUILD_DIR/bin ==="
