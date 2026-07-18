#!/bin/bash
set -e

echo "=== [5/5] Harvesting Binaries and System Runtime DLL Dependencies ==="

if [ "$MSYSTEM" != "UCRT64" ]; then
  echo "ERROR: This script must be run inside an MSYS2 UCRT64 terminal shell"
  exit 1
fi

WORKSPACE_DIR=$(pwd)
STAGE_FF_DIR="$WORKSPACE_DIR/ffmpeg-dist/bin"
STAGE_MPV_DIR="$WORKSPACE_DIR/mpv-dist/bin"
STAGE_MPV_WRAPPER_DIR="$WORKSPACE_DIR/mpv-wrapper"
OUTPUT_DIR="$WORKSPACE_DIR/../src-tauri/lib"

UCRT_BIN_DIR="${MINGW_PREFIX:-/ucrt64}/bin"

rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"
mkdir -p "$STAGE_MPV_WRAPPER_DIR"

if [ ! -f "$STAGE_FF_DIR/ffmpeg.exe" ] || [ ! -f "$STAGE_MPV_DIR/libmpv-2.dll" ]; then
  echo "ERROR: Core processing binaries were not found. Run scripts 3 and 4 first."
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
  echo "3. Extract or copy 'libmpv-wrapper.dll' into one of these directories:"
  echo "   ${STAGE_MPV_WRAPPER_DIR}"
  echo "   ${OUTPUT_DIR}"
  echo "========================================================================="
  echo ""

  if [ -t 0 ]; then
    read -p "Press [ENTER] once you have manually placed 'libmpv-wrapper.dll' to verify..."
  else
    echo "Non-interactive shell detected. Continuing to verification..."
  fi
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
    echo "place it inside one of these directories:"
    echo "  ${STAGE_MPV_WRAPPER_DIR}"
    echo "  ${OUTPUT_DIR}"
    echo "========================================================================="

    if [ -t 0 ]; then
      read -p "Press [ENTER] once you have manually placed 'libmpv-wrapper.dll'..."
    else
      echo "Non-interactive shell detected. Continuing to verification..."
    fi
  fi
fi

find "$STAGE_MPV_WRAPPER_DIR" -type f -name "libmpv-wrapper.dll" -exec cp -f {} "$OUTPUT_DIR/" \;

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
  "libintl"
  "libpcre2-8"
  "libplacebo"
  "libpng"
  "libshaderc_shared"
  "libspirv-cross-c-shared"
  "libstdc++"
  "libSvtAv1Enc"
  "libunibreak"
  "libvpx"
  "libwinpthread"
  "libx264"
  "libx265"
  "libzimg"
  "swresample"
  "swscale"
  "zlib"
)

copy_dll_matches() {
  local dir="$1"
  local pattern="$2"
  local label="$3"
  local copied=0
  local match

  if [ ! -d "$dir" ]; then
    return 1
  fi

  while IFS= read -r -d '' match; do
    echo "Found dependency (${label}): $(basename "$match")"
    cp -f "$match" "$OUTPUT_DIR/"
    copied=1
  done < <(find "$dir" -maxdepth 1 -type f -name "$pattern" -print0 2>/dev/null)

  [ "$copied" -eq 1 ]
}

echo "Locating and gathering UCRT64 runtime dependencies..."

for dll_base in "${REQUIRED_DLLS[@]}"; do
  if copy_dll_matches "$STAGE_FF_DIR" "${dll_base}*.dll" "ffmpeg-dist exact"; then
    continue
  fi

  if copy_dll_matches "$UCRT_BIN_DIR" "${dll_base}*.dll" "ucrt64 exact"; then
    continue
  fi

  if copy_dll_matches "$UCRT_BIN_DIR" "*${dll_base}*.dll" "ucrt64 alt"; then
    continue
  fi

  if copy_dll_matches "$STAGE_FF_DIR" "*${dll_base}*.dll" "ffmpeg-dist alt"; then
    continue
  fi

  echo "WARNING: Could not resolve dependency mapping target for element: ${dll_base}"
done

echo "--------------------------------------------------------"
echo "SUCCESS: Everything is gathered and ready for deployment."
echo "Your shipping directory is located at: $OUTPUT_DIR"
echo "--------------------------------------------------------"

ls -la "$OUTPUT_DIR"
