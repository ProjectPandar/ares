# M71: PrintConfig filament identity and statistics option registry

## Goal
Port the adjacent FFF filament softening-temperature, price/statistics, identity, and vendor option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2828-2859` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1324,1326,1330,1332`, `PrintConfig.cpp:2828-2859`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, filament statistics behavior, vendor resolution, preset identity behavior, CLI/no-CLI behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `temperature_vitrification`, `filament_cost`, `filament_settings_id`, `filament_ids`, and `filament_vendor` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- The existing registry shards are adjusted so all modified Rust files remain under 400 LOC.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/tooltip/sidetext/min/mode/cli metadata remains deferred beyond the current metadata boundary.
- Softening-temperature behavior, filament cost/statistics behavior, settings-id/ids identity behavior, vendor resolution, UI behavior, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `infill_direction`, `solid_infill_direction`, `sparse_infill_density`, and following options from `PrintConfig.cpp:2861+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
