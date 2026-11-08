#!/usr/bin/env bash
# Reproduce the Phase 1-2 artifact pipeline for eTilbudsavis 7.2.
#
# Everything this script writes is gitignored. Run it from the repository root
# with the flake devshell on PATH (`nix develop -c ./re/scripts/extract_apk.sh`).
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$root"

mkdir -p apk/raw apk/extracted re/decompiled re/extracted

if ! compgen -G "apk/raw/*.xapk" >/dev/null; then
  apkeep -a com.eTilbudsavis.eTilbudsavis@7.2 -d apk-pure \
    -o 'acknowledge_dangers=true' apk/raw
fi

for archive in apk/raw/*.xapk; do
  unzip -o "$archive" -d apk/extracted >/dev/null
done

unzip -o apk/extracted/com.eTilbudsavis.eTilbudsavis.apk 'assets/*' -d re/extracted >/dev/null

jadx -d re/decompiled/jadx --no-res apk/extracted/com.eTilbudsavis.eTilbudsavis.apk
apktool d -f -o re/decompiled/apktool apk/extracted/com.eTilbudsavis.eTilbudsavis.apk

find apk re/extracted -type f -printf '%s %p\n' | sort -nr | head -40
