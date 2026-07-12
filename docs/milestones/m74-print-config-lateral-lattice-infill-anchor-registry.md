# M74: PrintConfig lateral lattice and infill anchor option registry

## Goal
Port the adjacent FFF lateral lattice angle, infill overhang angle, and infill anchor option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2987-3066` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1103-1105,1195-1196`, `PrintConfig.cpp:2987-3066`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, lateral lattice behavior, infill overhang behavior, infill anchor behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `lateral_lattice_angle_1`, `lateral_lattice_angle_2`, `infill_overhang_angle`, `infill_anchor`, and `infill_anchor_max` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/category/tooltip/sidetext/min/max/mode/ratio/gui/enum metadata remains deferred beyond the current metadata boundary.
- Lateral lattice behavior, infill overhang behavior, infill anchor behavior, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `inner_wall_acceleration`, `travel_acceleration`, and following options from `PrintConfig.cpp:3068+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
