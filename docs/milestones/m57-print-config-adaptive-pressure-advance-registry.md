# M57: PrintConfig adaptive pressure advance option registry

## Goal
Port the adjacent FFF adaptive pressure advance option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2264-2320` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1305-1308`, `PrintConfig.cpp:2264-2320`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, adaptive pressure advance behavior, calibration-model parsing, firmware-specific behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `adaptive_pressure_advance`, `adaptive_pressure_advance_model`, `adaptive_pressure_advance_overhangs`, and `adaptive_pressure_advance_bridges` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/tooltip/mode/multiline/full-width/height/max behavior remains deferred because the current registry boundary stores only key, kind, default, and source citation.
- Runtime adaptive pressure advance, calibration-model parsing/fitting, firmware-specific behavior, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `line_width` and following options from `PrintConfig.cpp:2322+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
