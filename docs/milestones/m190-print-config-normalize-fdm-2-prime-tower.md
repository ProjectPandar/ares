# M190: PrintConfig normalize_fdm_2 prime tower changed keys

## Goal
Port OrcaSlicer's changed-key-returning prime-tower normalization branch from `DynamicPrintConfig::normalize_fdm_2` into an explicit advanced `SliceOptions::normalize_fdm_2(num_objects, used_filaments)` API for future UI/API consumers.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8452-8505` plus declaration context in `PrintConfig.hpp:628-631`, covering only the optional `enable_prime_tower`, `independent_support_layer_height`, `print_sequence`, `timelapse_type`, and `enable_wrapping_detection` changed-key branch. No `normalize_fdm_1`, commented-out adaptive-layer-height behavior, commented-out independent-support re-enable branch, automatic `Print::Apply` integration, slicing, extrusion, G-code, UI runtime, new crate, or dependency behavior is added.

## Exit checklist
- `normalize_fdm_2` returns an empty changed-key list when `used_filaments == 0` or `enable_prime_tower` is absent.
- Entering the branch creates `independent_support_layer_height` with Orca's default `true` when absent without reporting the default creation by itself.
- Non-smooth timelapse with wrapping detection disabled and one used filament disables true `enable_prime_tower` and reports `enable_prime_tower`.
- Non-smooth timelapse with wrapping detection disabled, `print_sequence = "by object"`, and more than one object disables true `enable_prime_tower` and reports `enable_prime_tower`.
- `print_sequence = "by object"` with exactly one object does not disable prime tower.
- Smooth timelapse or enabled wrapping detection preserves enabled prime tower and only reports `independent_support_layer_height` when that value changes to false.
- Already false `enable_prime_tower` and already false `independent_support_layer_height` are not reported as changed.
- M186-M189 `normalize_fdm(used_filaments)` behavior remains intact.
- Deferred `normalize_fdm_1`, commented-out behavior, and `Print::Apply` integration remain unchanged.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
