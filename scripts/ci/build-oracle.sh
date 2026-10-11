#!/usr/bin/env bash
# Builds one Native oracle variant for CI (matrix job calls it per variant)
# and prints the resulting /nix/store path on the last line.
# Requires nix with flakes and the hestia substituter already configured.
set -euo pipefail

variant="${1:?usage: build-oracle.sh <variant>}"
repo="$(cd "$(dirname "$0")/../.." && pwd)"

# Pinned nixpkgs identical to the local gate store path
# /nix/store/9qzkr0n1671awz5sjrmbgzmbp0y58wpc-source
# (narHash sha256-dveqXShac4ZH9whSBeFyP8GY4iQU6zpMIkyCgPUdLOY=).
NIXPKGS_URL="github:NixOS/nixpkgs/dc56dcbed57fa354b45a3a68a4eedd74663ae5ce"
nixpkgs="$(nix eval --raw --expr "(builtins.getFlake \"$NIXPKGS_URL\").outPath")"

# Fresh source tree from the pinned upstream checkout (actions/checkout into
# $repo/OrcaSlicer, sparse-checkout src + resources). git worktree gives the
# full tree each recipe needs.
src="$(mktemp -d /tmp/orca-src-XXXXXX)"
git -C "$repo/OrcaSlicer" worktree add --detach "$src" HEAD >/dev/null

case "$variant" in
  seed-replay)
    git -C "$src" apply --unidiff-zero "$repo/scripts/orca-replay/seed-only.patch"
    ORCA_REPLAY_SOURCE="$src" ORCA_REPLAY_NIXPKGS="$nixpkgs" \
      nix build --impure --cores 4 --max-jobs 1 --no-link --print-out-paths \
      --file "$repo/scripts/orca-replay/build.nix"
    ;;
  procgcode|procgcode-stable-options|procgcode-dynamics|marker-time|marker-time-stable-options)
    ORCA_PROCGCODE_SOURCE="$src" ORCA_PROCGCODE_NIXPKGS="$nixpkgs" \
      nix build --impure --cores 4 --max-jobs 1 --no-link --print-out-paths \
      --file "$repo/tools/chain-probe/$variant.nix"
    ;;
  print-statistics)
    git -C "$src" apply --unidiff-zero "$repo/scripts/orca-replay/seed-only.patch"
    ORCA_PRINT_STATISTICS_SOURCE="$src" ORCA_PRINT_STATISTICS_NIXPKGS="$nixpkgs" \
      nix build --impure --cores 4 --max-jobs 1 --no-link --print-out-paths \
      --file "$repo/tools/chain-probe/print-statistics.nix"
    ;;
  ksr-initialized)
    git -C "$src" apply --unidiff-zero "$repo/tools/orca-probes/ksr-initialized-source.patch"
    ORCA_KSR_INITIALIZED_SOURCE="$src" ORCA_KSR_INITIALIZED_NIXPKGS="$nixpkgs" \
      nix build --impure --cores 4 --max-jobs 1 --no-link --print-out-paths \
      --file "$repo/tools/orca-probes/ksr-initialized.nix"
    ;;
  gdb)
    nix build --cores 4 --max-jobs 1 --no-link --print-out-paths "$NIXPKGS_URL#gdb"
    ;;
  *)
    echo "unknown variant: $variant" >&2
    exit 2
    ;;
esac
