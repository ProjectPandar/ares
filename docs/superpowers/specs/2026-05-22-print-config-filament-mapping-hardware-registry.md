# M60 Spec: PrintConfig filament mapping and hardware flag option registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` filament multi-color, filament color type, nozzle hardness, filament-to-extruder mapping, dynamic map, and filament-switcher option-definition slice into `ares-core` option registry metadata by adding registry coverage for `filament_multi_colour`, `filament_colour_type`, `required_nozzle_HRC`, `filament_map`, `physical_extruder_map`, `filament_map_mode`, `enable_filament_dynamic_map`, and `has_filament_switcher`.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2385-2386`: `filament_multi_colour` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2389-2390`: `filament_colour_type` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1334`: `required_nozzle_HRC` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2393-2399`: `required_nozzle_HRC` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1336`: `filament_map` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2401-2405`: `filament_map` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1341`: `physical_extruder_map` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2407-2412`: `physical_extruder_map` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1335`: `filament_map_mode` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2414-2428`: `filament_map_mode` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2430-2434`: `enable_filament_dynamic_map` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2436-2440`: `has_filament_switcher` option definition.

Related upstream behavior explicitly deferred:

- Filament mapping runtime behavior and logical-to-physical extruder selection.
- Dynamic filament map behavior and filament switcher hardware behavior.
- Nozzle-HRC validation or warning behavior.
- Typed accessors or behavior changes for the newly registered keys.
- UI label/tooltip/min/max/mode/enum-label metadata beyond the current registry boundary.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2442+`: `filament_flush_temp`, `filament_flush_volumetric_speed`, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle.rs`: add sorted `enable_filament_dynamic_map` and `filament_*` definitions.
- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: add sorted `has_filament_switcher` definition.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: add sorted `physical_extruder_map` definition.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add sorted `required_nozzle_HRC` definition.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/filament.rs`: extend metadata assertions for filament mapping/hardware options.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_filament.rs`: extend public lookup coverage.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `filament_multi_colour` (`coStrings`, default empty string, definition lines 2385-2386)
- `filament_colour_type` (`coStrings`, default `1`, definition lines 2389-2390)
- `required_nozzle_HRC` (`coInts`, default `0`, field at `PrintConfig.hpp:1334`, definition lines 2393-2399)
- `filament_map` (`coInts`, default `1`, field at `PrintConfig.hpp:1336`, definition lines 2401-2405)
- `physical_extruder_map` (`coInts`, default `0`, field at `PrintConfig.hpp:1341`, definition lines 2407-2412)
- `filament_map_mode` (`coEnum`, default `Auto For Flush`, field at `PrintConfig.hpp:1335`, definition lines 2414-2428)
- `enable_filament_dynamic_map` (`coBool`, default `false`, definition lines 2430-2434)
- `has_filament_switcher` (`coBool`, default `false`, definition lines 2436-2440)

## Functional requirements

1. Add the included missing options to sorted definition shards using `OptionValueKind::Strings`, `OptionValueKind::Ints`, `OptionValueKind::Enum`, and `OptionValueKind::Bool`.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve sorted/no-duplicate test coverage across the merged table.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, filament mapping runtime behavior, dynamic map behavior, filament switcher behavior, nozzle-HRC validation, UI behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter `filament_flush_temp`, `filament_flush_volumetric_speed`, or following options outside the included slice.
8. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Update roadmap and milestone docs so E2E parity moves to M61.
10. Keep modified Rust files under 400 LOC.

## Deferred behavior

- Upstream UI and validation metadata from `PrintConfig.cpp:2385-2440` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Filament mapping runtime behavior, dynamic map behavior, filament switcher behavior, nozzle-HRC validation, slicing behavior, extrusion behavior, and G-code behavior are deferred to later source-cited milestones.
- `filament_flush_temp`, `filament_flush_volumetric_speed`, and following options from `PrintConfig.cpp:2442+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all eight new keys have expected kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The merged definition stream remains sorted and binary-search compatible.
- Public lookup coverage exists for all eight new keys.
- Plan/spec explicitly account for deferred UI metadata, filament mapping/runtime behavior, nozzle-HRC validation, slicing/extrusion/G-code behavior, and following `filament_flush_*` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
