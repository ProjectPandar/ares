# M168 Spec: PrintConfig SLA material speed registry slice

## Goal
Port the SLA material print speed setting from `libslic3r::PrintConfigDef::init_sla_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1805`, `PrintConfig.hpp:1821`, `PrintConfig.cpp:413-417`, `PrintConfig.cpp:7855-7864`: `SLAMaterialSpeed` enum mapping and `material_print_speed` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/enum labels/mode metadata beyond the current registry metadata boundary.
- SLA material-speed runtime behavior, exposure-speed behavior, and SL1 export profile selection.
- Typed accessors or behavior changes for the newly registered key.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7867+`: legacy option handling and later non-`init_sla_params` behavior.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/late_tail_material.rs`: add `material_print_speed` in lexicographic order after `material_density` and before `material_type`.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add `material_print_speed` in lexicographic order after `material_density` and before `material_type`.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/sla_material_speed.rs`: add metadata assertions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_sla_material_speed.rs`: add public lookup assertions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/sla_display_tilt_values.rs`: add a fixture value for the covered key near the existing SLA material values.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by 1.
- `docs/roadmap.md` and `docs/milestones/m168-print-config-sla-material-speed-registry.md`: milestone sequencing docs.

## Included option definition

- `material_print_speed` (`coEnum`, `SLAMaterialSpeed`, enum keys `slow`/`fast`, default `slamsFast` / `fast`, enum declaration at `PrintConfig.hpp:1805`, field at `PrintConfig.hpp:1821`, enum map at `PrintConfig.cpp:413-417`, definition lines 7855-7864, Ares kind `Enum`)

## Explicit non-included adjacent behavior

- `PrintConfigDef::handle_legacy` beginning at `PrintConfig.cpp:7867` is deferred.
- `OrcaSlicer/src/libslic3r/Format/SL1.cpp:385` usage of `material_print_speed` for `expUserProfile` is deferred to a source-cited SL1 export/runtime milestone.
- UI enum labels/mode metadata remains deferred until the registry metadata boundary expands.

## Functional requirements

1. Add the missing option using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, SLA material-speed runtime behavior, SL1 export behavior, slicing behavior, extrusion behavior, or G-code behavior for this option in this milestone.
6. Do not add legacy option handling from `PrintConfig.cpp:7867+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC; current destination shards have enough room and should not require a split.

## Acceptance checks

- Registry tests prove the covered key has expected kind, default value, enum/source line references, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered key while preserving unknown options.
- Public lookup coverage exists for the covered definition.
- Plan/spec explicitly account for deferred SL1/export/runtime behavior and `PrintConfig.cpp:7867+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
