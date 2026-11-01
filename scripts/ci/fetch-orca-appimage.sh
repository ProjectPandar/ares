#!/usr/bin/env bash
# Downloads the official OrcaSlicer 2.5.0-beta AppImage and extracts it to
# $1 (the resulting ORCA_APPDIR, containing bin/orca-slicer and resources/).
# Reproducible: pinned tag + release asset sha256.
set -euo pipefail

DEST="${1:?usage: fetch-orca-appimage.sh <dest-dir>}"
URL="https://github.com/OrcaSlicer/OrcaSlicer/releases/download/v2.5.0-beta/OrcaSlicer_Linux_AppImage_Ubuntu2404_V2.5.0-beta.AppImage"
SHA256="15b0e9f9d577bc7a84cfef4f1ecc6e16ef75bc253e500d772fcaa66f4c25b97e"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
curl --proto '=https' --tlsv1.2 -fL --retry 3 -o "$work/orca.AppImage" "$URL"
echo "$SHA256  $work/orca.AppImage" | sha256sum -c -
chmod +x "$work/orca.AppImage"
mkdir -p "$DEST"
(cd "$work" && ./orca.AppImage --appimage-extract >/dev/null)
mv "$work/squashfs-root"/* "$DEST"/
test -x "$DEST/bin/orca-slicer"
test -d "$DEST/resources"
