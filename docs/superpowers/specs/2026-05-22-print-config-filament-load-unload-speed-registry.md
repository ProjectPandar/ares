# M64 Spec: PrintConfig filament load/unload speed option registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` filament adhesiveness category and filament loading/unloading speed option-definition slice into `ares-core` option registry metadata by adding registry coverage for `filament_adhesiveness_category`, `filament_loading_speed`, `filament_loading_speed_start`, `filament_unloading_speed`, and `filament_unloading_speed_start`.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1320`: `filament_adhesiveness_category` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2596-2601`: `filament_adhesiveness_category` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1436`: `filament_loading_speed` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2603-2609`: `filament_loading_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1437`: `filament_loading_speed_start` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2611-2617`: `filament_loading_speed_start` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1438`: `filament_unloading_speed` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2619-2626`: `filament_unloading_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1439`: `filament_unloading_speed_start` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2628-2634`: `filament_unloading_speed_start` option definition.

Related upstream behavior explicitly deferred:

- UI label/tooltip/sidetext/min/max/mode metadata beyond the current registry boundary.
- Wipe-tower loading/unloading behavior.
- Ramming, toolchange sequencing, cooling/stamping moves, and runtime timing behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2636+`: `filament_toolchange_delay`, `filament_cooling_moves`, and following options.
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

- `filament_adhesiveness_category` (`coInts`, default `0`, field at `PrintConfig.hpp:1320`, definition lines 2596-2601, Ares kind `Ints`)
- `filament_loading_speed` (`coFloats`, default `28`, field at `PrintConfig.hpp:1436`, definition lines 2603-2609, Ares kind `Floats`)
- `filament_loading_speed_start` (`coFloats`, default `3`, field at `PrintConfig.hpp:1437`, definition lines 2611-2617, Ares kind `Floats`)
- `filament_unloading_speed` (`coFloats`, default `90`, field at `PrintConfig.hpp:1438`, definition lines 2619-2626, Ares kind `Floats`)
- `filament_unloading_speed_start` (`coFloats`, default `100`, field at `PrintConfig.hpp:1439`, definition lines 2628-2634, Ares kind `Floats`)

## Functional requirements

1. Add the included missing options to sorted definition shards using `Ints` and `Floats`.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve sorted/no-duplicate test coverage across the merged table.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, wipe-tower loading/unloading behavior, ramming behavior, toolchange runtime behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter `filament_toolchange_delay`, `filament_cooling_moves`, or following options outside the included slice.
8. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Update roadmap and milestone docs so E2E parity moves to M65.
10. Keep modified Rust files under 400 LOC.

## Deferred behavior

- Upstream UI and validation metadata from `PrintConfig.cpp:2596-2634` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Wipe-tower loading/unloading, ramming, toolchange sequencing, cooling/stamping moves, slicing, extrusion, and G-code behavior are deferred to later source-cited milestones.
- `filament_toolchange_delay`, `filament_cooling_moves`, and following options from `PrintConfig.cpp:2636+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all five new keys have expected kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The merged definition stream remains sorted and binary-search compatible.
- Public lookup coverage exists for all five new keys.
- Plan/spec explicitly account for deferred UI metadata, wipe-tower behavior, ramming/toolchange behavior, slicing/extrusion/G-code behavior, and following `filament_toolchange_delay` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
