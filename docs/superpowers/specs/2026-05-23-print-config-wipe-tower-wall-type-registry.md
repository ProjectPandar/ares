# M139 Spec: PrintConfig wipe-tower wall type registry slice

## Goal
Port the wipe-tower wall type enum option definition from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:405-408`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:558-563`: `WipeTowerWallType` enum and key map (`rectangle`, `cone`, `rib`).
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1597`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6759-6773`: `wipe_tower_wall_type` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/enum labels/mode metadata beyond the current registry metadata boundary.
- Runtime wall-shape selection, rectangle/cone/rib tower geometry, cone stabilization, rib generation, fillet wall behavior, prime tower generation, slicing behavior, geometry behavior, extrusion behavior, and G-code behavior.
- Typed accessors or behavior changes for the newly registered key.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6775+`: `wipe_tower_extra_rib_length`, `wipe_tower_rib_width`, `wipe_tower_fillet_wall`, `wipe_tower_filament`, and following options.
- Filesystem behavior, network behavior, UI behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/tail_terminal_suffix.rs`: add `wipe_tower_wall_type` in lexicographic order after `wipe_tower_type` and before `wipe_tower_x`.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add the covered expected key in sorted position.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/wipe_tower_wall_type.rs`: add metadata assertions for the covered definition.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_wipe_tower_wall_type.rs`: add public lookup assertions for the covered definition.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/tail_values.rs`: add known-count fixture value for the covered key.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by one.
- `docs/roadmap.md` and `docs/milestones/m139-print-config-wipe-tower-wall-type-registry.md`: milestone sequencing docs.

## Included option definitions

- `wipe_tower_wall_type` (`coEnum`, enum values `rectangle`, `cone`, `rib`, default `rib`, field at `PrintConfig.hpp:1597`, definition lines 6759-6773, Ares kind `Enum`)

## Functional requirements

1. Add the missing option using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, wall-shape behavior, cone behavior, rib behavior, fillet behavior, prime tower behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code behavior for this option in this milestone.
6. Do not add `wipe_tower_extra_rib_length`, `wipe_tower_rib_width`, `wipe_tower_fillet_wall`, `wipe_tower_filament`, or following options from `PrintConfig.cpp:6775+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the covered key has expected kind, default value, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered key while preserving unknown options.
- Public lookup coverage exists for the covered definition.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/geometry/extrusion/G-code behavior, and following `PrintConfig.cpp:6775+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
