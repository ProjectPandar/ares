# M202: PrintConfig validate filament flow ratio

## Goal
Port OrcaSlicer's filament-flow-ratio validation slice into Ares as an explicit `SliceOptions::validate_filament_flow_ratio_options()` API for UI/config consumers.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10200-10205`, with option-definition/default context from `PrintConfig.cpp:2227-2237` and `PrintConfig.hpp:1301`. It covers only `filament_flow_ratio` vector positive-value validation. No spiral-vase, later validation, full `DynamicPrintConfig::validate`, `FullPrintConfig`, UI runtime, slicing, extrusion, G-code, new crate, or dependency behavior is added.

## Exit checklist
- `SliceOptions::validate_filament_flow_ratio_options()` returns a key-to-message map like Orca validation.
- Missing `filament_flow_ratio` uses the source-cited registry default and passes.
- Any `filament_flow_ratio` entry `<= 0` reports key `filament_flow_ratio` with `invalid value {serialized_vector}`.
- Vector serialization follows the existing Ares/M196 numeric-vector behavior used for `filament_diameter` and `nozzle_diameter` validation.
- JSON boundary values accepted by the existing numeric-vector parser remain accepted: JSON numbers, numeric strings, arrays of numbers/strings, and comma/semicolon-separated strings.
- Malformed vector boundary values return `SliceError::InvalidInput`.
- Existing M196-M201 validation behavior remains intact.
- `PrintConfig.cpp:10207+` validation behavior remains unchanged/deferred.
- `crates/ares-core/src/options/validation.rs` is split before adding new logic so modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
