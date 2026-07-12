# M34: PrintConfig bridge angle and density option registry

## Goal
Port the FFF bridge angle and bridge density `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1213-1264` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:991,1081-1082,1189` and `PrintConfig.cpp:1213-1264`; no new Ares pipeline, crate, bridge planning, bridge density behavior, extrusion behavior, G-code behavior, filesystem, network, UI, or slicing behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes the four bridge angle/density options with exact defaults and source line ranges.
- `option_definition()` lookup remains sorted/binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- `bridge_flow`, `internal_bridge_flow`, top/bottom solid infill flow ratio options, bridge planning, bridge density spacing, extrusion behavior, and bridge G-code behavior remain deferred or unchanged.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
