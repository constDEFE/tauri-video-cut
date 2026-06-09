#!/bin/bash
set -e

echo "=== [5/5] Harvesting Binaries and System Runtime DLL Dependencies ==="

WORKSPACE_DIR=$(pwd)
STAGE_FF_DIR="$WORKSPACE_DIR/ffmpeg-dist/bin"
STAGE_MPV_DIR="$WORKSPACE_DIR/mpv-dist/bin"
STAGE_MPV_WRAPPER_DIR="$WORKSPACE_DIR/mpv-wrapper"
OUTPUT_DIR="$WORKSPACE_DIR/../src-tauri/lib"

rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"
mkdir -p "$STAGE_MPV_WRAPPER_DIR"

if [ ! -f "$STAGE_FF_DIR/ffmpeg.exe" ] || [ ! -f "$STAGE_MPV_DIR/libmpv-2.dll" ]; then
  echo "ERROR: Core processing binaries were not found. Run script 2 first."
  exit 1
fi

echo "Staging core executables and libmpv runtimes..."
cp "$STAGE_FF_DIR/ffmpeg.exe" "$OUTPUT_DIR/"
cp "$STAGE_FF_DIR/ffprobe.exe" "$OUTPUT_DIR/"

find "$STAGE_MPV_DIR" -maxdepth 1 -type f -name "*mpv*.dll" -exec cp {} "$OUTPUT_DIR/" \;

echo "Attempting to download libmpv-wrapper bridge automatically..."
WRAPPER_URL="https://github.com/nini22P/libmpv-wrapper/releases/latest/download/libmpv-wrapper-windows-x86_64.zip"

set +e
curl -L -f -o "$STAGE_MPV_WRAPPER_DIR/libmpv-wrapper.zip" "$WRAPPER_URL"
CURL_STATUS=$?
set -e

if [ $CURL_STATUS -ne 0 ]; then
  echo ""
  echo "========================================================================="
  echo "  WARNING: Automated download of libmpv-wrapper failed"
  echo "========================================================================="
  echo "Please download and stage it manually:"
  echo "1. Open this URL in your web browser:"
  echo "   ${WRAPPER_URL}"
  echo "2. Open the downloaded .zip file."
  echo "3. Extract or copy 'libmpv-wrapper.dll' directly into this directory:"
  echo "   ${STAGE_MPV_WRAPPER_DIR}"
  echo "========================================================================="
  echo ""
  read -p "Press [ENTER] once you have manually placed 'libmpv-wrapper.dll' in the folder to verify..."
else
  echo "Extracting wrapper bridge..."
  set +e
  unzip -o "$STAGE_MPV_WRAPPER_DIR/libmpv-wrapper.zip" -d "$STAGE_MPV_WRAPPER_DIR"
  UNZIP_STATUS=$?
  set -e
  rm -f "$STAGE_MPV_WRAPPER_DIR/libmpv-wrapper.zip"
  
  if [ $UNZIP_STATUS -ne 0 ]; then
    echo "========================================================================="
    echo "  WARNING: Extraction of the wrapper archive failed"
    echo "========================================================================="
    echo "Extract 'libmpv-wrapper.dll' manually from your downloads and"
    echo "place it directly inside: ${OUTPUT_DIR}"
    echo "========================================================================="
    read -p "Press [ENTER] once you have manually placed 'libmpv-wrapper.dll' to verify..."
  fi

  cp "$STAGE_MPV_WRAPPER_DIR/bin/libmpv-wrapper.dll" "$OUTPUT_DIR"
fi

if [ ! -f "$OUTPUT_DIR/libmpv-wrapper.dll" ]; then
  echo "ERROR: 'libmpv-wrapper.dll' could not be found or verified in the target directory"
  echo "Aborting script execution."
  exit 1
fi	
echo "Successfully verified libmpv-wrapper link layer."

REQUIRED_DLLS=(
  "avcodec"
  "avfilter"
  "avformat"
  "avutil"
  "libass"
  "libbrotlicommon"
  "libbrotlidec"
  "libbz2"
  "libdav1d"
  "libdovi"
  "libexpat"
  "libfontconfig"
  "libfreetype"
  "libfribidi"
  "libgcc_s_seh"
  "libglib"
  "libgraphite2"
  "libharfbuzz-0"
  "libiconv"
  "libintl"
  "libjpeg"
  "liblcms2"
  "libpcre2-8"
  "libplacebo"
  "libpng"
  "libshaderc_shared"
  "libspirv-cross-c-shared"
  "libstdc++"
  "libSvtAv1Enc"
  "libunibreak"
  "libva"
  "libva_win32"
  "libvpx"
  "libwinpthread"
  "libx264"
  "libx265"
  "libzimg"
  "swresample"
  "swscale"
  "zlib"
)

echo "Locating and gathering environment runtime dependencies..."
for dll_base in "${REQUIRED_DLLS[@]}"; do
  copied=0
  
  while IFS= read -r -d '' match; do
    echo "Found dependency: $(basename "$match")"
    cp "$match" "$OUTPUT_DIR/"
    copied=1
  done < <(find "$STAGE_FF_DIR" -maxdepth 1 -type f -name "${dll_base}*.dll" -print0 2>/dev/null)
  
  if [ "$copied" -eq 0 ]; then
    while IFS= read -r -d '' match; do
      echo "Found dependency: $(basename "$match")"
      cp "$match" "$OUTPUT_DIR/"
      copied=1
    done < <(find /mingw64/bin -maxdepth 1 -type f -name "${dll_base}*.dll" -print0 2>/dev/null)
  fi

  if [ "$copied" -eq 0 ]; then
    while IFS= read -r -d '' match; do
      echo "Found dependency (alt match): $(basename "$match")"
      cp "$match" "$OUTPUT_DIR/"
      copied=1
    done < <(find /mingw64/bin -maxdepth 1 -type f -iname "*${dll_base}*.dll" -print0 2>/dev/null)
    
    if [ "$copied" -eq 0 ]; then
      while IFS= read -r -d '' match; do
        echo "Found dependency (alt match): $(basename "$match")"
        cp "$match" "$OUTPUT_DIR/"
        copied=1
      done < <(find "$STAGE_FF_DIR" -maxdepth 1 -type f -iname "*${dll_base}*.dll" -print0 2>/dev/null)
    fi

    if [ "$copied" -eq 0 ]; then
      echo "WARNING: Could not resolve dependency mapping target for element: ${dll_base}"
    fi
  fi
done

echo "--------------------------------------------------------"
echo "SUCCESS: Everything is gathered and ready for deployment."
echo "Your shipping directory is located at: $OUTPUT_DIR"
echo "--------------------------------------------------------"
ls -la "$OUTPUT_DIR"