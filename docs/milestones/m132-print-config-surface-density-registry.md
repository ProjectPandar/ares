# M132: PrintConfig surface-density registry

## Goal
Port the adjacent top and bottom surface-density option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6586-6607` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1088-1089`, `PrintConfig.cpp:6586-6607`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, top/bottom surface density behavior, surface pattern behavior, extrusion planning behavior, UI behavior, slicing behavior, geometry behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `bottom_surface_density` and `top_surface_density` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for top/bottom surface density, surface pattern application, slicing, geometry, extrusion planning, and downstream G-code remains unchanged/deferred.
- `travel_speed`, `travel_speed_z`, and following options from `PrintConfig.cpp:6610+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
