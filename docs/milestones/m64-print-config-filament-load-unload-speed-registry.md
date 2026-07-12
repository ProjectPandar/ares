# M64: PrintConfig filament load/unload speed option registry

## Goal
Port the adjacent FFF filament adhesiveness category and filament loading/unloading speed option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2596-2634` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1320,1436-1439`, `PrintConfig.cpp:2596-2634`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, wipe-tower loading/unloading behavior, ramming/toolchange runtime, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `filament_adhesiveness_category`, `filament_loading_speed`, `filament_loading_speed_start`, `filament_unloading_speed`, and `filament_unloading_speed_start` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/tooltip/sidetext/min/max/mode metadata remains deferred beyond the current metadata boundary.
- Wipe-tower loading/unloading behavior, ramming, toolchange runtime, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `filament_toolchange_delay`, `filament_cooling_moves`, and following options from `PrintConfig.cpp:2636+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
