# M152 Spec: PrintConfig narrow internal solid infill registry slice

## Goal
Port the `detect_narrow_internal_solid_infill` option definition from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1017`: `detect_narrow_internal_solid_infill` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7154-7161`: `detect_narrow_internal_solid_infill` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/category/mode metadata beyond the current registry metadata boundary.
- Narrow internal solid infill area detection.
- Concentric versus rectilinear internal solid infill pattern selection.
- Typed accessors or behavior changes for the newly registered key.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7164+`: `PrintConfigDef::init_extruder_option_keys` behavior.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`: add `detect_narrow_internal_solid_infill` after `deretraction_speed` and before `detect_overhang_wall`.
- `crates/ares-core/src/options/registry/tests/keys/first.rs`: add `detect_narrow_internal_solid_infill` after `deretraction_speed` and before `detect_overhang_wall`.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/narrow_internal_solid_infill.rs`: add metadata assertions for the definition.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_narrow_internal_solid_infill.rs`: add public lookup assertions for the definition.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values.rs` and a new `process_values` shard: add fixtures while keeping `values.rs` below 400 LOC.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by one.
- `docs/roadmap.md` and `docs/milestones/m152-print-config-narrow-internal-solid-infill-registry.md`: milestone sequencing docs.

## Included option definition

- `detect_narrow_internal_solid_infill` (`coBool`, default `true`, field at `PrintConfig.hpp:1017`, definition lines 7154-7161, Ares kind `Bool`)

## Functional requirements

1. Add the missing option using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, narrow internal solid infill detection, infill pattern selection, slicing behavior, extrusion behavior, or G-code behavior for this option in this milestone.
6. Do not add `PrintConfigDef::init_extruder_option_keys` behavior from `PrintConfig.cpp:7164+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC; split known-count fixtures as needed.

## Acceptance checks

- Registry tests prove the covered key has expected kind, default value, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered key while preserving unknown options.
- Public lookup coverage exists for the covered definition.
- Plan/spec explicitly account for deferred runtime narrow-infill behavior and `PrintConfig.cpp:7164+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
