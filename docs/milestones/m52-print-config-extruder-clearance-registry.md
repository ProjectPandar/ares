# M52: PrintConfig extruder clearance option registry

## Goal
Port the FFF extruder-clearance and nozzle-height option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2127-2160` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1513-1516` and `PrintConfig.cpp:2127-2160`; no new Ares pipeline, crate, dependency, collision-avoidance behavior, by-object scheduling, nozzle-height behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `extruder_clearance_height_to_rod`, `extruder_clearance_height_to_lid`, `extruder_clearance_radius`, and `nozzle_height` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- The registry definition table is split into an additional sorted shard so modified Rust files remain under 400 LOC while preserving public API.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/category/tooltip/sidetext/min/mode metadata remains deferred because the current registry boundary stores only key, kind, default, and source citation.
- Collision avoidance, by-object scheduling, nozzle-height runtime behavior, typed accessors, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `extruder` from `PrintConfig.cpp:2112-2125` remains deferred because the current registry boundary requires an explicit default value and upstream does not set one in this slice.
- `bed_mesh_min` and following options from `PrintConfig.cpp:2162+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
