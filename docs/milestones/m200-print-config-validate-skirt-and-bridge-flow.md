# M200: PrintConfig validate skirt height and bridge flow ratios

## Goal
Port OrcaSlicer's skirt-height and bridge-flow validation slice into Ares as an explicit `SliceOptions::validate_skirt_and_bridge_flow_options()` API for UI/config consumers.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10172-10185`, with option-definition/default context from `PrintConfig.cpp:1266-1284`, `PrintConfig.cpp:5559-5565`, `PrintConfig.hpp:1083-1084`, and `PrintConfig.hpp:1553`. It covers only `skirt_height < 0`, `bridge_flow <= 0`, and the upstream `internal_bridge_flow` error insertion that is also guarded by `cfg.bridge_flow <= 0`. No extruder-clearance, filament-flow, spiral-vase, later validation, full `DynamicPrintConfig::validate`, `FullPrintConfig`, UI runtime, slicing, extrusion, G-code, new crate, or dependency behavior is added.

## Exit checklist
- `SliceOptions::validate_skirt_and_bridge_flow_options()` returns a key-to-message map like Orca validation.
- Missing `skirt_height`, `bridge_flow`, and `internal_bridge_flow` use source-cited registry defaults and pass.
- Negative `skirt_height` reports key `skirt_height` with `invalid value {value}`.
- `bridge_flow <= 0` reports key `bridge_flow` with `invalid value {bridge_flow:.6}`, matching C++ `std::to_string(double)` formatting for this slice.
- Matching upstream source behavior, `bridge_flow <= 0` also reports key `internal_bridge_flow` with `invalid value {internal_bridge_flow:.6}`, matching C++ `std::to_string(double)` formatting for this slice.
- Matching upstream source behavior, `internal_bridge_flow <= 0` alone does not report an error when `bridge_flow > 0` because `PrintConfig.cpp:10183` checks `cfg.bridge_flow <= 0`.
- JSON boundary type errors for non-integer `skirt_height`, and non-number/non-numeric-string `bridge_flow` / `internal_bridge_flow`, return `SliceError::InvalidInput`.
- Existing M196-M199 validation behavior remains intact.
- `PrintConfig.cpp:10187+` validation behavior remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
