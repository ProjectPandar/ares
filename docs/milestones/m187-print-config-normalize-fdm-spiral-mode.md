# M187: PrintConfig normalize_fdm spiral mode normalization

## Goal
Port the `spiral_mode` branch of OrcaSlicer's `DynamicPrintConfig::normalize_fdm` into the existing explicit `SliceOptions::normalize_fdm(used_filaments)` API.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8355-8369`, covering the `spiral_mode` condition, layer-change retraction disabling, and forced single-wall / no-top / no-infill settings. No resolution clamp, prime-tower, independent-support-height, filament-count, arrange, slicing, extrusion, G-code, UI runtime, new crate, or dependency behavior is added.

## Exit checklist
- `SliceOptions::normalize_fdm` detects present `spiral_mode == true` after the already-ported M186 extruder role propagation.
- When spiral mode is enabled, `retract_when_changing_layer` is present as an array of `false` values, preserving existing array length or using one default entry when absent.
- When spiral mode is enabled, `filament_retract_when_changing_layer` is present as an array of `false` values, preserving existing boolean/null array length or using one default entry when absent.
- When spiral mode is enabled, `wall_loops` is set to `1`, `alternate_extra_wall` to `false`, `top_shell_layers` to `0`, and `sparse_infill_density` to `0`.
- When `spiral_mode` is absent or false, this M187 branch makes no spiral-specific changes.
- Invalid `spiral_mode` or needed retraction-array boundary values return `SliceError::InvalidInput` rather than panicking.
- M186 behavior remains intact.
- `PrintConfig.cpp:8372+` behavior remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
