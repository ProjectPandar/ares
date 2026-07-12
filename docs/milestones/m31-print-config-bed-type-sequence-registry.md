# M31: PrintConfig bed type and filament sequence option registry

## Goal
Port the FFF bed type and filament sequence `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1043-1108` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:314-323,333-335,1489,1507-1509`, `PrintConfig.cpp:467-483`, and `PrintConfig.cpp:1043-1108`; no new Ares pipeline, crate, bed-selection, print-order, G-code, filesystem, network, UI, or slicing behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes the seven bed-type and sequence options with exact defaults and source line ranges.
- `option_definition()` lookup remains sorted/binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- `before_layer_change_gcode`, shell/cooling options, enum value APIs, bed-selection behavior, print-order behavior, and G-code behavior remain deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
