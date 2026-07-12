# M111: PrintConfig internal solid infill and spiral registry

## Goal
Port the adjacent internal-solid-infill and spiral option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5657-5726` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1162-1163`, `PrintConfig.hpp:1560-1564`, `PrintConfig.cpp:5657-5726`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, internal-solid-infill width/speed behavior, spiral-vase path generation, XY smoothing, spiral transition flow behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `internal_solid_infill_line_width`, `internal_solid_infill_speed`, `spiral_mode`, `spiral_mode_smooth`, `spiral_mode_max_xy_smoothing`, `spiral_starting_flow_ratio`, and `spiral_finishing_flow_ratio` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Runtime behavior for internal solid infill, spiral vase, XY smoothing, spiral transition flow, slicing, extrusion, and downstream G-code behavior remains unchanged/deferred.
- Following `timelapse_type`, standby/preheat, and later options from `PrintConfig.cpp:5728+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
