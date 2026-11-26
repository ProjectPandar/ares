#!/usr/bin/env bash
# Provisions pinned, SHA-256-verified unmodified Moonraker metadata.py extractors
# (4ba6fd228 and v0.9.3 71f9e677b) plus Pillow into a cacheable directory and
# appends ARES_MOONRAKER_* env vars to the target env file.
set -euo pipefail

cache_dir="${1:?usage: setup-moonraker.sh <cache-dir> [env-file]}"
env_file="${2:-${GITHUB_ENV:-/dev/stdout}}"

CURRENT_REV="4ba6fd228f264564f6613197fd9abb18ec068cf4"
CURRENT_SHA="ae2488ff23e6ddcaf961b063506dad1a7141b38b92102bc174422ed244641d2e"
LEGACY_REV="71f9e677b81afcc6b99dd5002f595025c38edc7b"
LEGACY_SHA="871632a1c71aa066ecf3bc88f71c978e249e88568f75f2e46b622c91afb0051c"

mkdir -p "$cache_dir/4ba6fd228" "$cache_dir/71f9e677b" "$cache_dir/pillow"

fetch_verified() {
  local rev="$1" sha="$2" dst="$3"
  local url="https://raw.githubusercontent.com/Arksine/moonraker/${rev}/moonraker/components/file_manager/metadata.py"
  if [ ! -f "$dst" ] || [ "$(sha256sum "$dst" | awk '{print $1}')" != "$sha" ]; then
    curl -fsSL "$url" -o "$dst.tmp"
    actual="$(sha256sum "$dst.tmp" | awk '{print $1}')"
    if [ "$actual" != "$sha" ]; then
      rm -f "$dst.tmp"
      echo "SHA-256 mismatch for Moonraker $rev: expected $sha, got $actual" >&2
      exit 1
    fi
    mv "$dst.tmp" "$dst"
  fi
}

fetch_verified "$CURRENT_REV" "$CURRENT_SHA" "$cache_dir/4ba6fd228/metadata.py"
fetch_verified "$LEGACY_REV" "$LEGACY_SHA" "$cache_dir/71f9e677b/metadata.py"

python_bin="$(command -v python3)"
if ! "$python_bin" -I -c 'import sys; sys.path.insert(0, sys.argv[1]); import PIL' "$cache_dir/pillow" 2>/dev/null; then
  "$python_bin" -m pip install --quiet --no-cache-dir --target "$cache_dir/pillow" pillow
  "$python_bin" -I -c 'import sys; sys.path.insert(0, sys.argv[1]); import PIL' "$cache_dir/pillow"
fi

{
  echo "ARES_MOONRAKER_METADATA_PY=$cache_dir/4ba6fd228/metadata.py"
  echo "ARES_MOONRAKER_LEGACY_METADATA_PY=$cache_dir/71f9e677b/metadata.py"
  echo "ARES_MOONRAKER_PYTHON=$python_bin"
  echo "ARES_MOONRAKER_PILLOW_PATH=$cache_dir/pillow"
} >> "$env_file"
