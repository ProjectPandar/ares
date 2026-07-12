# M56: PrintConfig pressure advance option registry

## Goal
Port the adjacent FFF pressure advance option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2252-2262` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1302-1303`, `PrintConfig.cpp:2252-2262`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, pressure advance behavior, firmware-specific behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `enable_pressure_advance` and `pressure_advance` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/tooltip/max/mode behavior remains deferred because the current registry boundary stores only key, kind, default, and source citation.
- Runtime pressure advance, firmware-specific linear advance behavior, adaptive pressure advance behavior, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `adaptive_pressure_advance`, `adaptive_pressure_advance_model`, `adaptive_pressure_advance_overhangs`, `adaptive_pressure_advance_bridges`, and following options from `PrintConfig.cpp:2264+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
