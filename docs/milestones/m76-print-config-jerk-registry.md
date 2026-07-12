# M76: PrintConfig wall, infill, and travel jerk option registry

## Goal
Port the adjacent FFF wall, infill, top surface, first layer, travel, and first-layer travel jerk option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3188-3249` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1053-1058,1423`, `PrintConfig.cpp:3188-3249`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, jerk runtime behavior, ratio resolution behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `outer_wall_jerk`, `inner_wall_jerk`, `top_surface_jerk`, `infill_jerk`, `initial_layer_jerk`, `travel_jerk`, and `initial_layer_travel_jerk` with exact kinds, defaults, and source line ranges.
- `default_jerk` and `default_junction_deviation` remain deferred to a later source-cited milestone because adding their sorted `default_*` registry entries requires a separate pre-middle shard split to keep modified Rust files under 400 LOC.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/category/tooltip/sidetext/min/mode/ratio metadata remains deferred beyond the current metadata boundary.
- Jerk resolution, first-layer travel jerk ratio behavior, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `initial_layer_line_width`, `initial_layer_print_height`, and following options from `PrintConfig.cpp:3251+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
