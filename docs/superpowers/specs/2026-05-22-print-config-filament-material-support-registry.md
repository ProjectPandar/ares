# M70 Spec: PrintConfig filament material and support option registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` filament material/statistics/support option-definition slice into `ares-core` option registry metadata by adding registry coverage for `filament_density`, `filament_type`, `filament_soluble`, `filament_change_length`, `filament_is_support`, and `filament_printable`.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1321`: `filament_density` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2776-2782`: `filament_density` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1322`: `filament_type` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2784-2796`: `filament_type` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1323`: `filament_soluble` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2798-2802`: `filament_soluble` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1329`: `filament_change_length` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2804-2810`: `filament_change_length` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1327`: `filament_is_support` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2812-2816`: `filament_is_support` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1328`: `filament_printable` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2818-2826`: `filament_printable` option definition.

Related upstream behavior explicitly deferred:

- UI label/tooltip/sidetext/min/mode/gui-type/gui-flags/enum-values metadata beyond the current registry boundary.
- `MaterialType::all()` enum population and material database behavior for `filament_type`.
- Filament density statistics behavior, soluble/support material behavior, filament change-length runtime behavior, and extruder printability behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2828+`: `temperature_vitrification`, `filament_cost`, `filament_settings_id`, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle.rs` and/or `pre_middle_tail.rs`: add sorted option definitions while keeping modified Rust files under 400 LOC.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/filament.rs`: extend metadata assertions for the six options.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_filament.rs`: extend public lookup coverage for the six options.
- `docs/roadmap.md` and `docs/milestones/m70-print-config-filament-material-support-registry.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `filament_density` (`coFloats`, default `0`, field at `PrintConfig.hpp:1321`, definition lines 2776-2782, Ares kind `Floats`)
- `filament_type` (`coStrings`, default `PLA`, field at `PrintConfig.hpp:1322`, definition lines 2784-2796, Ares kind `Strings`)
- `filament_soluble` (`coBools`, default `false`, field at `PrintConfig.hpp:1323`, definition lines 2798-2802, Ares kind `Bools`)
- `filament_change_length` (`coFloats`, default `10`, field at `PrintConfig.hpp:1329`, definition lines 2804-2810, Ares kind `Floats`)
- `filament_is_support` (`coBools`, default `false`, field at `PrintConfig.hpp:1327`, definition lines 2812-2816, Ares kind `Bools`)
- `filament_printable` (`coInts`, default `3`, field at `PrintConfig.hpp:1328`, definition lines 2818-2826, Ares kind `Ints`)

## Functional requirements

1. Add the included missing options to existing sorted definition shards using `Floats`, `Strings`, `Bools`, and `Ints`.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve sorted/no-duplicate test coverage across the merged table.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, material database behavior, soluble/support runtime behavior, printability behavior, statistics behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter `temperature_vitrification`, `filament_cost`, `filament_settings_id`, or following options outside the included slice.
8. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
10. Keep modified Rust files under 400 LOC.

## Deferred behavior

- Upstream UI, validation, mode, and enum metadata from `PrintConfig.cpp:2776-2826` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- `MaterialType::all()` enum population, filament statistics, soluble/support behavior, filament change-length behavior, extruder printability, slicing, extrusion, and G-code behavior are deferred to later source-cited milestones.
- `temperature_vitrification`, `filament_cost`, `filament_settings_id`, and following options from `PrintConfig.cpp:2828+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all six new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all six new keys.
- Plan/spec explicitly account for deferred UI/enum metadata, material database behavior, soluble/support/printability/statistics behavior, slicing/extrusion/G-code behavior, and following `temperature_vitrification` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
