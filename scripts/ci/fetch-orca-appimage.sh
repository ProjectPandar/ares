#!/usr/bin/env bash
# Downloads (or reuses a cached) official OrcaSlicer v2.5.0 AppImage, verifies
# its pinned SHA-256 digest, and extracts squashfs-root into $1 (ORCA_APPDIR).
set -euo pipefail

DEST="${1:?usage: fetch-orca-appimage.sh <dest-dir> [cached-appimage]}"
CACHE_FILE="${2:-}"
URL="https://github.com/SoftFever/OrcaSlicer/releases/download/v2.5.0/OrcaSlicer_Linux_AppImage_Ubuntu2404_V2.5.0.AppImage"
SHA256="d13eb0a9cd618ed080ab6b82c64c0a60c4501aac91e88d5ac2c3b526fb141a7c"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

appimage="$work/orca.AppImage"
if [ -n "$CACHE_FILE" ] && [ -f "$CACHE_FILE" ]; then
  if printf '%s  %s\n' "$SHA256" "$CACHE_FILE" | sha256sum -c - >/dev/null 2>&1; then
    cp "$CACHE_FILE" "$appimage"
  fi
fi

if [ ! -f "$appimage" ]; then
  curl --proto '=https' --tlsv1.2 -fL --retry 3 -o "$appimage" "$URL"
  printf '%s  %s\n' "$SHA256" "$appimage" | sha256sum -c -
  if [ -n "$CACHE_FILE" ]; then
    mkdir -p "$(dirname "$CACHE_FILE")"
    cp "$appimage" "$CACHE_FILE"
  fi
fi

chmod +x "$appimage"
mkdir -p "$DEST"
(cd "$work" && ./orca.AppImage --appimage-extract >/dev/null)
cp -a "$work/squashfs-root/." "$DEST/"
test -x "$DEST/bin/orca-slicer"
test -d "$DEST/resources"
