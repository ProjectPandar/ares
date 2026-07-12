# M73: PrintConfig gyroid optimization and sparse infill pattern option registry

## Goal
Port the adjacent FFF gyroid optimization and sparse infill pattern option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2915-2985` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1102,1136`, `PrintConfig.cpp:2915-2985`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, gyroid optimization behavior, sparse infill pattern runtime behavior, enum database expansion beyond default metadata, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `gyroid_optimized` and `sparse_infill_pattern` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Upstream label/category/tooltip/enum values/enum labels metadata remains deferred beyond the current metadata boundary.
- Gyroid optimization behavior, sparse infill pattern runtime behavior, UI behavior, slicing behavior, extrusion behavior, and downstream G-code behavior remain deferred.
- `lateral_lattice_angle_1`, `lateral_lattice_angle_2`, `infill_overhang_angle`, and following options from `PrintConfig.cpp:2987+` remain unchanged/deferred.
- Modified Rust files remain under 400 LOC.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
