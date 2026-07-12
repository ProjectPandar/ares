# M196: PrintConfig validate basic dimension and count checks

## Goal
Port the first scalar/vector validation checks from OrcaSlicer's `Slic3r::validate(const FullPrintConfig&, bool)` into Ares as a focused `SliceOptions::validate_basic_fdm_options()` API for UI/config consumers.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10088-10128`, plus `OrcaSlicer/src/libslic3r/libslic3r.h:60` for `SCALING_FACTOR_INTERNAL = 0.000001`. It covers only `layer_height`, `initial_layer_print_height`, `filament_diameter`, `nozzle_diameter`, `wall_loops`, `top_shell_layers`, and `bottom_shell_layers` checks. No firmware-retraction, enum, bridge-flow, clearance, spiral-vase, extrusion-width, full `DynamicPrintConfig::validate`, `FullPrintConfig`, UI runtime, slicing, extrusion, G-code, new crate, or dependency behavior is added.

## Exit checklist
- `SliceOptions::validate_basic_fdm_options()` returns a key-to-message map like Orca validation.
- Default/absent values pass using source-cited registry defaults.
- Invalid `layer_height <= 0` is reported, and the exact upstream predicate `fabs(fmod(layer_height, SCALING_FACTOR)) > 1e-4` is preserved using `SCALING_FACTOR = SCALING_FACTOR_INTERNAL` without inventing a reachable invalid finite JSON case.
- Invalid `initial_layer_print_height <= 0` is reported.
- Any `filament_diameter < 1` is reported.
- Any `nozzle_diameter < 0.005` is reported.
- Negative `wall_loops`, `top_shell_layers`, and `bottom_shell_layers` are reported.
- Type errors at the JSON API boundary return `SliceError::InvalidInput`.
- Existing option count/resize APIs remain intact.
- `PrintConfig.cpp:10131+` validation behavior remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
