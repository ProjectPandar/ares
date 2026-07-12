# M63: PrintConfig pellet flow and shrinkage option registry

## Goal
Port the adjacent FFF pellet flow coefficient, adaptive volumetric speed metadata, volumetric speed coefficients, and filament shrinkage option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2551-2594` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1318-1319,1621-1622`, `PrintConfig.cpp:2551-2594`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, pellet-to-diameter conversion, adaptive volumetric speed limiting, shrinkage scaling behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OptionValueKind` includes metadata-only `BoolsNullable` and `Percents` for Orca nullable bool-vector and percent-vector registry metadata.
- `OPTION_DEFINITIONS` includes `pellet_flow_coefficient`, `filament_adaptive_volumetric_speed`, `volumetric_speed_coefficients`, `filament_shrink`, and `filament_shrinkage_compensation_z` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/tooltip/sidetext/min/max/mode/ratio/nullable behavior remains deferred beyond the current metadata boundary, except preserving nullable/percent type identity through `BoolsNullable` and `Percents`.
- Pellet-to-diameter conversion, adaptive volumetric speed limiting, shrinkage scaling, extrusion behavior, slicing behavior, and downstream G-code behavior remain deferred.
- `filament_adhesiveness_category`, `filament_loading_speed`, and following options from `PrintConfig.cpp:2596+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
