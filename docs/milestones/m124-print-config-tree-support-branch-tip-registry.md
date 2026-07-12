# M124: PrintConfig tree support branch and tip registry

## Goal
Port the adjacent tree-support branch angle, preferred angle, branch distance, branch density, auto brim, brim width, and tip diameter option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6264-6354` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1008-1009`, `PrintConfig.hpp:1011`, `PrintConfig.hpp:1013`, `PrintConfig.hpp:1015-1016`, `PrintConfig.hpp:1034-1035`, `PrintConfig.hpp:1037`, `PrintConfig.cpp:6264-6354`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, tree-support generation behavior, support geometry, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `tree_support_branch_angle`, `tree_support_branch_angle_organic`, `tree_support_angle_slow`, `tree_support_branch_distance`, `tree_support_branch_distance_organic`, `tree_support_top_rate`, `tree_support_auto_brim`, `tree_support_brim_width`, and `tree_support_tip_diameter` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for tree support generation, organic support branch routing, branch density application, brim generation, support geometry, slicing, extrusion, and downstream G-code remains unchanged/deferred.
- `tree_support_branch_diameter`, `tree_support_branch_diameter_organic`, `tree_support_branch_diameter_angle`, `tree_support_wall_count`, and following options from `PrintConfig.cpp:6356+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
