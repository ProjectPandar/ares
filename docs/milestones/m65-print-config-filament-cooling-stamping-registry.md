# M65: PrintConfig filament cooling and stamping option registry

## Goal
Port the adjacent FFF filament toolchange delay, cooling moves, stamping, and initial cooling speed option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2636-2676` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1440-1442,1455-1456`, `PrintConfig.cpp:2636-2676`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, wipe-tower cooling/stamping behavior, ramming/toolchange runtime, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `filament_toolchange_delay`, `filament_cooling_moves`, `filament_stamping_loading_speed`, `filament_stamping_distance`, and `filament_cooling_initial_speed` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/tooltip/sidetext/min/max/mode metadata remains deferred beyond the current metadata boundary.
- Wipe-tower cooling/stamping behavior, ramming, toolchange runtime, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `filament_minimal_purge_on_wipe_tower`, `filament_cooling_before_tower`, and following options from `PrintConfig.cpp:2678+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
