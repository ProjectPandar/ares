# M123 Spec: PrintConfig support independent layer height and threshold registry slice

## Goal
Port the adjacent independent support layer height, support threshold angle, and support threshold overlap option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1618`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6232-6238`: `independent_support_layer_height` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:993`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6240-6251`: `support_threshold_angle` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:994`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6253-6262`: `support_threshold_overlap` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/categories/sidetext/minimums/maximums/modes beyond the current registry metadata boundary.
- Independent support layer-height behavior, prime-tower invalidation behavior, support threshold angle/overlap behavior, support generation, overhang detection, support geometry, and support path generation.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6264+`: `tree_support_branch_angle`, `tree_support_branch_angle_organic`, and following tree-support options.
- Slicing, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: mechanically move the existing `infill_*` through `input_shaping_*` tail block out of this near-400-LOC shard.
- `crates/ares-core/src/options/registry/definitions/table/middle_independent.rs`: create a new shard merged after `middle.rs` and before `middle_tail.rs`; add `independent_support_layer_height` after `hot_plate_temp_initial_layer` and before the moved `infill_anchor` entry, then preserve the moved `infill_*`, `inherits*`, `initial_layer_*`, `inner_wall_*`, and `input_shaping_*` order.
- `crates/ares-core/src/options/registry/definitions/table/tail_support.rs`: add the two covered `support_threshold_*` definitions after `support_style` and before `support_top_z_distance` in sorted order.
- Registry key, metadata, fixture-count, and public lookup tests cover all three definitions; table assembly includes the new `middle_independent` shard without changing lookup APIs.
- `docs/roadmap.md` and `docs/milestones/m123-print-config-support-threshold-layer-height-registry.md`: milestone sequencing docs.

## Included option definitions

- `independent_support_layer_height` (`coBool`, default `true`, field at `PrintConfig.hpp:1618`, definition lines 6232-6238, Ares kind `Bool`)
- `support_threshold_angle` (`coInt`, default `30`, field at `PrintConfig.hpp:993`, definition lines 6240-6251, Ares kind `Int`)
- `support_threshold_overlap` (`coFloatOrPercent`, default `50%`, field at `PrintConfig.hpp:994`, definition lines 6253-6262, Ares kind `FloatOrPercent`)

## Functional requirements

1. Add the three missing options using existing value kinds only; use a mechanical registry shard split only to preserve global sorted order and the 400 LOC rule.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup, including shard merge order after the mechanical split.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, independent support layer-height behavior, support threshold behavior, support geometry, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `tree_support_branch_angle`, `tree_support_branch_angle_organic`, or following options from `PrintConfig.cpp:6264+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the three covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible after adding `middle_independent` between `middle` and `middle_tail`.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all three covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:6264+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
