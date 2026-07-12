# M66 Spec: PrintConfig filament tower purge and interface option registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` filament minimal purge, wipe-tower cooling, and tower interface pre-extrusion/ironing option-definition slice into `ares-core` option registry metadata by adding registry coverage for `filament_minimal_purge_on_wipe_tower`, `filament_cooling_before_tower`, `filament_tower_interface_pre_extrusion_dist`, `filament_tower_interface_pre_extrusion_length`, and `filament_tower_ironing_area`.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1443`: `filament_minimal_purge_on_wipe_tower` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2678-2687`: `filament_minimal_purge_on_wipe_tower` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1444`: `filament_cooling_before_tower` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2689-2695`: `filament_cooling_before_tower` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1445`: `filament_tower_interface_pre_extrusion_dist` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2697-2703`: `filament_tower_interface_pre_extrusion_dist` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1446`: `filament_tower_interface_pre_extrusion_length` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2705-2711`: `filament_tower_interface_pre_extrusion_length` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1447`: `filament_tower_ironing_area` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2713-2719`: `filament_tower_ironing_area` option definition.

Related upstream behavior explicitly deferred:

- UI label/tooltip/sidetext/min/max/mode/nullable metadata beyond the current registry boundary, except nullable type identity through `FloatsNullable`.
- Wipe-tower purge, cooling, tower-interface pre-extrusion, and tower ironing behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2721+`: `filament_tower_interface_purge_volume`, `filament_tower_interface_print_temp`, `filament_cooling_final_speed`, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle.rs`: add sorted filament definitions while keeping file size below 400 LOC.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/filament.rs`: extend metadata assertions for the five options.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_filament.rs`: extend public lookup coverage for the five options.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `filament_minimal_purge_on_wipe_tower` (`coFloats`, default `15`, field at `PrintConfig.hpp:1443`, definition lines 2678-2687, Ares kind `Floats`)
- `filament_cooling_before_tower` (`coFloats`, `nullable = true`, default `10`, field at `PrintConfig.hpp:1444`, definition lines 2689-2695, Ares kind `FloatsNullable`)
- `filament_tower_interface_pre_extrusion_dist` (`coFloats`, default `10`, field at `PrintConfig.hpp:1445`, definition lines 2697-2703, Ares kind `Floats`)
- `filament_tower_interface_pre_extrusion_length` (`coFloats`, default `0`, field at `PrintConfig.hpp:1446`, definition lines 2705-2711, Ares kind `Floats`)
- `filament_tower_ironing_area` (`coFloats`, default `4`, field at `PrintConfig.hpp:1447`, definition lines 2713-2719, Ares kind `Floats`)

## Functional requirements

1. Add the included missing options to sorted definition shards using `Floats` and `FloatsNullable`.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve sorted/no-duplicate test coverage across the merged table.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, wipe-tower purge/cooling/interface behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter `filament_tower_interface_purge_volume`, `filament_tower_interface_print_temp`, `filament_cooling_final_speed`, or following options outside the included slice.
8. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Update roadmap and milestone docs so E2E parity moves to M67.
10. Keep modified Rust files under 400 LOC.

## Deferred behavior

- Upstream UI and validation metadata from `PrintConfig.cpp:2678-2719` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation; nullable identity is preserved through `FloatsNullable` only.
- Wipe-tower purge/cooling/interface runtime, slicing, extrusion, and G-code behavior are deferred to later source-cited milestones.
- `filament_tower_interface_purge_volume`, `filament_tower_interface_print_temp`, `filament_cooling_final_speed`, and following options from `PrintConfig.cpp:2721+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all five new keys have expected kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The merged definition stream remains sorted and binary-search compatible.
- Public lookup coverage exists for all five new keys.
- Plan/spec explicitly account for deferred UI metadata, wipe-tower behavior, slicing/extrusion/G-code behavior, and following `filament_tower_interface_purge_volume` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
