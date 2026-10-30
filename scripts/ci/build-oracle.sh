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

# Pinned nixpkgs identical to the local gate store path
# /nix/store/9qzkr0n1671awz5sjrmbgzmbp0y58wpc-source
# (narHash sha256-dveqXShac4ZH9whSBeFyP8GY4iQU6zpMIkyCgPUdLOY=).
NIXPKGS_URL="github:NixOS/nixpkgs/dc56dcbed57fa354b45a3a68a4eedd74663ae5ce"
nixpkgs="$(nix eval --raw --expr "(builtins.getFlake \"$NIXPKGS_URL\").outPath")"

# Fresh source tree from the pinned upstream tarball. Fetching the archive
# keeps the nix src (and therefore the derivation hash) identical whether the
# job checked out OrcaSlicer sparsely or not at all.
ORCA_REV="8500fcdccaa10b5099ac20d252af3a7c560046f1" # v2.4.2
src="$(mktemp -d /tmp/orca-src-XXXXXX)"
curl --proto '=https' --tlsv1.2 -fL --retry 3 \
  "https://github.com/SoftFever/OrcaSlicer/archive/$ORCA_REV.tar.gz" \
  | tar xz --strip-components=1 -C "$src"
git -C "$src" init -q .

case "$variant" in
  merged-oracle)
    git -C "$src" apply --unidiff-zero \
      "$repo/scripts/orca-replay/seed-only.patch" \
      "$repo/scripts/orca-replay/concentric-initialization.patch" \
      "$repo/scripts/orca-replay/object-id-initialization.patch"
    git -C "$src" apply \
      "$repo/tools/chain-probe/procgcode.patch" \
      "$repo/tools/chain-probe/procgcode-stable-options.patch" \
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
    # The approved seed-replay oracle carries all three initialization patches,
    # not just seed-only (verified byte-identical to the accepted source tree).
    git -C "$src" apply --unidiff-zero \
      "$repo/scripts/orca-replay/seed-only.patch" \
      "$repo/scripts/orca-replay/concentric-initialization.patch" \
      "$repo/scripts/orca-replay/object-id-initialization.patch"
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
    # Same source as the approved print-statistics oracle v4zhqnbm: the full
    # seed-replay producer source (three init patches) + print-statistics.patch
    # added by the nix recipe.
    git -C "$src" apply --unidiff-zero \
      "$repo/scripts/orca-replay/seed-only.patch" \
      "$repo/scripts/orca-replay/concentric-initialization.patch" \
      "$repo/scripts/orca-replay/object-id-initialization.patch"
    ORCA_PRINT_STATISTICS_SOURCE="$src" ORCA_PRINT_STATISTICS_NIXPKGS="$nixpkgs" \
    ORCA_PLATE_CUSTOM_GCODES_SOURCE="$src" ORCA_PLATE_CUSTOM_GCODES_NIXPKGS="$nixpkgs" \
      nix build --impure "${jobs_flag[@]}" --no-link --print-out-paths \
      --file "$repo/tools/chain-probe/$variant.nix"
    ;;
  ksr-initialized)
    git -C "$src" apply --unidiff-zero "$repo/tools/orca-probes/ksr-initialized-source.patch"
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
