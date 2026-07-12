# M59 Spec: PrintConfig fan cooling and filament color note option registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` fan cooling layer time, default filament color, filament color, and filament notes option-definition slice into `ares-core` option registry metadata by adding registry coverage for `fan_cooling_layer_time`, `default_filament_colour`, `filament_colour`, and `filament_notes`.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1521`: `fan_cooling_layer_time` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1331`: `default_filament_colour` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1325`: `filament_colour` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1632`: `filament_notes` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2349-2357`: `fan_cooling_layer_time` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2359-2365`: `default_filament_colour` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2367-2372`: `filament_colour` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2375-2382`: `filament_notes` option definition.

Related upstream behavior explicitly deferred:

- Fan runtime behavior and layer-time cooling behavior.
- Color UI behavior and note UI behavior.
- Typed accessors or behavior changes for the newly registered keys.
- UI label/tooltip/sidetext/min/max/mode/gui/multiline/full-width/height metadata beyond the current registry boundary.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2385+`: `filament_multi_colour`, `filament_colour_type`, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle.rs`: add sorted definitions for all four options.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/filament.rs`: new metadata assertions for fan cooling and filament color/note options.
- `crates/ares-core/src/options/registry/tests/metadata.rs`: include the new metadata module.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_filament.rs`: public lookup coverage.
- `crates/ares-core/src/options/tests.rs`: include the new lookup module.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `fan_cooling_layer_time` (`coFloats`, default `60`, field at `PrintConfig.hpp:1521`, definition lines 2349-2357)
- `default_filament_colour` (`coStrings`, default empty string, field at `PrintConfig.hpp:1331`, definition lines 2359-2365)
- `filament_colour` (`coStrings`, default `#F2754E`, field at `PrintConfig.hpp:1325`, definition lines 2367-2372)
- `filament_notes` (`coStrings`, default empty string, field at `PrintConfig.hpp:1632`, definition lines 2375-2382)

## Functional requirements

1. Add the included missing options to sorted definition shards using `OptionValueKind::Floats` and `OptionValueKind::Strings`.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve sorted/no-duplicate test coverage across the merged table.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, fan runtime behavior, color UI behavior, note UI behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter `filament_multi_colour`, `filament_colour_type`, or following options outside the included slice.
8. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Update roadmap and milestone docs so E2E parity moves to M60.
10. Keep modified Rust files under 400 LOC.

## Deferred behavior

- Upstream UI and validation metadata from `PrintConfig.cpp:2349-2382` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Fan runtime behavior, color UI behavior, note UI behavior, slicing behavior, extrusion behavior, and G-code behavior are deferred to later source-cited milestones.
- `filament_multi_colour`, `filament_colour_type`, and following options from `PrintConfig.cpp:2385+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all four new keys have expected kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The merged definition stream remains sorted and binary-search compatible.
- Public lookup coverage exists for all four new keys.
- Plan/spec explicitly account for deferred UI metadata, fan/color/note runtime behavior, slicing/extrusion/G-code behavior, and following `filament_multi_colour` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
