# M71 Spec: PrintConfig filament identity and statistics option registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` filament softening-temperature, price/statistics, identity, and vendor option-definition slice into `ares-core` option registry metadata by adding registry coverage for `temperature_vitrification`, `filament_cost`, `filament_settings_id`, `filament_ids`, and `filament_vendor`.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1332`: `temperature_vitrification` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2828-2835`: `temperature_vitrification` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1330`: `filament_cost` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2837-2843`: `filament_cost` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2845-2849`: `filament_settings_id` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1324`: `filament_ids` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2850-2852`: `filament_ids` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1326`: `filament_vendor` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2854-2859`: `filament_vendor` option definition.

Related upstream behavior explicitly deferred:

- UI label/tooltip/sidetext/min/mode and CLI/no-CLI metadata beyond the current registry boundary.
- Softening-temperature behavior, filament price/statistics behavior, preset settings-id/ids identity behavior, and vendor resolution/display behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2861+`: `infill_direction`, `solid_infill_direction`, `sparse_infill_density`, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle.rs`: add sorted `filament_cost` and `filament_ids`; move existing `filament_ramming_parameters`, `filament_shrink`, and `filament_shrinkage_compensation_z` to `pre_middle_tail.rs` so `pre_middle.rs` remains below 400 LOC.
- `crates/ares-core/src/options/registry/definitions/table/pre_middle_tail.rs`: add sorted `filament_ramming_parameters`, `filament_settings_id`, `filament_shrink`, `filament_shrinkage_compensation_z`, and `filament_vendor` while preserving the existing moved definitions exactly.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add sorted `temperature_vitrification`.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/filament.rs`: extend metadata assertions for the five options.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_filament.rs`: extend public lookup coverage for the five options.
- `docs/roadmap.md` and `docs/milestones/m71-print-config-filament-identity-statistics-registry.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `temperature_vitrification` (`coInts`, default `100`, field at `PrintConfig.hpp:1332`, definition lines 2828-2835, Ares kind `Ints`)
- `filament_cost` (`coFloats`, default `0`, field at `PrintConfig.hpp:1330`, definition lines 2837-2843, Ares kind `Floats`)
- `filament_settings_id` (`coStrings`, default empty string, no generated config field in `PrintConfig.hpp`, definition lines 2845-2849, Ares kind `Strings`)
- `filament_ids` (`coStrings`, default empty string, field at `PrintConfig.hpp:1324`, definition lines 2850-2852, Ares kind `Strings`)
- `filament_vendor` (`coStrings`, default `(Undefined)`, field at `PrintConfig.hpp:1326`, definition lines 2854-2859, Ares kind `Strings`)

## Functional requirements

1. Add the included missing options to existing sorted definition shards using `Ints`, `Floats`, and `Strings`.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve sorted/no-duplicate test coverage across the merged table.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, softening-temperature behavior, statistics behavior, settings identity behavior, vendor behavior, CLI/no-CLI behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter `infill_direction`, `solid_infill_direction`, `sparse_infill_density`, or following options outside the included slice.
8. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
10. Keep modified Rust files under 400 LOC.

## Deferred behavior

- Upstream UI, validation, mode, and CLI metadata from `PrintConfig.cpp:2828-2859` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Softening-temperature behavior, filament price/statistics, settings-id/ids identity behavior, vendor display/resolution, slicing, extrusion, and G-code behavior are deferred to later source-cited milestones.
- `infill_direction`, `solid_infill_direction`, `sparse_infill_density`, and following options from `PrintConfig.cpp:2861+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all five new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all five new keys.
- Plan/spec explicitly account for deferred UI/CLI metadata, softening-temperature/statistics/identity/vendor behavior, slicing/extrusion/G-code behavior, and following `infill_direction` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
