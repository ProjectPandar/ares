# M67: PrintConfig filament tower temperature and final cooling option registry

## Goal
Port the adjacent FFF filament tower interface purge volume, tower interface print temperature, and final cooling speed option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2721-2743` into `ares-core` registry metadata while splitting the current `pre_middle` registry shard to keep Rust files below 400 LOC.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1448-1450`, `PrintConfig.cpp:2721-2743`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, wipe-tower purge/temperature/cooling behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- Registry table shards are split without changing existing option metadata or sorted binary-search semantics.
- `OPTION_DEFINITIONS` includes `filament_tower_interface_purge_volume`, `filament_tower_interface_print_temp`, and `filament_cooling_final_speed` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/tooltip/sidetext/min/max/mode metadata remains deferred beyond the current metadata boundary.
- Wipe-tower purge/temperature/cooling behavior, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `filament_ramming_parameters`, `filament_multitool_ramming`, and following options from `PrintConfig.cpp:2745+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
