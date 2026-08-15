#!/usr/bin/env bash
set -euo pipefail

LIST="legal/distributed-msys2-packages.txt"
OUT="legal/generated/msys2"
mkdir -p "$OUT/licenses"
: > "$OUT/packages.tsv"
: > "$OUT/missing-license-files.txt"

[[ -f "$LIST" ]] || { echo "No $LIST"; exit 0; }
command -v pactree >/dev/null || { echo "Run: pacman -S --needed --noconfirm pactree"; exit 1; }

while IFS= read -r root; do
  [[ -z "$root" || "$root" =~ ^[[:space:]]*# ]] && continue
  echo "Resolving: $root"
  mapfile -t pkgs < <(pactree -lu "$root" | sort -u)
  for pkg in "${pkgs[@]}"; do
    info="$(pacman -Qi "$pkg")"
    name="$(awk '/^Name/ {print $3}' <<<"$info")"
    version="$(awk '/^Version/ {print $3}' <<<"$info")"
    license="$(awk -F': *' '/^Licenses/ {print $2}' <<<"$info")"
    url="$(awk -F': *' '/^URL/ {print $2}' <<<"$info")"
    printf '%s\t%s\t%s\t%s\n' "$name" "$version" "$license" "$url" >> "$OUT/packages.tsv"
    mkdir -p "$OUT/licenses/$name"
    found=0
    while IFS= read -r f; do
      [[ -f "$f" ]] && { cp -f "$f" "$OUT/licenses/$name/"; found=1; }
    done < <(pacman -Ql "$pkg" | awk '{print $2}' \
            | grep -Ei '/(LICENSE|LICENCE|COPYING|COPYRIGHT)([.[:space:]]|$)' || true)
    [[ "$found" -eq 0 ]] && echo "$name" >> "$OUT/missing-license-files.txt"
  done
done < "$LIST"

sort -u "$OUT/packages.tsv" -o "$OUT/packages.tsv"
echo "Done → $OUT"
[[ -s "$OUT/missing-license-files.txt" ]] && cat "$OUT/missing-license-files.txt" || true
