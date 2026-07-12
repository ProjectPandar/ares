# M63 Spec: PrintConfig pellet flow and shrinkage option registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` pellet flow coefficient, adaptive volumetric speed metadata, volumetric speed coefficients, and filament shrinkage option-definition slice into `ares-core` option registry metadata by adding registry coverage for `pellet_flow_coefficient`, `filament_adaptive_volumetric_speed`, `volumetric_speed_coefficients`, `filament_shrink`, and `filament_shrinkage_compensation_z`.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2551-2555`: `pellet_flow_coefficient` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1318`: `filament_adaptive_volumetric_speed` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2557-2565`: `filament_adaptive_volumetric_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1319`: `volumetric_speed_coefficients` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2567-2569`: `volumetric_speed_coefficients` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1621`: `filament_shrink` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2571-2582`: `filament_shrink` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1622`: `filament_shrinkage_compensation_z` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2584-2594`: `filament_shrinkage_compensation_z` option definition.

Related upstream behavior explicitly deferred:

- Pellet flow coefficient conversion into filament diameter or volume calculations.
- Adaptive volumetric speed fitting, limiting, and planner behavior.
- Volumetric speed coefficient parsing/evaluation.
- XY/Z shrinkage scaling behavior.
- Typed accessors or behavior changes for the newly registered keys.
- UI label/tooltip/sidetext/min/max/mode/ratio/nullable metadata beyond the current registry boundary.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2596+`: `filament_adhesiveness_category`, `filament_loading_speed`, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry.rs`: add metadata-only `OptionValueKind::BoolsNullable` and `OptionValueKind::Percents`.
- `crates/ares-core/src/options/registry/definitions/table/pre_middle.rs`: add sorted `filament_adaptive_volumetric_speed`, `filament_shrink`, and `filament_shrinkage_compensation_z` definitions.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: add sorted `pellet_flow_coefficient` definition.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add sorted `volumetric_speed_coefficients` definition.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/filament.rs`: extend metadata assertions for filament adaptive volumetric speed and shrinkage options.
- `crates/ares-core/src/options/registry/tests/metadata/speed.rs`: extend metadata assertions for `pellet_flow_coefficient` and `volumetric_speed_coefficients`.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_filament.rs`: extend public lookup coverage for filament adaptive/shrinkage options.
- `crates/ares-core/src/options/tests/registry_lookup_speed.rs`: create public lookup coverage for `pellet_flow_coefficient` and `volumetric_speed_coefficients`.
- `crates/ares-core/src/options/tests.rs`: include the new speed lookup module.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `pellet_flow_coefficient` (`coFloats`, default `0.4157`, definition lines 2551-2555, Ares kind `Floats`)
- `filament_adaptive_volumetric_speed` (`coBools`, `nullable = true`, default `false`, field at `PrintConfig.hpp:1318`, definition lines 2557-2565, Ares kind `BoolsNullable`)
- `volumetric_speed_coefficients` (`coStrings`, default empty string, field at `PrintConfig.hpp:1319`, definition lines 2567-2569, Ares kind `Strings`)
- `filament_shrink` (`coPercents`, default `100`, field at `PrintConfig.hpp:1621`, definition lines 2571-2582, Ares kind `Percents`)
- `filament_shrinkage_compensation_z` (`coPercents`, default `100`, field at `PrintConfig.hpp:1622`, definition lines 2584-2594, Ares kind `Percents`)

## Functional requirements

1. Add metadata-only `OptionValueKind::BoolsNullable` and `OptionValueKind::Percents`; do not add typed parsing/runtime behavior for them in this milestone.
2. Add the included missing options to sorted definition shards using `Floats`, `BoolsNullable`, `Strings`, and `Percents`.
3. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
4. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
5. Preserve sorted/no-duplicate test coverage across the merged table.
6. Preserve `SliceOptions` unknown-value storage and current public slicing API.
7. Do not add typed parsing/accessors, pellet-to-diameter conversion, adaptive volumetric speed limiting, volumetric coefficient evaluation, shrinkage scaling, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
8. Do not add or alter `filament_adhesiveness_category`, `filament_loading_speed`, or following options outside the included slice.
9. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
10. Update roadmap and milestone docs so E2E parity moves to M64.
11. Keep modified Rust files under 400 LOC.

## Deferred behavior

- Upstream UI and validation metadata from `PrintConfig.cpp:2551-2594` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation; nullable and percent-vector identity is preserved only through `BoolsNullable` and `Percents`.
- Pellet-to-diameter conversion, adaptive volumetric speed limiting, volumetric coefficient evaluation, shrinkage scaling, slicing behavior, extrusion behavior, and G-code behavior are deferred to later source-cited milestones.
- `filament_adhesiveness_category`, `filament_loading_speed`, and following options from `PrintConfig.cpp:2596+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all five new keys have expected kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The merged definition stream remains sorted and binary-search compatible.
- Public lookup coverage exists for all five new keys.
- Plan/spec explicitly account for deferred UI metadata, pellet conversion, adaptive volumetric behavior, shrinkage scaling, slicing/extrusion/G-code behavior, and following `filament_loading_speed` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
