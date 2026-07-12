# M91: PrintConfig ironing and Z contouring option registry

## Goal
Port the adjacent FFF ironing and Z-layer anti-aliasing / Z contouring option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4161-4293` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:87-98,100-106,1138-1146,1237-1240`, `PrintConfig.cpp:225-263,4161-4293`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, ironing behavior, Z contouring behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `ironing_type`, `ironing_pattern`, `ironing_flow`, `ironing_spacing`, `ironing_inset`, `ironing_speed`, `ironing_angle`, `ironing_angle_fixed`, `ironing_expansion`, `zaa_enabled`, `zaa_minimize_perimeter_height`, `zaa_dont_alternate_fill_direction`, and `zaa_min_z` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream UI label/tooltip/category/sidetext/min/max/mode/enum-label metadata remains deferred beyond the current metadata boundary.
- Runtime behavior for ironing, ironing pattern generation, Z contouring, fill-direction alternation, slicing-plane changes, extrusion, and downstream G-code behavior remains deferred.
- `layer_change_gcode` and following options from `PrintConfig.cpp:4295+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
