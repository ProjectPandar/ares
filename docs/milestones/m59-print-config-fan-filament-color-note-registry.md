# M59: PrintConfig fan cooling and filament color note option registry

## Goal
Port the adjacent FFF fan cooling layer time, default filament color, filament color, and filament notes option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2349-2382` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1325,1331,1521,1632`, `PrintConfig.cpp:2349-2382`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, fan runtime behavior, color UI behavior, note UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `fan_cooling_layer_time`, `default_filament_colour`, `filament_colour`, and `filament_notes` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/tooltip/sidetext/min/max/mode/gui/multiline/full-width/height behavior remains deferred because the current registry boundary stores only key, kind, default, and source citation.
- Fan runtime behavior, color UI behavior, note UI behavior, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `filament_multi_colour`, `filament_colour_type`, and following options from `PrintConfig.cpp:2385+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
