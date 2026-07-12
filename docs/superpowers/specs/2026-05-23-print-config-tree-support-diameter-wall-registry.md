# M125 Spec: PrintConfig tree support diameter, wall, and infill registry slice

## Goal
Port the adjacent tree-support branch diameter, branch diameter angle, organic branch diameter, wall count, and tree-support-with-infill option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1010`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6356-6364`: `tree_support_branch_diameter` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1012`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6366-6378`: `tree_support_branch_diameter_angle` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1036`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6380-6388`: `tree_support_branch_diameter_organic` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1014`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6390-6397`: `tree_support_wall_count` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6399-6404`: `tree_support_with_infill` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/categories/sidetext/minimums/maximums/modes beyond the current registry metadata boundary.
- Tree-support generation, branch-diameter tapering, organic support branch routing, wall-loop generation, tree support infill generation, support geometry, and support path generation.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6406+`: `support_ironing`, `support_ironing_pattern`, `support_ironing_flow`, `support_ironing_spacing`, and following support-ironing options.
- Slicing, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/tail_terminal_suffix.rs`: add `tree_support_branch_diameter`, `tree_support_branch_diameter_angle`, and `tree_support_branch_diameter_organic` after `tree_support_branch_angle_organic` and before `tree_support_branch_distance`; add `tree_support_wall_count` and `tree_support_with_infill` after `tree_support_top_rate` and before `upward_compatible_machine`.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add the five covered expected keys in the same sorted positions among the existing M124 `tree_support_*` keys.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/tree_support_diameter_wall.rs`: add metadata assertions for all five definitions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_tree_support_diameter_wall.rs`: add public lookup assertions for all five definitions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs` and `crates/ares-core/src/options/tests/registry_helpers/known_count/values/tail_values.rs`: add known-count fixture values and expected counts without touching near-limit `values.rs`.
- `docs/roadmap.md` and `docs/milestones/m125-print-config-tree-support-diameter-wall-registry.md`: milestone sequencing docs.

## Included option definitions

- `tree_support_branch_diameter` (`coFloat`, default `5`, field at `PrintConfig.hpp:1010`, definition lines 6356-6364, Ares kind `Float`)
- `tree_support_branch_diameter_angle` (`coFloat`, default `5`, field at `PrintConfig.hpp:1012`, definition lines 6366-6378, Ares kind `Float`)
- `tree_support_branch_diameter_organic` (`coFloat`, default `2`, field at `PrintConfig.hpp:1036`, definition lines 6380-6388, Ares kind `Float`)
- `tree_support_wall_count` (`coInt`, default `0`, field at `PrintConfig.hpp:1014`, definition lines 6390-6397, Ares kind `Int`)
- `tree_support_with_infill` (`coBool`, default `false`, definition lines 6399-6404, Ares kind `Bool`)

## Functional requirements

1. Add the five missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, tree support generation behavior, branch-diameter behavior, wall-loop behavior, tree support infill behavior, support geometry, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `support_ironing`, `support_ironing_pattern`, `support_ironing_flow`, `support_ironing_spacing`, or following options from `PrintConfig.cpp:6406+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the five covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all five covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:6406+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
