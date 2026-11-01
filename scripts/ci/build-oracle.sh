#!/usr/bin/env bash
# Builds one Native oracle variant for CI (matrix job calls it per variant)
# and prints the resulting /nix/store paths (all paired seed outputs for producers).
# Requires Nix with flakes. Producers may reuse the configured Hestia cache.
set -euo pipefail

variant="${1:?usage: build-oracle.sh <variant>}"
repo="$(cd "$(dirname "$0")/../.." && pwd)"

# Consumers import in-run Native outputs first. Only signed Nixpkgs dependencies
# may be substituted afterwards; missing Native outputs cannot build locally.
jobs_flag=(--cores 4 --max-jobs 1)
if [ "${ORACLE_SUBSTITUTE_ONLY:-0}" = "1" ]; then
  jobs_flag=(--max-jobs 0 --substituters https://cache.nixos.org)
fi

# Pinned build dependencies shared by every Native variant.
NIXPKGS_URL="github:NixOS/nixpkgs/dc56dcbed57fa354b45a3a68a4eedd74663ae5ce"
nixpkgs="$(nix eval --raw --expr "(builtins.getFlake \"$NIXPKGS_URL\").outPath")"

# Fresh source tree from the pinned upstream tarball. Fetching the archive
# keeps the nix src (and therefore the derivation hash) identical whether the
# job checked out OrcaSlicer sparsely or not at all.
ORCA_REV="345efec64c7b95a05ce5cf96df0dde66a158835c" # v2.5.0-beta
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
src="$work/source"
mkdir "$src"
curl --proto '=https' --tlsv1.2 -fL --retry 3 \
  "https://github.com/OrcaSlicer/OrcaSlicer/archive/$ORCA_REV.tar.gz" -o "$work/source.tar.gz"
printf '%s  %s\n' 7b17a77b2a8f811bac96c6f7a538df0c838d37a4a02721dd7f73f45420938ff4 \
  "$work/source.tar.gz" | sha256sum --check >&2
tar xzf "$work/source.tar.gz" --strip-components=1 -C "$src"
git -C "$src" init -q .

case "$variant" in
  merged-oracle)
    git -C "$src" apply "$repo/scripts/orca-replay/seed-only.patch"
    git -C "$src" apply \
      "$repo/tools/chain-probe/procgcode.patch" \
      "$repo/tools/chain-probe/procgcode-dynamics.patch" \
      "$repo/tools/chain-probe/marker-time.patch" \
      "$repo/tools/chain-probe/print-statistics.patch"
    cp "$repo/tools/chain-probe/marker-time-oracle.hpp" "$src/src/marker-time-oracle.hpp"
    mkdir -p "$src/src/marker-time"
    cp "$repo/tools/chain-probe/marker-time/kernel.hpp" "$src/src/marker-time/kernel.hpp"
    ORCA_PROCGCODE_SOURCE="$src" ORCA_PROCGCODE_NIXPKGS="$nixpkgs" \
      nix build --impure "${jobs_flag[@]}" --no-link --print-out-paths \
      --file "$repo/tools/chain-probe/merged-oracle.nix"
    ;;
  seed-replay|seed-replay-debug|seed-replay-source)
    # Keep the Native ELF, debug info and source on one derivation so the
    # read-only Voronoi observer never mixes suppliers.
    attribute=out
    case "$variant" in
      seed-replay-debug) attribute=debug ;;
      seed-replay-source) attribute=src ;;
    esac
    # Beta owns the Arachne and object-id initializers; only seed RNGs here.
    git -C "$src" apply "$repo/scripts/orca-replay/seed-only.patch"
    export ORCA_REPLAY_SOURCE="$src" ORCA_REPLAY_NIXPKGS="$nixpkgs"
    if [ "${ORACLE_SUBSTITUTE_ONLY:-0}" = "1" ]; then
      if [ "$attribute" = "src" ]; then
        nix build --impure "${jobs_flag[@]}" --no-link --print-out-paths \
          --file "$repo/scripts/orca-replay/build.nix" src
      else
        nix build --impure "${jobs_flag[@]}" --no-link --print-out-paths \
          --file "$repo/scripts/orca-replay/build.nix" "^$attribute"
      fi
    else
      nix build --impure "${jobs_flag[@]}" --no-link --print-out-paths \
        --file "$repo/scripts/orca-replay/build.nix" '^*'
      nix build --impure "${jobs_flag[@]}" --no-link --print-out-paths \
        --file "$repo/scripts/orca-replay/build.nix" src
    fi
    ;;
  procgcode|procgcode-stable-options|procgcode-dynamics|procgcode-dynamics-stable-options|marker-time|marker-time-stable-options|custom-gcode-overview|actual-speed-profile|role-times|segment-hw|segment-colors)
    ORCA_PROCGCODE_SOURCE="$src" ORCA_PROCGCODE_NIXPKGS="$nixpkgs" \
      nix build --impure "${jobs_flag[@]}" --no-link --print-out-paths \
      --file "$repo/tools/chain-probe/$variant.nix"
    ;;
  print-statistics|plate-custom-gcodes)
    # Statistics and plate diagnostics observe the same seeded beta source.
    git -C "$src" apply "$repo/scripts/orca-replay/seed-only.patch"
    ORCA_PRINT_STATISTICS_SOURCE="$src" ORCA_PRINT_STATISTICS_NIXPKGS="$nixpkgs" \
    ORCA_PLATE_CUSTOM_GCODES_SOURCE="$src" ORCA_PLATE_CUSTOM_GCODES_NIXPKGS="$nixpkgs" \
      nix build --impure "${jobs_flag[@]}" --no-link --print-out-paths \
      --file "$repo/tools/chain-probe/$variant.nix"
    ;;
  ksr-initialized)
    git -C "$src" apply "$repo/tools/orca-probes/ksr-initialized-source.patch"
    ORCA_KSR_INITIALIZED_SOURCE="$src" ORCA_KSR_INITIALIZED_NIXPKGS="$nixpkgs" \
      nix build --impure "${jobs_flag[@]}" --no-link --print-out-paths \
      --file "$repo/tools/orca-probes/ksr-initialized.nix"
    ;;
  gdb|patchelf)
    nix build "${jobs_flag[@]}" --no-link --print-out-paths "$NIXPKGS_URL#$variant"
    ;;
  *)
    echo "unknown variant: $variant" >&2
    exit 2
    ;;
esac
