# M60: PrintConfig filament mapping and hardware flag option registry

## Goal
Port the adjacent FFF filament multi-color, filament mapping, nozzle hardness, and filament-switcher hardware flag option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2385-2440` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1334-1336,1341`, `PrintConfig.cpp:2385-2440`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, filament mapping runtime behavior, hardware-switching behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `filament_multi_colour`, `filament_colour_type`, `required_nozzle_HRC`, `filament_map`, `physical_extruder_map`, `filament_map_mode`, `enable_filament_dynamic_map`, and `has_filament_switcher` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/tooltip/min/max/mode/enum-label behavior remains deferred because the current registry boundary stores only key, kind, default, and source citation.
- Filament mapping runtime behavior, dynamic map behavior, filament switcher behavior, nozzle-HRC validation, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `filament_flush_temp`, `filament_flush_volumetric_speed`, and following options from `PrintConfig.cpp:2442+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
