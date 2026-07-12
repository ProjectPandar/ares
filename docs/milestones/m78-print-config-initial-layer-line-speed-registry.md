# M78: PrintConfig initial-layer line, speed, and slow-down registry

## Goal
Port the adjacent FFF initial-layer line width, print height, speed, travel speed, and slow-down option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3251-3314` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1421,1527-1529,1532,1627`, `PrintConfig.cpp:3251-3314`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, initial-layer runtime behavior, speed/ratio resolution behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `initial_layer_line_width`, `initial_layer_print_height`, `initial_layer_speed`, `initial_layer_infill_speed`, `initial_layer_travel_speed`, and `slow_down_layers` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/category/tooltip/sidetext/min/max/max_literal/mode/ratio metadata remains deferred beyond the current metadata boundary.
- Initial-layer line-width resolution, print-height behavior, speed behavior, travel-speed ratio behavior, slow-down behavior, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `nozzle_temperature_initial_layer`, `full_fan_speed_layer`, support/internal bridge fan speed, and following options from `PrintConfig.cpp:3316+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
