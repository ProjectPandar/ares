# M79 Spec: PrintConfig first-layer temperature and fan-speed registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` first-layer nozzle temperature and fan-speed option-definition slice into `ares-core` option registry metadata by adding registry coverage for `nozzle_temperature_initial_layer`, `full_fan_speed_layer`, `support_material_interface_fan_speed`, `internal_bridge_fan_speed`, and `ironing_fan_speed`.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1533`: `nozzle_temperature_initial_layer` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3316-3323`: `nozzle_temperature_initial_layer` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1534`: `full_fan_speed_layer` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3325-3335`: `full_fan_speed_layer` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1628`: `support_material_interface_fan_speed` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3337-3347`: `support_material_interface_fan_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1629`: `internal_bridge_fan_speed` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3350-3359`: `internal_bridge_fan_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1630`: `ironing_fan_speed` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3361-3370`: `ironing_fan_speed` option definition.

Related upstream behavior explicitly deferred:

- UI label/full_label/category/tooltip/sidetext/min/max/mode metadata beyond the current registry boundary.
- Nozzle-temperature behavior, fan-speed behavior, disable semantics for `-1`, and override behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3372+`: `filament_ironing_flow`, `filament_ironing_spacing`, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: add sorted definitions for `full_fan_speed_layer`, `internal_bridge_fan_speed`, and `ironing_fan_speed`.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: add sorted definition for `nozzle_temperature_initial_layer`.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add sorted definition for `support_material_interface_fan_speed`.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/cooling.rs`: extend source metadata assertions for the five options.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_cooling.rs`: extend public lookup coverage for the five options.
- `docs/roadmap.md` and `docs/milestones/m79-print-config-first-layer-fan-temperature-registry.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `nozzle_temperature_initial_layer` (`coInts`, default `200`, field at `PrintConfig.hpp:1533`, definition lines 3316-3323, Ares kind `Ints`)
- `full_fan_speed_layer` (`coInts`, default `0`, field at `PrintConfig.hpp:1534`, definition lines 3325-3335, Ares kind `Ints`)
- `support_material_interface_fan_speed` (`coInts`, default `-1`, field at `PrintConfig.hpp:1628`, definition lines 3337-3347, Ares kind `Ints`)
- `internal_bridge_fan_speed` (`coInts`, default `-1`, field at `PrintConfig.hpp:1629`, definition lines 3350-3359, Ares kind `Ints`)
- `ironing_fan_speed` (`coInts`, default `-1`, field at `PrintConfig.hpp:1630`, definition lines 3361-3370, Ares kind `Ints`)

## Functional requirements

1. Add the missing options to existing sorted definition shards using `Ints`.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, nozzle-temperature behavior, fan-speed behavior, disable semantics, override behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add or alter `filament_ironing_flow`, `filament_ironing_spacing`, or following options outside the included slice.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Keep modified Rust files under 400 LOC.

## Deferred behavior

- Upstream UI, validation, mode, and GUI metadata from `PrintConfig.cpp:3316-3370` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Nozzle-temperature behavior, fan-speed behavior, disable semantics, override behavior, slicing, extrusion, and G-code behavior are deferred to later source-cited milestones.
- `filament_ironing_flow`, `filament_ironing_spacing`, and following options from `PrintConfig.cpp:3372+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all five new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all five new keys.
- Plan/spec explicitly account for deferred UI/bounds metadata, nozzle/fan behavior, slicing/extrusion/G-code behavior, and following filament-ironing scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
