# M124 Spec: PrintConfig tree support branch and tip registry slice

## Goal
Port the adjacent tree-support branch angle, preferred angle, branch distance, branch density, auto brim, brim width, and tip diameter option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1011`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6264-6273`: `tree_support_branch_angle` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1037`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6275-6284`: `tree_support_branch_angle_organic` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1013`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6286-6296`: `tree_support_angle_slow` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1008`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6298-6306`: `tree_support_branch_distance` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1034`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6308-6316`: `tree_support_branch_distance_organic` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1035`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6318-6330`: `tree_support_top_rate` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1015`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6332-6336`: `tree_support_auto_brim` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1016`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6338-6343`: `tree_support_brim_width` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1009`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6345-6354`: `tree_support_tip_diameter` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/categories/sidetext/minimums/maximums/modes beyond the current registry metadata boundary.
- Tree-support generation, organic support branch routing, branch-density behavior, auto-brim width calculation, tree support brim geometry, support geometry, and support path generation.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6356+`: `tree_support_branch_diameter`, `tree_support_branch_diameter_organic`, `tree_support_branch_diameter_angle`, `tree_support_wall_count`, and following tree-support options.
- Slicing, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/tail_terminal_suffix.rs`: add the nine covered `tree_support_*` definitions after `travel_speed` and before `upward_compatible_machine` in sorted order.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add the nine covered expected keys after `travel_speed` and before `upward_compatible_machine`.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/tree_support_branch_tip.rs`: add metadata assertions for all nine definitions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_tree_support_branch_tip.rs`: add public lookup assertions for all nine definitions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs` and `crates/ares-core/src/options/tests/registry_helpers/known_count/values/tail_values.rs`: add known-count fixture values and expected counts without touching near-limit `values.rs`.
- `docs/roadmap.md` and `docs/milestones/m124-print-config-tree-support-branch-tip-registry.md`: milestone sequencing docs.

## Included option definitions

- `tree_support_branch_angle` (`coFloat`, default `40`, field at `PrintConfig.hpp:1011`, definition lines 6264-6273, Ares kind `Float`)
- `tree_support_branch_angle_organic` (`coFloat`, default `40`, field at `PrintConfig.hpp:1037`, definition lines 6275-6284, Ares kind `Float`)
- `tree_support_angle_slow` (`coFloat`, default `25`, field at `PrintConfig.hpp:1013`, definition lines 6286-6296, Ares kind `Float`)
- `tree_support_branch_distance` (`coFloat`, default `5`, field at `PrintConfig.hpp:1008`, definition lines 6298-6306, Ares kind `Float`)
- `tree_support_branch_distance_organic` (`coFloat`, default `1`, field at `PrintConfig.hpp:1034`, definition lines 6308-6316, Ares kind `Float`)
- `tree_support_top_rate` (`coPercent`, default `30`, field at `PrintConfig.hpp:1035`, definition lines 6318-6330, Ares kind `Percent`)
- `tree_support_auto_brim` (`coBool`, default `true`, field at `PrintConfig.hpp:1015`, definition lines 6332-6336, Ares kind `Bool`)
- `tree_support_brim_width` (`coFloat`, default `3`, field at `PrintConfig.hpp:1016`, definition lines 6338-6343, Ares kind `Float`)
- `tree_support_tip_diameter` (`coFloat`, default `0.8`, field at `PrintConfig.hpp:1009`, definition lines 6345-6354, Ares kind `Float`)

## Functional requirements

1. Add the nine missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, tree support generation behavior, organic branch routing, auto-brim calculation, support geometry, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `tree_support_branch_diameter`, `tree_support_branch_diameter_organic`, `tree_support_branch_diameter_angle`, `tree_support_wall_count`, or following options from `PrintConfig.cpp:6356+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the nine covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all nine covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:6356+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
