#!/usr/bin/env bash
# Builds one pinned OrcaSlicer v2.5.0 (40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19)
# oracle variant or runtime dependency for CI and prints the resulting /nix/store
# path(s) on stdout.
set -euo pipefail

variant="${1:?usage: build-oracle.sh <variant>}"
repo="$(cd "$(dirname "$0")/../.." && pwd)"

jobs_flag=(--cores "${ORCA_BUILD_CORES:-4}" --max-jobs 1)
if [ "${ORACLE_SUBSTITUTE_ONLY:-0}" = "1" ]; then
  jobs_flag=(--max-jobs 0)
fi

# Pinned nixpkgs commit matching the release oracle toolchain
# (narHash sha256-dveqXShac4ZH9whSBeFyP8GY4iQU6zpMIkyCgPUdLOY=).
NIXPKGS_URL="github:NixOS/nixpkgs/dc56dcbed57fa354b45a3a68a4eedd74663ae5ce"
nixpkgs="${ORCA_PROCGCODE_NIXPKGS:-$(nix eval --raw --expr "(builtins.getFlake \"$NIXPKGS_URL\").outPath")}"

case "$variant" in
  merged-oracle|seed-replay|plate-custom-gcodes|preheat-move-durations|segment-colors|preset-select)
    ORCA_PROCGCODE_NIXPKGS="$nixpkgs" \
      exec nix build --impure "${jobs_flag[@]}" --no-link --print-out-paths \
      --file "$repo/tools/chain-probe/$variant.nix"
    ;;
  gdb|patchelf|glibcLocales)
    exec nix build "${jobs_flag[@]}" --no-link --print-out-paths "$NIXPKGS_URL#$variant"
    ;;
  official-libs)
    NIX_PATH="nixpkgs=$nixpkgs" \
      exec nix build --impure "${jobs_flag[@]}" --no-link --print-out-paths --expr \
      'with import <nixpkgs> {}; builtins.map lib.getLib [ gtk3 webkitgtk_4_1 libGLU pango glib glib-networking gst_all_1.gstreamer gst_all_1.gst-plugins-base libsoup_3 libx11 libxext libxkbcommon libSM libICE libglvnd stdenv.cc.cc.lib wayland harfbuzz atk cairo gdk-pixbuf fontconfig.lib dbus libsecret glibc ]'
    ;;
  *)
    echo "unknown oracle variant: $variant" >&2
    exit 2
    ;;
esac
