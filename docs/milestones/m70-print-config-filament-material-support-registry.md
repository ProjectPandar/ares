# M70: PrintConfig filament material and support option registry

## Goal
Port the adjacent FFF filament material/statistics/support option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2776-2826` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1321-1323,1327-1329`, `PrintConfig.cpp:2776-2826`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, material database behavior, soluble/support runtime behavior, extruder-printability behavior, UI behavior, slicing behavior, extrusion behavior, statistics behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `filament_density`, `filament_type`, `filament_soluble`, `filament_change_length`, `filament_is_support`, and `filament_printable` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- If sorted placement would push `pre_middle.rs` past 400 LOC, the existing table shards are adjusted without changing lookup behavior.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/tooltip/sidetext/min/gui/mode/enum-values metadata remains deferred beyond the current metadata boundary.
- Material type database population, soluble/support behavior, filament change-length behavior, extruder printability behavior, statistics behavior, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `temperature_vitrification`, `filament_cost`, `filament_settings_id`, and following options from `PrintConfig.cpp:2828+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
