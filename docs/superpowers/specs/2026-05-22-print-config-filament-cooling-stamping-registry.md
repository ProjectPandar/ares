# M65 Spec: PrintConfig filament cooling and stamping option registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` filament toolchange delay, cooling moves, stamping, and initial cooling speed option-definition slice into `ares-core` option registry metadata by adding registry coverage for `filament_toolchange_delay`, `filament_cooling_moves`, `filament_stamping_loading_speed`, `filament_stamping_distance`, and `filament_cooling_initial_speed`.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1440`: `filament_toolchange_delay` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2636-2644`: `filament_toolchange_delay` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1441`: `filament_cooling_moves` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2646-2653`: `filament_cooling_moves` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1455`: `filament_stamping_loading_speed` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2655-2660`: `filament_stamping_loading_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1456`: `filament_stamping_distance` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2662-2668`: `filament_stamping_distance` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1442`: `filament_cooling_initial_speed` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2670-2676`: `filament_cooling_initial_speed` option definition.

Related upstream behavior explicitly deferred:

- UI label/tooltip/sidetext/min/max/mode metadata beyond the current registry boundary.
- Wipe-tower cooling/stamping behavior.
- Ramming, toolchange sequencing, cooling move runtime, and timing behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2678+`: `filament_minimal_purge_on_wipe_tower`, `filament_cooling_before_tower`, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle.rs`: add sorted filament definitions if file size remains below 400 LOC; otherwise split before implementing.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/filament.rs`: extend metadata assertions for the five options.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_filament.rs`: extend public lookup coverage for the five options.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `filament_toolchange_delay` (`coFloats`, default `0`, field at `PrintConfig.hpp:1440`, definition lines 2636-2644, Ares kind `Floats`)
- `filament_cooling_moves` (`coInts`, default `4`, field at `PrintConfig.hpp:1441`, definition lines 2646-2653, Ares kind `Ints`)
- `filament_stamping_loading_speed` (`coFloats`, default `0`, field at `PrintConfig.hpp:1455`, definition lines 2655-2660, Ares kind `Floats`)
- `filament_stamping_distance` (`coFloats`, default `0`, field at `PrintConfig.hpp:1456`, definition lines 2662-2668, Ares kind `Floats`)
- `filament_cooling_initial_speed` (`coFloats`, default `2.2`, field at `PrintConfig.hpp:1442`, definition lines 2670-2676, Ares kind `Floats`)

## Functional requirements

1. Add the included missing options to sorted definition shards using `Floats` and `Ints`.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve sorted/no-duplicate test coverage across the merged table.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, wipe-tower cooling/stamping behavior, ramming behavior, toolchange runtime behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter `filament_minimal_purge_on_wipe_tower`, `filament_cooling_before_tower`, or following options outside the included slice.
8. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Update roadmap and milestone docs so E2E parity moves to M66.
10. Keep modified Rust files under 400 LOC.

## Deferred behavior

- Upstream UI and validation metadata from `PrintConfig.cpp:2636-2676` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Wipe-tower cooling/stamping, ramming, toolchange sequencing, cooling move runtime, slicing, extrusion, and G-code behavior are deferred to later source-cited milestones.
- `filament_minimal_purge_on_wipe_tower`, `filament_cooling_before_tower`, and following options from `PrintConfig.cpp:2678+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all five new keys have expected kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The merged definition stream remains sorted and binary-search compatible.
- Public lookup coverage exists for all five new keys.
- Plan/spec explicitly account for deferred UI metadata, wipe-tower cooling/stamping behavior, ramming/toolchange behavior, slicing/extrusion/G-code behavior, and following `filament_minimal_purge_on_wipe_tower` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
