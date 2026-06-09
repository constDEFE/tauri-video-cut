#!/bin/bash
set -e

echo "=== [3/5] Compiling Custom FFmpeg tools ==="

if [ "$MSYSTEM" != "MINGW64" ]; then
  echo "ERROR: This script must be run inside an MSYS2 MINGW64 terminal shell"
  exit 1
fi

WORKSPACE_DIR=$(pwd)

FFMPEG_REPO_DIR="$WORKSPACE_DIR/ffmpeg-source"
FFMPEG_BUILD_DIR="$WORKSPACE_DIR/ffmpeg-dist"

if [ -z "$FFMPEG_BUILD_DIR" ] || [ "$FFMPEG_BUILD_DIR" == "/" ]; then
  echo "ERROR: Heavy safety constraint violation on build destination path"
  exit 1
fi

echo "Cleaning old FFmpeg binaries (preserving headers)..."
rm -rf "$FFMPEG_BUILD_DIR/bin"
rm -rf "$FFMPEG_BUILD_DIR/share"
mkdir -p "$FFMPEG_BUILD_DIR"

cd "$FFMPEG_REPO_DIR"
echo "Resetting FFmpeg repository compilation caches..."
make distclean || true

echo "Setting up local environment variables for Nvidia dependencies..."
export PKG_CONFIG_PATH="${FFMPEG_BUILD_DIR}/lib/pkgconfig:$PKG_CONFIG_PATH"

echo "Executing custom lean feature configuration..."
./configure \
  --extra-cflags="-I${FFMPEG_BUILD_DIR}/include" \
  --prefix="${FFMPEG_BUILD_DIR}" \
  --enable-shared \
  --disable-static \
  --disable-all \
  --disable-autodetect \
  --disable-doc \
  --disable-debug \
  \
  --enable-ffmpeg \
  --enable-ffprobe \
  \
  --enable-avcodec \
  --enable-avformat \
  --enable-avfilter \
  --enable-avutil \
  --enable-swscale \
  --enable-swresample \
  \
  --enable-ffnvcodec \
  --enable-nvenc \
  --enable-nvdec \
  --enable-cuvid \
	--enable-dxva2 \
  --enable-d3d11va \
  --enable-hwaccel=h264_d3d11va,hevc_d3d11va,vp9_d3d11va,av1_d3d11va,h264_dxva2,hevc_dxva2,vp9_dxva2,av1_dxva2,h264_nvdec,hevc_nvdec,vp9_nvdec,av1_nvdec \
  \
  --enable-gpl \
  --enable-libx264 \
  --enable-libx265 \
  --enable-libvpx \
  --enable-libsvtav1 \
  --enable-libdav1d \
  \
  --enable-decoder=libdav1d,av1,h264,hevc,vp9,aac,mp3,flac,opus,pcm_s16le,pcm_s24le,rawvideo,mpeg4,av1_cuvid,h264_cuvid,hevc_cuvid,vp9_cuvid \
  --enable-encoder=libx264,libx265,libsvtav1,libvpx_vp9,aac,flac,opus,pcm_s16le,pcm_s24le,av1_nvenc,h264_nvenc,hevc_nvenc \
  --enable-parser=av1,h264,hevc,aac,mjpeg,vp9,opus,mpeg4 \
  --enable-demuxer=mov,matroska,webm,concat,image2,aac,mp3,opus,rawvideo \
  --enable-muxer=mp4,matroska,webm,mov,null \
  --enable-protocol=file,pipe \
  --enable-filter=scale,trim,atrim,null,anull,fps,format,aresample \
  --enable-bsf=h264_mp4toannexb,hevc_mp4toannexb,aac_adtstoasc,av1_frame_merge,extract_extradata,vp9_superframe,vp9_superframe_split \
  --enable-asm \
  --enable-x86asm

echo "Compiling FFmpeg binaries..."
make -j$(nproc)
make install

echo "=== Build Complete. Staged assets located inside: $FFMPEG_BUILD_DIR/bin ==="