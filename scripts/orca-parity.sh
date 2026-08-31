#!/usr/bin/env bash
# OrcaSlicer 2.4.2 CLI oracle for the parity suite. Resolves the pinned
# nixpkgs build (cached after the first evaluation) so the smoke and option
# sweeps run without manual environment setup. Set ARES_ORCA_BIN to override.
set -euo pipefail
exec "$(nix build nixpkgs#orca-slicer --print-out-paths --no-link)/bin/orca-slicer" "$@"
