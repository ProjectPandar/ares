# M66: PrintConfig filament tower purge and interface option registry

## Goal
Port the adjacent FFF filament minimal purge, wipe-tower cooling, and tower interface pre-extrusion/ironing option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2678-2719` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1443-1447`, `PrintConfig.cpp:2678-2719`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, wipe-tower purge/cooling/interface behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `filament_minimal_purge_on_wipe_tower`, `filament_cooling_before_tower`, `filament_tower_interface_pre_extrusion_dist`, `filament_tower_interface_pre_extrusion_length`, and `filament_tower_ironing_area` with exact kinds, defaults, and source line ranges.
- `filament_cooling_before_tower` preserves upstream nullable float-vector identity through existing metadata-only `FloatsNullable` kind.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/tooltip/sidetext/min/max/mode/nullable metadata remains deferred beyond the current metadata boundary, except preserving nullable kind identity.
- Wipe-tower purge/cooling/interface behavior, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `filament_tower_interface_purge_volume`, `filament_tower_interface_print_temp`, `filament_cooling_final_speed`, and following options from `PrintConfig.cpp:2721+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
