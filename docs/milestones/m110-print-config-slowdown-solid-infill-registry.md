# M110: PrintConfig slowdown and solid-infill registry

## Goal
Port the adjacent slowdown and solid-infill option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5629-5655` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1160-1161`, `PrintConfig.hpp:1559`, `PrintConfig.cpp:5629-5655`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, layer-time slowdown behavior, sparse-area solid-fill replacement behavior, solid-infill extruder selection behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `slow_down_layer_time`, `minimum_sparse_infill_area`, and `solid_infill_filament` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Runtime behavior for layer-time slowdown, sparse infill replacement, solid-infill extruder selection, slicing, extrusion, and downstream G-code behavior remains unchanged/deferred.
- Following `internal_solid_infill_line_width`, `internal_solid_infill_speed`, spiral-mode, and later options from `PrintConfig.cpp:5657+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
