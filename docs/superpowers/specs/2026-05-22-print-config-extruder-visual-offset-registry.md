# M54 Spec: PrintConfig extruder visual and offset option registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` extruder visual and offset option-definition slice into `ares-core` option registry metadata by adding registry coverage for `grab_length`, `extruder_colour`, and `extruder_offset`.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1517`: `extruder_colour` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1518`: `extruder_offset` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1625`: `grab_length` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2202-2207`: `grab_length` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2209-2215`: `extruder_colour` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2217-2225`: `extruder_offset` option definition.

Related upstream behavior explicitly deferred:

- UI color swatch behavior and color validation.
- Firmware/tool offset application in G-code or coordinates.
- Typed accessors or behavior changes for the newly registered keys.
- UI label/tooltip/gui_type/sidetext/min/mode metadata.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2227+`: `filament_flow_ratio`, `print_flow_ratio`, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/early.rs`: add sorted definitions for `extruder_colour` and `extruder_offset`.
- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: add sorted definition for `grab_length`, which sorts between existing `gap_fill_target` and `hot_plate_temp`.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/hardware.rs`: metadata assertions for extruder visual/offset options.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_hardware.rs`: public lookup coverage.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `grab_length` (`coFloats`, default `0`, field at `PrintConfig.hpp:1625`, definition lines 2202-2207)
- `extruder_colour` (`coStrings`, default empty string, field at `PrintConfig.hpp:1517`, definition lines 2209-2215)
- `extruder_offset` (`coPoints`, default `0x0`, field at `PrintConfig.hpp:1518`, definition lines 2217-2225)

## Functional requirements

1. Add the included missing options to sorted definition shards using existing `OptionValueKind::Floats`, `Strings`, and `Points`.
2. Preserve public API: `OptionDefinition`, `option_definitions()`, and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve sorted/no-duplicate test coverage across the merged table.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, UI color behavior, extruder offset behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter `filament_flow_ratio`, `print_flow_ratio`, or following options outside the included slice.
8. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Update roadmap and milestone docs so E2E parity moves to M55.
10. Keep modified Rust files under 400 LOC.

## Deferred behavior

- Upstream UI and validation metadata from `PrintConfig.cpp:2202-2225` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- UI color behavior, extruder offset behavior, slicing behavior, extrusion behavior, and G-code behavior are deferred to later source-cited milestones.
- `filament_flow_ratio`, `print_flow_ratio`, and following options from `PrintConfig.cpp:2227+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all three new keys have expected kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The merged definition stream remains sorted and binary-search compatible.
- Public lookup coverage exists for all three new keys.
- Plan/spec explicitly account for deferred upstream UI metadata, UI color behavior, extruder offset behavior, slicing/extrusion/G-code behavior, and following flow-ratio scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
