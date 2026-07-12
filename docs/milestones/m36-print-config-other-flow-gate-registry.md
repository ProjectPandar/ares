# M36: PrintConfig other-flow gate option registry

## Goal
Port the FFF `set_other_flow_ratios` and `first_layer_flow_ratio` `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1307-1323` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:978`, `PrintConfig.hpp:1214-1215`, and `PrintConfig.cpp:1307-1323`; no new Ares pipeline, crate, flow planning, extrusion behavior, G-code behavior, filesystem, network, UI, preset behavior, or object override behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `set_other_flow_ratios` and `first_layer_flow_ratio` with exact defaults and source line ranges.
- `option_definition()` lookup remains sorted/binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Upstream label/category/tooltip/min/max/mode metadata remains deferred because the current registry boundary stores only key, kind, default, and source citation.
- Runtime flow scaling in `GCode.cpp:6415` and `GCode.cpp:6436`, preset option-list behavior in `Preset.cpp:1186-1187`, and object override handling in `PrintObject.cpp:1397` remain deferred.
- Following per-role flow-ratio options, flow planning, extrusion behavior, and G-code behavior remain deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
