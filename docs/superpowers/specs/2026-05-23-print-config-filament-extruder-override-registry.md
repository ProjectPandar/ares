# M151 Spec: PrintConfig filament extruder override registry slice

## Goal
Port the generated `filament_extruder_override_keys` option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:512`: declaration of `filament_extruder_override_keys`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:63-83`: ordered list of generated filament override keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7121-7156`: loop that creates nullable filament override definitions by copying type/default metadata from the raw extruder option.
- Existing raw extruder option definition line ranges already cited in the Ares registry for inherited kind/default metadata.

Related upstream behavior explicitly deferred:

- Runtime filament override resolution and following behavior.
- Retraction, z-hop, wipe, cut-retraction, and toolpath planning behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7158-7165`: `detect_narrow_internal_solid_infill` option definition.
- Filesystem behavior, network behavior, UI behavior, slicing behavior, extrusion behavior, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle_filament.rs`: add the generated definitions that sort before `filament_ramming_parameters`.
- `crates/ares-core/src/options/registry/definitions/table/pre_middle_tail.rs`: add the generated definitions that sort after `filament_ramming_parameters` and before `file_start_gcode`.
- `crates/ares-core/src/options/registry/tests/keys/first.rs`: add all generated keys in sorted order.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/filament_extruder_override.rs`: add metadata assertions for all generated definitions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_filament_extruder_override.rs`: add public lookup assertions for all generated definitions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values.rs` and a new shard under `values/`: add fixtures without pushing `values.rs` to 400 LOC or above.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by 16.
- `docs/roadmap.md` and `docs/milestones/m151-print-config-filament-extruder-override-registry.md`: milestone sequencing docs.

## Included option definitions

- `filament_deretraction_speed` (`FloatsNullable`, default `0`, generated from `deretraction_speed`)
- `filament_long_retractions_when_cut` (`BoolsNullable`, default `false`, generated from `long_retractions_when_cut`)
- `filament_retract_before_wipe` (`PercentsNullable`, default `100`, generated from `retract_before_wipe`)
- `filament_retract_lift_above` (`FloatsNullable`, default `0`, generated from `retract_lift_above`)
- `filament_retract_lift_below` (`FloatsNullable`, default `0`, generated from `retract_lift_below`)
- `filament_retract_lift_enforce` (`EnumsNullable`, default `All Surfaces`, generated from `retract_lift_enforce`)
- `filament_retract_restart_extra` (`FloatsNullable`, default `0`, generated from `retract_restart_extra`)
- `filament_retract_when_changing_layer` (`BoolsNullable`, default `false`, generated from `retract_when_changing_layer`)
- `filament_retraction_distances_when_cut` (`FloatsNullable`, default `18`, generated from `retraction_distances_when_cut`)
- `filament_retraction_length` (`FloatsNullable`, default `0.8`, generated from `retraction_length`)
- `filament_retraction_minimum_travel` (`FloatsNullable`, default `2`, generated from `retraction_minimum_travel`)
- `filament_retraction_speed` (`FloatsNullable`, default `30`, generated from `retraction_speed`)
- `filament_wipe` (`BoolsNullable`, default `false`, generated from `wipe`)
- `filament_wipe_distance` (`FloatsNullable`, default `1`, generated from `wipe_distance`)
- `filament_z_hop` (`FloatsNullable`, default `0.4`, generated from `z_hop`)
- `filament_z_hop_types` (`EnumsNullable`, default `Slope Lift`, generated from `z_hop_types`)

## Functional requirements

1. Add the 16 missing generated options using existing nullable value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, runtime filament override resolution, retraction behavior, z-hop behavior, wipe behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `detect_narrow_internal_solid_infill` from `PrintConfig.cpp:7158-7165`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC; shard known-count fixtures if needed.

## Acceptance checks

- Registry tests prove all covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all covered definitions.
- Plan/spec explicitly account for deferred runtime override behavior and `detect_narrow_internal_solid_infill` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
