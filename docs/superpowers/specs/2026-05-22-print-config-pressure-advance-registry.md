# M56 Spec: PrintConfig pressure advance option registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` pressure advance option-definition slice into `ares-core` option registry metadata by adding registry coverage for `enable_pressure_advance` and `pressure_advance`.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1302`: `enable_pressure_advance` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1303`: `pressure_advance` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2252-2255`: `enable_pressure_advance` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2257-2262`: `pressure_advance` option definition.

Related upstream behavior explicitly deferred:

- Runtime pressure advance and firmware-specific Linear Advance / Klipper PA behavior.
- Typed accessors or behavior changes for the newly registered keys.
- UI label/tooltip/max/mode metadata beyond the current registry boundary.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2264+`: adaptive pressure advance options and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/early.rs`: add sorted definition for `enable_pressure_advance`.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: add sorted definition for `pressure_advance`.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/pressure.rs`: new metadata assertions for pressure advance options.
- `crates/ares-core/src/options/registry/tests/metadata.rs`: include the new metadata module.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_pressure.rs`: new public lookup coverage file.
- `crates/ares-core/src/options/tests.rs`: include the new lookup module.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `enable_pressure_advance` (`coBools`, default `false`, field at `PrintConfig.hpp:1302`, definition lines 2252-2255)
- `pressure_advance` (`coFloats`, default `0.02`, field at `PrintConfig.hpp:1303`, definition lines 2257-2262)

## Functional requirements

1. Add the included missing options to sorted definition shards using `OptionValueKind::Bools` and `OptionValueKind::Floats`.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve sorted/no-duplicate test coverage across the merged table.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, runtime pressure advance behavior, firmware-specific behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter adaptive pressure advance options or following options outside the included slice.
8. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Update roadmap and milestone docs so E2E parity moves to M57.
10. Keep modified Rust files under 400 LOC.

## Deferred behavior

- Upstream UI and validation metadata from `PrintConfig.cpp:2252-2262` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Runtime pressure advance, adaptive pressure advance, firmware-specific behavior, slicing behavior, extrusion behavior, and G-code behavior are deferred to later source-cited milestones.
- Adaptive pressure advance and following options from `PrintConfig.cpp:2264+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove both new keys have expected kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The merged definition stream remains sorted and binary-search compatible.
- Public lookup coverage exists for both new keys.
- Plan/spec explicitly account for deferred UI metadata, runtime pressure advance behavior, slicing/extrusion/G-code behavior, and following adaptive pressure advance scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
