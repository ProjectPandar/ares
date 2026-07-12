# M61: PrintConfig filament flush and toolchange timing option registry

## Goal
Port the adjacent FFF filament flush temperature, flush volumetric speed, max volumetric speed, and filament/tool-change timing option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2442-2497` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1333,1343-1344,1433-1435`, `PrintConfig.cpp:2442-2497`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, flushing runtime behavior, volumetric speed limiting, tool-change timing behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OptionValueKind` includes `IntsNullable` for Orca `ConfigOptionIntsNullable` registry metadata.
- `OPTION_DEFINITIONS` includes `filament_flush_temp`, `filament_flush_volumetric_speed`, `filament_max_volumetric_speed`, `machine_load_filament_time`, `machine_unload_filament_time`, and `machine_tool_change_time` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/tooltip/sidetext/min/max/mode/nullable behavior remains deferred beyond the current metadata boundary, except preserving nullable type identity through `IntsNullable`/`FloatsNullable` kinds.
- Flushing runtime behavior, volumetric speed limiting, tool-change timing behavior, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `support_object_skip_flush`, `bed_temperature_formula`, `nozzle_flush_dataset`, `filament_diameter` source-refresh, and following options from `PrintConfig.cpp:2500+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
