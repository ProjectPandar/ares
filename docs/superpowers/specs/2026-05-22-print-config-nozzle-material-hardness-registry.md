# M83 Spec: PrintConfig nozzle material and hardness registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` nozzle material/hardness option-definition slice into `ares-core` option registry metadata by adding registry coverage for `nozzle_type` and `nozzle_hrc`.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/CommonDefs.hpp:12-20`, `OrcaSlicer/src/libslic3r/PrintConfig.hpp:338-353`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:485-492`, `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1402`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3652-3669`: `nozzle_type` enum metadata and option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1403`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3672-3679`: `nozzle_hrc` option definition.

Related upstream behavior explicitly deferred:

- UI label/tooltip/sidetext/min/max/mode/nullable/enum label metadata beyond the current registry boundary, except nullable enum-vector type identity.
- Nozzle material compatibility checks and nozzle hardness validation during slicing.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3681+`: `printer_structure` and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry.rs`: add metadata-only `OptionValueKind::EnumsNullable` for upstream `coEnums` plus nullable generic enum-vector metadata.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: add sorted definitions for `nozzle_hrc` and `nozzle_type`.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata.rs`: add `mod nozzle;`.
- `crates/ares-core/src/options/registry/tests/metadata/nozzle.rs`: source metadata assertions for both options.
- `crates/ares-core/src/options/tests.rs`: add `mod registry_lookup_nozzle;`.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_nozzle.rs`: public lookup coverage for both options.
- `docs/roadmap.md` and `docs/milestones/m83-print-config-nozzle-material-hardness-registry.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `nozzle_type` (`coEnums`, `nullable = true`, `ConfigOptionEnumsGenericNullable({ ntUndefine })` default, field at `PrintConfig.hpp:1402`, enum at `CommonDefs.hpp:12-20`, enum string maps at `PrintConfig.hpp:338-353`, enum map at `PrintConfig.cpp:485-492`, definition lines 3652-3669, Ares kind `EnumsNullable`, default string `undefine`)
- `nozzle_hrc` (`coInt`, default `0`, field at `PrintConfig.hpp:1403`, definition lines 3672-3679, Ares kind `Int`)

## Functional requirements

1. Add metadata-only `OptionValueKind::EnumsNullable`; do not add parsing/runtime behavior for it in this milestone.
2. Add the included missing options to existing sorted definition shards using `EnumsNullable` and `Int`.
3. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
4. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, nozzle material compatibility behavior, nozzle hardness validation, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter `printer_structure` or following options outside the included slice.
8. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
10. Keep modified Rust files under 400 LOC; create focused test files instead of growing existing near-limit files.

## Deferred behavior

- Upstream UI and validation metadata from `PrintConfig.cpp:3652-3679` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation; nullable enum-vector identity is preserved through `EnumsNullable` only.
- Nozzle material compatibility checks, nozzle hardness validation, typed accessors, slicing, extrusion, and G-code behavior are deferred to later source-cited milestones.
- `printer_structure` and following options from `PrintConfig.cpp:3681+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove both new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for both new keys.
- Plan/spec explicitly account for `EnumsNullable`, deferred UI/bounds/enum-label metadata, runtime behavior, slicing/extrusion/G-code behavior, and following `printer_structure` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
