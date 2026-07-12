# M61 Spec: PrintConfig filament flush and toolchange timing option registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` filament flush temperature, flush volumetric speed, max volumetric speed, and filament/tool-change timing option-definition slice into `ares-core` option registry metadata by adding registry coverage for `filament_flush_temp`, `filament_flush_volumetric_speed`, `filament_max_volumetric_speed`, `machine_load_filament_time`, `machine_unload_filament_time`, and `machine_tool_change_time`.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1344`: `filament_flush_temp` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2442-2450`: `filament_flush_temp` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1343`: `filament_flush_volumetric_speed` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2452-2460`: `filament_flush_volumetric_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1333`: `filament_max_volumetric_speed` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2462-2470`: `filament_max_volumetric_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1433`: `machine_load_filament_time` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2472-2479`: `machine_load_filament_time` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1435`: `machine_unload_filament_time` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2481-2488`: `machine_unload_filament_time` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1434`: `machine_tool_change_time` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2490-2497`: `machine_tool_change_time` option definition.

Related upstream behavior explicitly deferred:

- Flushing runtime behavior, temperature selection, and purge/flush calculations.
- Volumetric speed limiting and speed planner behavior.
- Tool-change timing use in statistics, estimates, scheduling, or G-code.
- Typed accessors or behavior changes for the newly registered keys.
- UI label/tooltip/sidetext/min/max/mode metadata beyond the current registry boundary.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2500+`: `support_object_skip_flush`, `bed_temperature_formula`, `nozzle_flush_dataset`, `filament_diameter` source-refresh, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry.rs`: add `OptionValueKind::IntsNullable` for source-cited nullable integer vector metadata.
- `crates/ares-core/src/options/registry/definitions/table/pre_middle.rs`: add sorted `filament_flush_*` and `filament_max_volumetric_speed` definitions.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: add sorted `machine_load_filament_time`, `machine_tool_change_time`, and `machine_unload_filament_time` definitions.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/filament.rs`: extend metadata assertions for filament flush and volumetric-speed options.
- `crates/ares-core/src/options/registry/tests/metadata/hardware.rs`: extend metadata assertions for machine filament/tool-change timing options.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_filament.rs`: extend public lookup coverage.
- `crates/ares-core/src/options/tests/registry_lookup_hardware.rs`: extend public lookup coverage for machine timing options.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `filament_flush_temp` (`coInts`, `nullable = true`, default `0`, field at `PrintConfig.hpp:1344`, definition lines 2442-2450, Ares kind `IntsNullable`)
- `filament_flush_volumetric_speed` (`coFloats`, `nullable = true`, default `0`, field at `PrintConfig.hpp:1343`, definition lines 2452-2460, Ares kind `FloatsNullable`)
- `filament_max_volumetric_speed` (`coFloats`, default `2`, field at `PrintConfig.hpp:1333`, definition lines 2462-2470, Ares kind `Floats`)
- `machine_load_filament_time` (`coFloat`, default `0`, field at `PrintConfig.hpp:1433`, definition lines 2472-2479, Ares kind `Float`)
- `machine_unload_filament_time` (`coFloat`, default `0`, field at `PrintConfig.hpp:1435`, definition lines 2481-2488, Ares kind `Float`)
- `machine_tool_change_time` (`coFloat`, default `0`, field at `PrintConfig.hpp:1434`, definition lines 2490-2497, Ares kind `Float`)

## Functional requirements

1. Add `OptionValueKind::IntsNullable` as registry metadata for Orca `ConfigOptionIntsNullable`; do not add parsing/runtime behavior for it in this milestone.
2. Add the included missing options to sorted definition shards using `IntsNullable`, `FloatsNullable`, `Floats`, and `Float`.
3. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
4. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
5. Preserve sorted/no-duplicate test coverage across the merged table.
6. Preserve `SliceOptions` unknown-value storage and current public slicing API.
7. Do not add typed parsing/accessors, flushing runtime behavior, volumetric speed limiting, tool-change timing behavior, UI behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
8. Do not add or alter `support_object_skip_flush`, `bed_temperature_formula`, `nozzle_flush_dataset`, `filament_diameter` source-refresh, or following options outside the included slice.
9. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
10. Update roadmap and milestone docs so E2E parity moves to M62.
11. Keep modified Rust files under 400 LOC.

## Deferred behavior

- Upstream UI and validation metadata from `PrintConfig.cpp:2442-2497` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation; nullable identity is preserved only through `IntsNullable`/`FloatsNullable` kinds.
- Flushing runtime behavior, volumetric speed limiting, tool-change timing behavior, slicing behavior, extrusion behavior, and G-code behavior are deferred to later source-cited milestones.
- `support_object_skip_flush`, `bed_temperature_formula`, `nozzle_flush_dataset`, `filament_diameter` source-refresh, and following options from `PrintConfig.cpp:2500+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all six new keys have expected kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The merged definition stream remains sorted and binary-search compatible.
- Public lookup coverage exists for all six new keys.
- Plan/spec explicitly account for deferred UI metadata, flushing runtime behavior, volumetric speed limiting, tool-change timing behavior, slicing/extrusion/G-code behavior, and following `support_object_skip_flush` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
