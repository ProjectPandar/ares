# M86 Spec: PrintConfig G-code flavor and object-label registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` G-code flavor, pellet-printer, multi-bed, object-label, exclude-object, and verbose-G-code option-definition slice into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:33-46`, `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1355`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:161-176`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3785-3817`: `gcode_flavor` enum metadata and option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3819-3823`: `pellet_modded_printer` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1461`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3825-3829`: `support_multi_bed_types` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1623`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3831-3837`: `gcode_label_objects` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1624`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3839-3843`: `exclude_object` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1626`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3845-3851`: `gcode_comments` option definition.

Related upstream behavior explicitly deferred:

- UI label/tooltip/mode/readonly/enum-label metadata beyond the current registry boundary.
- G-code flavor typed enum behavior and flavor-dependent G-code generation.
- Pellet-printer behavior and pellet flow conversion behavior.
- Multi-bed UI/runtime behavior.
- Object label and exclude-object G-code command emission.
- Verbose G-code comment emission.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3853+`: `infill_combination` and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`: add sorted definition for `exclude_object`.
- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: add sorted definitions for `gcode_comments`, `gcode_flavor`, and `gcode_label_objects`.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: add sorted definition for `pellet_modded_printer`.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add sorted definition for `support_multi_bed_types`.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata.rs`: add `mod gcode;`.
- `crates/ares-core/src/options/registry/tests/metadata/gcode.rs`: source metadata assertions for all six options.
- `crates/ares-core/src/options/tests.rs`: add `mod registry_lookup_gcode;`.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_gcode.rs`: public lookup coverage for all six options.
- `docs/roadmap.md` and `docs/milestones/m86-print-config-gcode-flavor-object-label-registry.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `gcode_flavor` (`coEnum`, default `gcfMarlinLegacy`, field at `PrintConfig.hpp:1355`, enum at `PrintConfig.hpp:33-46`, enum map at `PrintConfig.cpp:161-176`, definition lines 3785-3817, Ares kind `Enum`, default string `marlin`)
- `pellet_modded_printer` (`coBool`, default `false`, definition lines 3819-3823, Ares kind `Bool`)
- `support_multi_bed_types` (`coBool`, default `false`, field at `PrintConfig.hpp:1461`, definition lines 3825-3829, Ares kind `Bool`)
- `gcode_label_objects` (`coBool`, default `true`, field at `PrintConfig.hpp:1623`, definition lines 3831-3837, Ares kind `Bool`)
- `exclude_object` (`coBool`, default `false`, field at `PrintConfig.hpp:1624`, definition lines 3839-3843, Ares kind `Bool`)
- `gcode_comments` (`coBool`, default `false`, field at `PrintConfig.hpp:1626`, definition lines 3845-3851, Ares kind `Bool`)

## Functional requirements

1. Add the included missing options to existing sorted definition shards using existing value kinds only.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, G-code flavor behavior, pellet-printer behavior, multi-bed behavior, object-label/exclude-object behavior, verbose-comment behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add or alter `infill_combination` or following options outside the included slice.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Keep modified Rust files under 400 LOC; create focused test files instead of growing existing near-limit files.

## Deferred behavior

- Upstream UI and validation metadata from `PrintConfig.cpp:3785-3851` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- G-code flavor-dependent output, object labels, exclude-object commands, verbose comments, pellet-printer behavior, multi-bed behavior, typed accessors, slicing, extrusion, and G-code behavior are deferred to later source-cited milestones.
- `infill_combination` and following options from `PrintConfig.cpp:3853+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all six new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all six new keys.
- Plan/spec explicitly account for deferred UI metadata, runtime behavior, slicing/extrusion/G-code behavior, and following `infill_combination` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
