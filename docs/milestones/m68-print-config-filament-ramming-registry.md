# M68: PrintConfig filament ramming option registry

## Goal
Port the adjacent FFF filament ramming parameters and multitool ramming option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2745-2774` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1451-1454`, `PrintConfig.cpp:2745-2774`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, ramming editor/runtime, wipe-tower behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `filament_ramming_parameters`, `filament_multitool_ramming`, `filament_multitool_ramming_volume`, and `filament_multitool_ramming_flow` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/tooltip/sidetext/min/max/mode metadata remains deferred beyond the current metadata boundary.
- Ramming parameter parsing/editing/runtime, multitool ramming behavior, wipe-tower behavior, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `filament_density`, `filament_type`, and following options from `PrintConfig.cpp:2776+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
