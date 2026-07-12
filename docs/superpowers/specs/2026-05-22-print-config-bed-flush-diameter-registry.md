# M62 Spec: PrintConfig bed temperature and flush dataset option registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` support skip-flush, bed temperature formula, nozzle flush dataset, and filament diameter source-refresh option-definition slice into `ares-core` option registry metadata by adding registry coverage for `support_object_skip_flush`, `bed_temperature_formula`, and `nozzle_flush_dataset`, while refreshing the existing `filament_diameter` source citation.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1339`: `support_object_skip_flush` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2500-2501`: `support_object_skip_flush` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1340`: `bed_temperature_formula` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2503-2512`: `bed_temperature_formula` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1342`: `nozzle_flush_dataset` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2514-2516`: `nozzle_flush_dataset` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1317`: `filament_diameter` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2518-2523`: `filament_diameter` option definition.

Related upstream behavior explicitly deferred:

- Bed-temperature selection behavior and formula application during slicing.
- Nozzle flush dataset semantics and flushing calculations.
- Support-object skip-flush behavior.
- Filament diameter typed/runtime behavior changes; the existing Ares typed handling remains unchanged.
- UI label/tooltip/sidetext/min/mode/enum-label metadata beyond the current registry boundary.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2551+`: `pellet_flow_coefficient`, `filament_adaptive_volumetric_speed`, `volumetric_speed_coefficients`, `filament_shrink`, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/early.rs`: add sorted `bed_temperature_formula` definition.
- `crates/ares-core/src/options/registry/definitions/table/pre_middle.rs`: refresh existing `filament_diameter` source citation only.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: add sorted `nozzle_flush_dataset` definition.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add sorted `support_object_skip_flush` definition.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/filament.rs`: refresh `filament_diameter` metadata coverage.
- `crates/ares-core/src/options/registry/tests/metadata/hardware.rs`: extend metadata assertions for bed temperature formula and nozzle flush dataset.
- `crates/ares-core/src/options/registry/tests/metadata/support.rs`: add metadata assertion for support-object skip-flush.
- `crates/ares-core/src/options/registry/tests/metadata.rs`: include the new support metadata module.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_filament.rs`: add public `filament_diameter` source lookup coverage.
- `crates/ares-core/src/options/tests/registry_lookup_hardware.rs`: extend public lookup coverage for bed temperature formula and nozzle flush dataset.
- `crates/ares-core/src/options/tests/registry_lookup_support.rs`: add public lookup coverage for support-object skip-flush.
- `crates/ares-core/src/options/tests.rs`: include the new support lookup module.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add or refresh registry metadata for these exact upstream options and default values:

- `support_object_skip_flush` (`coBool`, default `false`, field at `PrintConfig.hpp:1339`, definition lines 2500-2501, Ares kind `Bool`)
- `bed_temperature_formula` (`coEnum`, default `by_highest_temp`, field at `PrintConfig.hpp:1340`, definition lines 2503-2512, Ares kind `Enum`)
- `nozzle_flush_dataset` (`coInts`, `nullable = true`, default `0`, field at `PrintConfig.hpp:1342`, definition lines 2514-2516, Ares kind `IntsNullable`)
- `filament_diameter` (`coFloats`, default `1.75`, field at `PrintConfig.hpp:1317`, definition lines 2518-2523, Ares kind `Floats`; this key already exists and is source-refreshed only)

## Functional requirements

1. Add the three missing options to sorted definition shards using `Bool`, `Enum`, and `IntsNullable`.
2. Refresh only the `filament_diameter` source citation to include `PrintConfig.hpp:1317; PrintConfig.cpp:2518-2523`; do not change its kind, default value, typed parser, or runtime behavior.
3. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
4. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
5. Preserve sorted/no-duplicate test coverage across the merged table.
6. Preserve `SliceOptions` unknown-value storage and current public slicing API.
7. Do not add typed parsing/accessors, bed-temperature selection behavior, nozzle flush dataset behavior, support-object skip-flush behavior, filament diameter runtime behavior changes, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
8. Do not add or alter `pellet_flow_coefficient`, `filament_adaptive_volumetric_speed`, `volumetric_speed_coefficients`, `filament_shrink`, or following options outside the included slice.
9. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
10. Update roadmap and milestone docs so E2E parity moves to M63.
11. Keep modified Rust files under 400 LOC.

## Deferred behavior

- Upstream UI and validation metadata from `PrintConfig.cpp:2500-2523` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation; nullable identity is preserved only through `IntsNullable`.
- Bed-temperature selection behavior, nozzle flush dataset behavior, support-object skip-flush behavior, filament diameter runtime changes, slicing behavior, extrusion behavior, and G-code behavior are deferred to later source-cited milestones.
- `pellet_flow_coefficient`, `filament_adaptive_volumetric_speed`, `volumetric_speed_coefficients`, `filament_shrink`, and following options from `PrintConfig.cpp:2551+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all three new keys and refreshed `filament_diameter` have expected kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The merged definition stream remains sorted and binary-search compatible.
- Public lookup coverage exists for all three new keys and refreshed `filament_diameter` source citation.
- Plan/spec explicitly account for deferred UI metadata, bed-temperature selection behavior, nozzle flush dataset behavior, support-object skip-flush behavior, filament diameter runtime changes, slicing/extrusion/G-code behavior, and following `pellet_flow_coefficient` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
