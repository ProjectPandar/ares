#!/usr/bin/env bash
# Downloads the official OrcaSlicer 2.4.2 AppImage and extracts it to
# $1 (the resulting ORCA_APPDIR, containing bin/orca-slicer and resources/).
# Equivalent of the offline /tmp/orca-official-242 squashfs-root used by the
# local parity gates. Reproducible: pinned tag + sha256 of the asset.
set -euo pipefail

DEST="${1:?usage: fetch-orca-appimage.sh <dest-dir>}"
URL="https://github.com/SoftFever/OrcaSlicer/releases/download/v2.4.2/OrcaSlicer_Linux_AppImage_Ubuntu2404_V2.4.2.AppImage"
SHA256="d12fb8c8eac1aecd2dfb6377acd48f994f8fa439ed5292fa532dd82880f029fd"

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
