# M189: PrintConfig normalize_fdm prime tower normalization

## Goal
Port OrcaSlicer's prime-tower normalization branch from `DynamicPrintConfig::normalize_fdm` into the existing explicit `SliceOptions::normalize_fdm(used_filaments)` API.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8376-8401`, covering only the optional `enable_prime_tower`, `independent_support_layer_height`, `print_sequence`, and `timelapse_type` normalization branch. No commented-out adaptive-layer-height behavior, commented-out independent-support re-enable branch, later `normalize_fdm_1` duplicate behavior, slicing, extrusion, G-code, UI runtime, new crate, or dependency behavior is added.

## Exit checklist
- The branch runs only when `used_filaments > 0` and `enable_prime_tower` is present.
- Non-smooth timelapse with one used filament disables `enable_prime_tower`.
- Non-smooth timelapse with `print_sequence = "by object"` disables `enable_prime_tower`.
- Smooth timelapse (`timelapse_type = "1"`) preserves enabled prime tower even with one used filament.
- Entering the branch creates `independent_support_layer_height` with Orca's default `true` when absent; if `enable_prime_tower` remains true, it is then set to `false`.
- Missing `enable_prime_tower` remains absent and no prime-tower side effects are inserted.
- M186, M187, and M188 behavior remains intact.
- `PrintConfig.cpp:8402+` behavior remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
