# M67 Spec: PrintConfig filament tower temperature and final cooling option registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` tower interface purge volume, tower interface print temperature, and final cooling speed option-definition slice into `ares-core` option registry metadata by adding registry coverage for `filament_tower_interface_purge_volume`, `filament_tower_interface_print_temp`, and `filament_cooling_final_speed`, while splitting the existing `pre_middle` registry shard so all Rust files remain below 400 LOC.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1448`: `filament_tower_interface_purge_volume` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2721-2727`: `filament_tower_interface_purge_volume` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1449`: `filament_tower_interface_print_temp` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2729-2735`: `filament_tower_interface_print_temp` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1450`: `filament_cooling_final_speed` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2737-2743`: `filament_cooling_final_speed` option definition.

Related upstream behavior explicitly deferred:

- UI label/tooltip/sidetext/min/max/mode metadata beyond the current registry boundary.
- Wipe-tower purge volume, interface print temperature selection, final cooling move behavior, and runtime toolchange behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2745+`: `filament_ramming_parameters`, `filament_multitool_ramming`, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table.rs`: include a new sorted shard in merge order after `pre_middle` and before `middle`.
- `crates/ares-core/src/options/registry/definitions/table/pre_middle.rs`: add sorted `filament_cooling_final_speed` and move the existing tail beginning at `filament_stamping_distance` into the new shard without changing metadata.
- `crates/ares-core/src/options/registry/definitions/table/pre_middle_tail.rs`: create the new sorted shard containing moved existing tail definitions plus `filament_tower_interface_print_temp` and `filament_tower_interface_purge_volume`.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/filament.rs`: extend metadata assertions for the three options.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_filament.rs`: extend public lookup coverage for the three options.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `filament_tower_interface_purge_volume` (`coFloats`, default `20`, field at `PrintConfig.hpp:1448`, definition lines 2721-2727, Ares kind `Floats`)
- `filament_tower_interface_print_temp` (`coInts`, default `-1`, field at `PrintConfig.hpp:1449`, definition lines 2729-2735, Ares kind `Ints`)
- `filament_cooling_final_speed` (`coFloats`, default `3.4`, field at `PrintConfig.hpp:1450`, definition lines 2737-2743, Ares kind `Floats`)

## Functional requirements

1. Split the registry table shard without changing existing option keys, kinds, defaults, or source citations; moved definitions must be copied unchanged.
2. Add the included missing options to sorted definition shards using `Floats` and `Ints`.
3. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
4. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
5. Preserve sorted/no-duplicate test coverage across the merged table.
6. Preserve `SliceOptions` unknown-value storage and current public slicing API.
7. Do not add typed parsing/accessors, wipe-tower purge/temperature/final-cooling behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
8. Do not add or alter `filament_ramming_parameters`, `filament_multitool_ramming`, or following options outside the included slice.
9. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
10. Update roadmap and milestone docs so E2E parity moves to M68.
11. Keep modified Rust files under 400 LOC.

## Deferred behavior

- Upstream UI and validation metadata from `PrintConfig.cpp:2721-2743` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Wipe-tower purge volume, interface print temperature, final cooling runtime, slicing, extrusion, and G-code behavior are deferred to later source-cited milestones.
- `filament_ramming_parameters`, `filament_multitool_ramming`, and following options from `PrintConfig.cpp:2745+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all three new keys have expected kinds, default values, and source line references.
- The shard split preserves existing metadata unchanged and the merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all three new keys.
- Plan/spec explicitly account for deferred UI metadata, wipe-tower behavior, slicing/extrusion/G-code behavior, unchanged moved-tail metadata, and following ramming scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
