# M125: PrintConfig tree support diameter, wall, and infill registry

## Goal
Port the adjacent tree-support branch diameter, branch diameter angle, organic branch diameter, wall count, and tree-support-with-infill option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6356-6404` into `ares-core` registry metadata.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `PrintConfig.hpp:1010`, `PrintConfig.hpp:1012`, `PrintConfig.hpp:1014`, `PrintConfig.hpp:1036`, `PrintConfig.cpp:6356-6404`, and the current option registry metadata boundary; no new Ares pipeline, crate, dependency, tree-support generation behavior, support geometry, UI behavior, slicing behavior, extrusion behavior, or G-code behavior is added.

## Exit checklist
- `OPTION_DEFINITIONS` includes `tree_support_branch_diameter`, `tree_support_branch_diameter_angle`, `tree_support_branch_diameter_organic`, `tree_support_wall_count`, and `tree_support_with_infill` with exact kinds, defaults, and source line ranges.
- `OPTION_DEFINITIONS` remains a single sorted slice for `option_definition()` binary-search lookup.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists without growing any modified Rust file to 400 LOC or above.
- Runtime behavior for tree support generation, branch-diameter tapering, wall-loop generation, support infill generation, support geometry, slicing, extrusion, and downstream G-code remains unchanged/deferred.
- `support_ironing`, `support_ironing_pattern`, `support_ironing_flow`, `support_ironing_spacing`, and following support-ironing options from `PrintConfig.cpp:6406+` remain unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
