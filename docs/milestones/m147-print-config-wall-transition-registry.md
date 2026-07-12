# M147: PrintConfig wall-transition registry

## Goal
Port the adjacent wall-transition option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7003-7049` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1021-1024`, `PrintConfig.cpp:7003-7049`, and the current option registry metadata boundary. No new Ares pipeline, crate, dependency, Arachne/classic perimeter behavior, wall-transition geometry behavior, UI behavior, slicing behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `wall_transition_length`, `wall_transition_filter_deviation`, `wall_transition_angle`, and `wall_distribution_count` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for Arachne/classic perimeter generation, wall transition planning, slicing, geometry, extrusion planning, and downstream G-code remains unchanged/deferred.
- `min_feature_size`, `min_length_factor`, `wall_maximum_resolution`, and following options from `PrintConfig.cpp:7051+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
