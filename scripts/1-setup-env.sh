#!/bin/bash
set -e

echo "=== [1/5] Initializing MSYS2 MINGW64 Environment ==="

if [ "$MSYSTEM" != "MINGW64" ]; then
  echo "ERROR: This script must be run inside an MSYS2 MINGW64 terminal shell!"
  exit 1
fi

echo "Updating pacman repositories..."
pacman -Sy --noconfirm

echo "Installing build tools (GCC, Make, Meson, Ninja)..."
pacman -S --needed --noconfirm \
  mingw-w64-x86_64-toolchain \
  git \
  make \
  nasm \
  yasm \
  pkg-config \
  mingw-w64-x86_64-meson \
  mingw-w64-x86_64-ninja

echo "Installing codec and player dependencies..."
pacman -S --needed --noconfirm \
  mingw-w64-x86_64-ffmpeg \
  mingw-w64-x86_64-libass \
  mingw-w64-x86_64-dav1d \
  mingw-w64-x86_64-libdovi \
  mingw-w64-x86_64-fontconfig \
  mingw-w64-x86_64-freetype \
  mingw-w64-x86_64-fribidi \
  mingw-w64-x86_64-glib2 \
  mingw-w64-x86_64-graphite2 \
  mingw-w64-x86_64-harfbuzz \
  mingw-w64-x86_64-libiconv \
  mingw-w64-x86_64-libjpeg-turbo \
  mingw-w64-x86_64-lcms2 \
  mingw-w64-x86_64-libplacebo \
  mingw-w64-x86_64-libpng \
  mingw-w64-x86_64-shaderc \
  mingw-w64-x86_64-spirv-cross \
  mingw-w64-x86_64-gcc \
  mingw-w64-x86_64-bzip2 \
  mingw-w64-x86_64-brotli \
  mingw-w64-x86_64-expat \
  mingw-w64-x86_64-gettext \
  mingw-w64-x86_64-pcre2 \
  mingw-w64-x86_64-zimg \
  mingw-w64-x86_64-svt-av1 \
  mingw-w64-x86_64-libunibreak \
  mingw-w64-x86_64-libva \
  mingw-w64-x86_64-libvpx \
  mingw-w64-x86_64-x264 \
  mingw-w64-x86_64-x265 \

WORKSPACE_DIR=$(pwd)

if [ ! -d "$WORKSPACE_DIR/ffmpeg-source" ]; then
  echo "Cloning stable FFmpeg release branch (release/7.1)..."
  git clone -b "release/7.1" --single-branch https://github.com/FFmpeg/FFmpeg.git "$WORKSPACE_DIR/ffmpeg-source"
else
  echo "FFmpeg source repository already exists. Skipping."
fi

if [ ! -d "$WORKSPACE_DIR/nvidia-codec-headers-source" ]; then
  echo "Cloning NVidia codec headers repository (n12.2.72.0)..."
  git clone -b "n12.2.72.0" --single-branch https://github.com/FFmpeg/nv-codec-headers.git "$WORKSPACE_DIR/nvidia-codec-headers-source"
else
  echo "NVidia codec headers source repository already exists. Skipping."
fi

if [ ! -d "$WORKSPACE_DIR/mpv-source" ]; then
  echo "Cloning mpv player source repository..."
  git clone https://github.com/mpv-player/mpv.git "$WORKSPACE_DIR/mpv-source"
else
  echo "mpv source repository already exists. Skipping."
fi

echo "=== Environment Setup Complete. You are ready to build. ==="