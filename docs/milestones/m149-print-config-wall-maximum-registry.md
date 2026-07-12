# M149: PrintConfig wall maximum resolution registry

## Goal
Port the adjacent wall maximum resolution/deviation option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7076-7097` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1030-1031`, `PrintConfig.cpp:7076-7097`, and the current option registry metadata boundary. No new Ares pipeline, crate, dependency, wall resolution simplification behavior, deviation-limited path reduction, Arachne/classic perimeter behavior, UI behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `wall_maximum_resolution` and `wall_maximum_deviation` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for wall path simplification, maximum deviation enforcement, Arachne/classic perimeter generation, slicing, geometry, extrusion planning, and downstream G-code remains unchanged/deferred.
- `initial_layer_min_bead_width`, `min_bead_width`, and following options from `PrintConfig.cpp:7099+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
