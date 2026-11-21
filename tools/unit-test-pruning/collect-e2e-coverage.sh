#!/usr/bin/env bash
# E2E-only coverage for unit-test pruning evidence. Runs orca_parity (seeded,
# which exercises every ares path the official suite does) + stock_project_whole_file.
set -euo pipefail
P=/nix/store/wahgmwqll1vg1svk64fjqgzfzq0q2asf-llvm-23.1.0-rc1
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
if [ -n "${ARES_PARITY_ENV:-}" ]; then
  set -a
  source "$ARES_PARITY_ENV"
  set +a
fi
: "${ORCA_REPLAY_BIN:?select the v2.5.0-beta seed-only executable}"
: "${ARES_PARITY_ARTIFACT_ROOT:?select an external artifact directory}"
: "${ARES_ORCA_CACHE_DIR:?select an external reference cache}"
exec env \
  LLVM_COV=$P/bin/llvm-cov LLVM_PROFDATA=$P/bin/llvm-profdata \
  CARGO_INCREMENTAL=0 \
  ARES_ORCA_BIN="$ROOT/scripts/orca-seeded-parity.sh" \
  ORCA_REPLAY_SEED=0 ARES_PARITY_SEEDED_REFERENCE=1 \
  cargo llvm-cov -p ares-cli --test orca_parity --test stock_project_whole_file \
    --no-report
