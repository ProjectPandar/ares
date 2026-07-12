# M104 Spec: PrintConfig Z-hop and extruder/nozzle type registry slice

## Goal
Port the adjacent Z-hop, lift-boundary, lift-type, travel-slope, lift-enforcement, extruder type, nozzle-volume type, and default nozzle-volume type option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1375`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5122-5131`: `z_hop` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1379`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5133-5139` and duplicate definition at `PrintConfig.cpp:5173-5178`: `retract_lift_above` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1380`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5141-5147` and duplicate definition at `PrintConfig.cpp:5180-5185`: `retract_lift_below` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:382-388`, `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1377`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:526-532`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5149-5162`: `z_hop_types` option definition and enum map.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1378`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5164-5171`: `travel_slope` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:390-394`, `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1381`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:534-540`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5187-5200`: `retract_lift_enforce` option definition and enum map.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:412-415`, `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1408`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:565-569`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5202-5212`: `extruder_type` option definition and enum map.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:418-421`, `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1409`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:571-575`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5215-5225`: `nozzle_volume_type` option definition and enum map.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:418-421`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:571-575`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5227-5237`: `default_nozzle_volume_type` option definition and enum map.

Related upstream behavior explicitly deferred:

- UI full-label/tooltip/sidetext/min/max/mode metadata beyond the current registry boundary.
- Z-hop movement generation, slope/spiral lift travel behavior, surface-based lift enforcement, and emitted G-code changes.
- Extruder/nozzle variant resolution, `get_extruder_variant_string`, `get_nozzle_volume_type_string`, preset/project `default_nozzle_volume_type` synchronization, and calibration behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5239+`: `extruder_variant_list`, `extruder_ams_count`, `printer_extruder_id`, `printer_extruder_variant`, `master_extruder_id`, and following options.
- Slicing, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle_defaults.rs`: add `default_nozzle_volume_type` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`: add `extruder_type` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/tail_final.rs`: add `travel_slope`, `z_hop`, and `z_hop_types` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/late_tail.rs`: add `nozzle_volume_type` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add `retract_lift_above`, `retract_lift_below`, and `retract_lift_enforce` in sorted order.
- Registry key, metadata, fixture-count, and public lookup tests cover all nine definitions.
- `docs/roadmap.md` and `docs/milestones/m104-print-config-zhop-extruder-type-registry.md`: milestone sequencing docs.

## Included option definitions

- `z_hop` (`coFloats`, default `0.4`, field at `PrintConfig.hpp:1375`, definition lines 5122-5131, Ares kind `Floats`)
- `retract_lift_above` (`coFloats`, default `0`, field at `PrintConfig.hpp:1379`, definition lines 5133-5139 and duplicate 5173-5178, Ares kind `Floats`)
- `retract_lift_below` (`coFloats`, default `0`, field at `PrintConfig.hpp:1380`, definition lines 5141-5147 and duplicate 5180-5185, Ares kind `Floats`)
- `z_hop_types` (`coEnums`, default `Slope Lift`, field at `PrintConfig.hpp:1377`, enum map lines 526-532, definition lines 5149-5162, Ares kind `Enums`)
- `travel_slope` (`coFloats`, default `3`, field at `PrintConfig.hpp:1378`, definition lines 5164-5171, Ares kind `Floats`)
- `retract_lift_enforce` (`coEnums`, default `All Surfaces`, field at `PrintConfig.hpp:1381`, enum map lines 534-540, definition lines 5187-5200, Ares kind `Enums`)
- `extruder_type` (`coEnums`, default `Direct Drive`, field at `PrintConfig.hpp:1408`, enum map lines 565-569, definition lines 5202-5212, Ares kind `Enums`)
- `nozzle_volume_type` (`coEnums`, default `Standard`, field at `PrintConfig.hpp:1409`, enum map lines 571-575, definition lines 5215-5225, Ares kind `Enums`)
- `default_nozzle_volume_type` (`coEnums`, default `Standard`, enum map lines 571-575, definition lines 5227-5237, Ares kind `Enums`)

## Functional requirements

1. Add the nine missing options to sorted definition shards using existing value kinds only.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, Z-hop movement behavior, lift-enforcement behavior, extruder/nozzle variant resolution, preset synchronization, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add or alter following `extruder_variant_list`, `extruder_ams_count`, `printer_extruder_id`, `printer_extruder_variant`, `master_extruder_id`, or later options from `PrintConfig.cpp:5239+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the nine new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all nine covered definitions.
- Plan/spec explicitly account for deferred UI metadata, Z-hop/lift/extruder/nozzle runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:5239+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
