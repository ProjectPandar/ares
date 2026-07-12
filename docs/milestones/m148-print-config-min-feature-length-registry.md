# M148: PrintConfig minimum feature and wall-length registry

## Goal
Port the adjacent minimum feature and minimum wall-length option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7051-7074` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1025`, `PrintConfig.hpp:1039`, `PrintConfig.cpp:7051-7074`, and the current option registry metadata boundary. No new Ares pipeline, crate, dependency, thin-feature widening behavior, wall pruning behavior, Arachne/classic perimeter behavior, UI behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code writer behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `min_feature_size` and `min_length_factor` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for minimum feature filtering/widening, short wall pruning, Arachne/classic perimeter generation, slicing, geometry, extrusion planning, and downstream G-code remains unchanged/deferred.
- `wall_maximum_resolution`, `wall_maximum_deviation`, `initial_layer_min_bead_width`, `min_bead_width`, and following options from `PrintConfig.cpp:7076+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
