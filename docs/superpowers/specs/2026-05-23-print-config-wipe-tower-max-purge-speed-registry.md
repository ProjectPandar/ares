# M138 Spec: PrintConfig wipe-tower max purge speed registry slice

## Goal
Port the wipe-tower maximum purge speed option definition from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1596`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6746-6757`: `wipe_tower_max_purge_speed` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/sidetext/min/mode metadata beyond the current registry metadata boundary.
- Wipe-tower purge-speed selection, sparse-layer speed fallback, filament max-volumetric-speed comparison, tower stability behavior, prime tower generation, slicing behavior, geometry behavior, extrusion behavior, and G-code behavior.
- Typed accessors or behavior changes for the newly registered key.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6759+`: `wipe_tower_wall_type`, `wipe_tower_extra_rib_length`, and following options.
- Filesystem behavior, network behavior, UI behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/tail_terminal_suffix.rs`: add `wipe_tower_max_purge_speed` in lexicographic order after `wipe_tower_cone_angle` and before `wipe_tower_no_sparse_layers`.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add the covered expected key in sorted position.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/wipe_tower_max_purge_speed.rs`: add metadata assertions for the covered definition.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_wipe_tower_max_purge_speed.rs`: add public lookup assertions for the covered definition.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/tail_values.rs`: add known-count fixture value for the covered key.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by one.
- `docs/roadmap.md` and `docs/milestones/m138-print-config-wipe-tower-max-purge-speed-registry.md`: milestone sequencing docs.

## Included option definitions

- `wipe_tower_max_purge_speed` (`coFloat`, default `90`, field at `PrintConfig.hpp:1596`, definition lines 6746-6757, Ares kind `Float`)

## Functional requirements

1. Add the missing option using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, wipe-tower purge-speed behavior, sparse-layer speed behavior, volumetric-speed behavior, prime tower behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code behavior for this option in this milestone.
6. Do not add `wipe_tower_wall_type`, `wipe_tower_extra_rib_length`, or following options from `PrintConfig.cpp:6759+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the covered key has expected kind, default value, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered key while preserving unknown options.
- Public lookup coverage exists for the covered definition.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/geometry/extrusion/G-code behavior, and following `PrintConfig.cpp:6759+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
