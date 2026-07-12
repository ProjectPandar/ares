# M31 Spec: PrintConfig bed type and filament sequence option registry slice

## Goal
Port the next adjacent FFF-specific `libslic3r::PrintConfigDef::init_fff_params` option-definition slice into `ares-core` option registry metadata, covering current/default bed type and filament print sequence options without adding bed selection, print-order, or G-code behavior.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:314-323`: `BedType` enum values.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:333-335`: `LayerSeq` enum values.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1489`: typed `ConfigOptionEnum<BedType>` field for `curr_bed_type`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1507-1509`: typed print-sequence fields.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:467-483`: `BedType` and `LayerSeq` enum key maps.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1043-1108`: `PrintConfigDef::init_fff_params()` bed type and filament sequence option definitions.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table.rs`: sorted source-cited definition table.
- `crates/ares-core/src/options/registry/tests.rs`: registry table tests.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public lookup/count coverage.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `curr_bed_type` (`coEnum`, default `Cool Plate` from `btPC`, lines 1043-1061)
- `default_bed_type` (`coString`, default ``, lines 1065-1069)
- `first_layer_print_sequence` (`coInts`, default `0`, lines 1072-1076)
- `other_layers_print_sequence` (`coInts`, default `0`, lines 1078-1082)
- `other_layers_print_sequence_nums` (`coInt`, default `0`, lines 1084-1086)
- `first_layer_sequence_choice` (`coEnum`, default `Auto` from `flsAuto`, lines 1088-1097)
- `other_layers_sequence_choice` (`coEnum`, default `Auto` from `flsAuto`, lines 1099-1108)

## Functional requirements

1. Add the included options to the sorted definition table using existing `OptionValueKind` variants.
2. Preserve public API: `OptionDefinition`, `option_definitions()`, and `option_definition(key)` remain unchanged.
3. Preserve binary-search lookup and sorted/no-duplicate test coverage.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, enum value APIs, bed selection logic, print-order behavior, or G-code behavior for these options in this milestone.
6. Do not add a new pipeline stage, crate, dependency, filesystem behavior, network behavior, or UI behavior.
7. Update roadmap and milestone docs so E2E parity moves to M32.
8. Modified Rust files must remain under 400 LOC.

## Deferred behavior

- `before_layer_change_gcode` and following shell/cooling options from `PrintConfig.cpp:1110+` are deferred.
- Full enum value exposure for `BedType` and `LayerSeq` is deferred.
- Actual bed-type selection, filament sequencing, print ordering, and G-code behavior are deferred to later source-cited milestones.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all seven new keys, kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
