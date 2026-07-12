# M62: PrintConfig bed temperature and flush dataset option registry

## Goal
Port the adjacent FFF support skip-flush, bed temperature formula, nozzle flush dataset, and filament diameter source-refresh option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2500-2523` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1317,1339-1340,1342`, `PrintConfig.cpp:2500-2523`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, bed-temperature selection behavior, nozzle flush dataset behavior, support-object skip-flush behavior, extrusion behavior, slicing behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `support_object_skip_flush`, `bed_temperature_formula`, and `nozzle_flush_dataset` with exact kinds, defaults, and source line ranges.
- Existing `filament_diameter` remains `Floats` with default `1.75` and gains the source citation `PrintConfig.hpp:1317; PrintConfig.cpp:2518-2523` without changing typed/runtime behavior.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/tooltip/sidetext/min/mode/enum-label/nullable behavior remains deferred beyond the current metadata boundary, except preserving nullable type identity for `nozzle_flush_dataset` through `IntsNullable`.
- Bed-temperature selection behavior, nozzle flush dataset behavior, support-object skip-flush behavior, extrusion behavior, slicing behavior, and downstream G-code behavior remain deferred.
- `pellet_flow_coefficient`, `filament_adaptive_volumetric_speed`, `volumetric_speed_coefficients`, `filament_shrink`, and following options from `PrintConfig.cpp:2551+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
