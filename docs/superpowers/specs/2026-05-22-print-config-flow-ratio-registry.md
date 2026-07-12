# M55 Spec: PrintConfig filament and print flow ratio option registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` filament and print flow ratio option-definition slice into `ares-core` option registry metadata by adding registry coverage for `filament_flow_ratio` and `print_flow_ratio`, including the missing `OptionValueKind::FloatsNullable` representation for the upstream nullable float-vector shape (`coFloats`, `nullable = true`, `ConfigOptionFloatsNullable` default).

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1301`: `filament_flow_ratio` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2227-2237`: `filament_flow_ratio` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2239-2250`: `print_flow_ratio` option definition.

Related upstream behavior explicitly deferred:

- Runtime flow scaling and object/material override behavior.
- Typed accessors or behavior changes for the newly registered keys.
- UI label/category/tooltip/min/max/mode metadata beyond the encoded nullable value-kind shape.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2252+`: `enable_pressure_advance`, `pressure_advance`, adaptive pressure advance options, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry.rs`: add `OptionValueKind::FloatsNullable` for upstream `coFloats` plus `nullable = true` metadata.
- `crates/ares-core/src/options/registry/definitions/table/early.rs`: add sorted definition for `filament_flow_ratio`.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: add sorted definition for `print_flow_ratio`.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/flow.rs`: metadata assertions for flow ratio options.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_flow.rs`: new public lookup coverage file so existing lookup files stay focused and under the LOC limit.
- `crates/ares-core/src/options/tests.rs`: include the new lookup module.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `filament_flow_ratio` (`coFloats` with `nullable = true`, `ConfigOptionFloatsNullable` default `1`, field at `PrintConfig.hpp:1301`, definition lines 2227-2237)
- `print_flow_ratio` (`coFloat`, default `1`, definition lines 2239-2250)

## Functional requirements

1. Add `OptionValueKind::FloatsNullable` for upstream `coFloats` plus `nullable = true` option metadata.
2. Add the included missing options to sorted definition shards using `OptionValueKind::FloatsNullable` and `Float`.
3. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
4. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
5. Preserve sorted/no-duplicate test coverage across the merged table.
6. Preserve `SliceOptions` unknown-value storage and current public slicing API.
7. Do not add typed parsing/accessors, runtime flow scaling, material override behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
8. Do not add or alter `enable_pressure_advance`, `pressure_advance`, adaptive pressure advance options, or following options outside the included slice.
9. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
10. Update roadmap and milestone docs so E2E parity moves to M56.
11. Keep modified Rust files under 400 LOC.

## Deferred behavior

- Upstream UI and validation metadata from `PrintConfig.cpp:2227-2250` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Runtime flow scaling, material/profile override behavior, slicing behavior, extrusion behavior, and G-code behavior are deferred to later source-cited milestones.
- Pressure advance and following options from `PrintConfig.cpp:2252+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove both new keys have expected kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The merged definition stream remains sorted and binary-search compatible.
- Public lookup coverage exists for both new keys.
- Plan/spec explicitly account for the new `FloatsNullable` kind, deferred upstream UI metadata, flow-scaling behavior, slicing/extrusion/G-code behavior, and following pressure-advance scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
