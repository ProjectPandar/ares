# M137 Spec: PrintConfig wipe-tower angle and brim registry slice

## Goal
Port the adjacent wipe-tower rotation, prime-tower brim width, and wipe-tower cone angle option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1581`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6718-6723`: `wipe_tower_rotation_angle` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1582`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6725-6734`: `prime_tower_brim_width` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1594`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6736-6744`: `wipe_tower_cone_angle` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/sidetext/min/max/mode/gui_type/enum metadata beyond the current registry metadata boundary.
- Wipe-tower rotation behavior, prime-tower brim width use, auto brim-width calculation, cone stabilization behavior, prime tower generation, slicing behavior, geometry behavior, extrusion behavior, and G-code behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6746+`: `wipe_tower_max_purge_speed`, `wipe_tower_wall_type`, and following options.
- Filesystem behavior, network behavior, UI behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/late_tail_final.rs`: add `prime_tower_brim_width` in lexicographic order after `pressure_advance` and before `prime_tower_enable_framework`.
- `crates/ares-core/src/options/registry/definitions/table/tail_terminal_suffix.rs`: add `wipe_tower_cone_angle` after `wipe_speed` and before `wipe_tower_no_sparse_layers`; add `wipe_tower_rotation_angle` after `wipe_tower_no_sparse_layers` and before `wipe_tower_type`.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add covered expected keys in sorted positions.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/wipe_tower_angle_brim.rs`: add metadata assertions for all three definitions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_wipe_tower_angle_brim.rs`: add public lookup assertions for all three definitions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/tail_values.rs`: add known-count fixture values for all three covered keys.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by three.
- `docs/roadmap.md` and `docs/milestones/m137-print-config-wipe-tower-angle-brim-registry.md`: milestone sequencing docs.

## Included option definitions

- `wipe_tower_rotation_angle` (`coFloat`, default `0`, field at `PrintConfig.hpp:1581`, definition lines 6718-6723, Ares kind `Float`)
- `prime_tower_brim_width` (`coFloat`, default `3`, field at `PrintConfig.hpp:1582`, definition lines 6725-6734, Ares kind `Float`)
- `wipe_tower_cone_angle` (`coFloat`, default `30`, field at `PrintConfig.hpp:1594`, definition lines 6736-6744, Ares kind `Float`)

## Functional requirements

1. Add the three missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, wipe-tower rotation behavior, prime-tower brim behavior, cone behavior, prime tower behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `wipe_tower_max_purge_speed`, `wipe_tower_wall_type`, or following options from `PrintConfig.cpp:6746+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove all three covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all three covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/geometry/extrusion/G-code behavior, and following `PrintConfig.cpp:6746+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
