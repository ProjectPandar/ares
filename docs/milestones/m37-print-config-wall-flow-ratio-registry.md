# M37: PrintConfig wall flow ratio option registry

## Goal
Port the FFF `outer_wall_flow_ratio` and `inner_wall_flow_ratio` `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1324-1343` into `ares-core` registry metadata while splitting the registry definition table so future option milestones remain below the 400 LOC rule.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1214-1217` and `PrintConfig.cpp:1324-1343`; no new Ares pipeline, crate, flow planning, extrusion behavior, G-code behavior, filesystem, network, UI, preset behavior, or object override behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup, with backing definitions split into smaller Rust source files.
- `OPTION_DEFINITIONS` includes `outer_wall_flow_ratio` and `inner_wall_flow_ratio` with exact defaults and source line ranges.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Upstream label/category/tooltip/min/max/mode metadata remains deferred because the current registry boundary stores only key, kind, default, and source citation.
- Runtime flow scaling, preset option-list behavior, and object override handling remain deferred.
- Following per-role flow-ratio options, flow planning, extrusion behavior, and G-code behavior remain deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
