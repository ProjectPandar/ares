# M123: PrintConfig support independent layer height and threshold registry

## Goal
Port the adjacent independent support layer height, support threshold angle, and support threshold overlap option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6232-6262` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:993-994`, `PrintConfig.hpp:1618`, `PrintConfig.cpp:6232-6262`, and the current option registry metadata boundary; a mechanical registry shard split is allowed only to preserve sorted metadata and LOC limits; no new Ares pipeline, crate, dependency, independent support layer-height behavior, support threshold behavior, support geometry, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `independent_support_layer_height`, `support_threshold_angle`, and `support_threshold_overlap` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup after any mechanical registry shard split.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file beyond 400 LOC.
- Runtime behavior for independent support layer heights, support threshold angle/overlap, support generation, support geometry, slicing, extrusion, and downstream G-code remains unchanged/deferred.
- `tree_support_branch_angle`, `tree_support_branch_angle_organic`, and following tree-support options from `PrintConfig.cpp:6264+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
