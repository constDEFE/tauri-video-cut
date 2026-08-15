#!/usr/bin/env bash
set -euo pipefail
mkdir -p LICENSES

BASE_URL="https://raw.githubusercontent.com/spdx/license-list-data/main/text"

ids=(
    MIT
    Apache-2.0
    GPL-2.0-only
    GPL-2.0-or-later
    LGPL-2.1-only
    BSD-2-Clause
    BSD-3-Clause
    ISC
    CC0-1.0
    Zlib
    MPL-2.0
)

for id in "${ids[@]}"; do
    dest="LICENSES/${id}.txt"
    if [[ ! -f "$dest" ]]; then
        echo "Downloading $id..."
        curl -fsSL "${BASE_URL}/${id}.txt" -o "$dest" || \
            echo "WARN: Failed to download $id"
    else
        echo "Already exists: $dest"
    fi
done

echo ""
echo "License texts ready in LICENSES/"
ls -1 LICENSES/
