# M201: PrintConfig validate extruder clearance dimensions

## Goal
Port OrcaSlicer's extruder-clearance validation slice into Ares as an explicit `SliceOptions::validate_extruder_clearance_options()` API for UI/config consumers.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10187-10198`, with option-definition/default context from `PrintConfig.cpp:2127-2160` and `PrintConfig.hpp:1513-1516`. It covers only `extruder_clearance_radius`, `extruder_clearance_height_to_rod`, `extruder_clearance_height_to_lid`, and `nozzle_height` positive-value validation. No filament-flow, spiral-vase, later validation, full `DynamicPrintConfig::validate`, `FullPrintConfig`, UI runtime, slicing, extrusion, G-code, new crate, or dependency behavior is added.

## Exit checklist
- `SliceOptions::validate_extruder_clearance_options()` returns a key-to-message map like Orca validation.
- Missing clearance/nozzle keys use source-cited registry defaults and pass.
- `extruder_clearance_radius <= 0` reports key `extruder_clearance_radius` with `invalid value {value:.6}`.
- `extruder_clearance_height_to_rod <= 0` reports key `extruder_clearance_height_to_rod` with `invalid value {value:.6}`.
- `extruder_clearance_height_to_lid <= 0` reports key `extruder_clearance_height_to_lid` with `invalid value {value:.6}`.
- `nozzle_height <= 0` reports key `nozzle_height` with `invalid value {value:.6}`.
- JSON boundary type errors for non-number/non-numeric-string values return `SliceError::InvalidInput`; numeric strings remain accepted to match existing Ares numeric option boundary behavior.
- Existing M196-M200 validation behavior remains intact.
- `PrintConfig.cpp:10200+` validation behavior remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
