# M79: PrintConfig first-layer temperature and fan-speed registry

## Goal
Port the adjacent FFF first-layer nozzle temperature and fan-speed option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3316-3370` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1533-1534,1628-1630`, `PrintConfig.cpp:3316-3370`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, nozzle-temperature runtime behavior, fan-speed runtime behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `nozzle_temperature_initial_layer`, `full_fan_speed_layer`, `support_material_interface_fan_speed`, `internal_bridge_fan_speed`, and `ironing_fan_speed` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/full_label/category/tooltip/sidetext/min/max/mode metadata remains deferred beyond the current metadata boundary.
- Nozzle-temperature behavior, fan-speed behavior, disable semantics, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `filament_ironing_flow`, `filament_ironing_spacing`, and following options from `PrintConfig.cpp:3372+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
