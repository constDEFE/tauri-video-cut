#!/bin/bash
set -e

echo "=== [3/5] Compiling Custom FFmpeg tools ==="

if [ "$MSYSTEM" != "UCRT64" ]; then
  echo "ERROR: This script must be run inside an MSYS2 UCRT64 terminal shell"
  exit 1
fi

WORKSPACE_DIR=$(pwd)
FFMPEG_REPO_DIR="$WORKSPACE_DIR/ffmpeg-source"
FFMPEG_BUILD_DIR="$WORKSPACE_DIR/ffmpeg-dist"

if [ -z "$FFMPEG_BUILD_DIR" ] || [ "$FFMPEG_BUILD_DIR" == "/" ]; then
  echo "ERROR: Heavy safety constraint violation on build destination path"
  exit 1
fi

echo "Cleaning old FFmpeg binaries (preserving headers and Nvidia pkgconfig files)..."
rm -rf "$FFMPEG_BUILD_DIR/bin"
rm -rf "$FFMPEG_BUILD_DIR/share"
mkdir -p "$FFMPEG_BUILD_DIR"

cd "$FFMPEG_REPO_DIR"
echo "Resetting FFmpeg repository compilation caches..."
make distclean || true

echo "Setting up local environment variables for Nvidia dependencies..."
export PKG_CONFIG_PATH="${FFMPEG_BUILD_DIR}/lib/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"

# `--cpu=x86-64-v3` AND `--enable-lto`
# AVX2 support. Better performance on modern CPUs,
# on older (~2015) will instantly crash.
# Use with `--disable-runtime-cpudetect`
echo "Executing custom lean feature configuration..."

./configure \
  --extra-cflags="-I${FFMPEG_BUILD_DIR}/include -O3 -pipe" \
  --prefix="${FFMPEG_BUILD_DIR}" \
  --enable-shared \
  --disable-static \
  --disable-all \
  --disable-autodetect \
  --disable-doc \
  --disable-debug \
  --disable-network \
  --disable-avdevice \
  --disable-iconv \
  --disable-dwt \
  --disable-faan \
  --disable-lsp \
  --disable-pixelutils \
  --disable-iamf \
  --disable-swscale-alpha \
  --cpu=x86-64-v3 \
  --disable-runtime-cpudetect \
  --enable-lto \
  --enable-pic \
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
  --enable-zlib \
  \
  --enable-ffnvcodec \
  --enable-nvenc \
  \
  --enable-hardcoded-tables \
  --enable-gpl \
  --enable-libx264 \
  --enable-libx265 \
  --enable-libvpx \
  --enable-libsvtav1 \
  --enable-libdav1d \
  \
  --enable-decoder=libdav1d,av1,h264,hevc,vp9,aac,ac3,eac3,mp3,opus,flac,alac,pcm_s16le,pcm_s24le,pcm_s32le,pcm_f32le,vorbis,rawvideo,mpeg4,mpeg2video,mjpeg \
  --enable-encoder=libx264,libx265,libsvtav1,libvpx_vp9,av1_nvenc,h264_nvenc,hevc_nvenc,pcm_s16le,pcm_s24le,pcm_s32le,pcm_f32le,aac,opus \
  --enable-parser=av1,h264,hevc,aac,mjpeg,vp9,opus,mpeg4,ac3,mpegaudio,flac,vorbis,mpegvideo \
  --enable-demuxer=mov,matroska,avi,mpegts,flv,ogg,wav,concat,pcm_s8,pcm_s16le \
  --enable-muxer=mp4,matroska,webm,avi,mov,pcm_s8,pcm_s16le \
  --enable-protocol=file,pipe \
  --enable-filter=trim,atrim,setpts,asetpts,format,aformat,aresample,null,anull \
  --enable-bsf=h264_mp4toannexb,hevc_mp4toannexb,aac_adtstoasc,av1_frame_merge,extract_extradata,vp9_superframe,vp9_superframe_split,mpeg4_unpack_bframes,aac_adtstoasc,opus_metadata \
  --enable-asm \
  --enable-x86asm

echo "Compiling FFmpeg binaries..."
make -j$(nproc)
make install

echo "=== Build Complete. Staged assets located inside: $FFMPEG_BUILD_DIR/bin ==="
