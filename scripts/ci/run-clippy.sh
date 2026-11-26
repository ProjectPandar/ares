#!/usr/bin/env bash
set -euo pipefail

RUST_CRATES="${RUST_CRATES:-all}"
if [ "$RUST_CRATES" = "all" ]; then
  cargo clippy --workspace --all-targets --all-features -- -D warnings
else
  args=()
  for c in $RUST_CRATES; do args+=(-p "$c"); done
  cargo clippy "${args[@]}" --all-targets --all-features -- -D warnings
fi
