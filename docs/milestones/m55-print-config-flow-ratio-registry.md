# M55: PrintConfig filament and print flow ratio option registry

## Goal
Port the FFF filament and print flow ratio option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2227-2250` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1301`, `PrintConfig.cpp:2227-2250`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, flow scaling behavior, object/material override behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OptionValueKind` includes a `FloatsNullable` variant for upstream `coFloats` plus `nullable = true` metadata.
- `OPTION_DEFINITIONS` includes `filament_flow_ratio` and `print_flow_ratio` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/category/tooltip/min/max/mode behavior remains deferred because the current registry boundary stores only key, kind, default, and source citation.
- Flow scaling, filament/material override behavior, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `enable_pressure_advance`, `pressure_advance`, adaptive pressure advance options, and following options from `PrintConfig.cpp:2252+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
