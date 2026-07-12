# M75: PrintConfig acceleration option registry

## Goal
Port the adjacent FFF acceleration and accel-to-decel option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3068-3167` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1043-1050,1419-1422`, `PrintConfig.cpp:3068-3167`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, acceleration runtime behavior, ratio resolution behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `inner_wall_acceleration`, `travel_acceleration`, `top_surface_acceleration`, `outer_wall_acceleration`, `bridge_acceleration`, `sparse_infill_acceleration`, `internal_solid_infill_acceleration`, `initial_layer_acceleration`, `initial_layer_travel_acceleration`, `accel_to_decel_enable`, and `accel_to_decel_factor` with exact kinds, defaults, and source line ranges.
- Existing `default_acceleration` metadata remains unchanged because it is already registered from its earlier upstream source boundary.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/category/tooltip/sidetext/min/max/mode/ratio metadata remains deferred beyond the current metadata boundary.
- Acceleration resolution, accel-to-decel behavior, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `default_jerk`, `default_junction_deviation`, wall/infill jerk, and following options from `PrintConfig.cpp:3169+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
