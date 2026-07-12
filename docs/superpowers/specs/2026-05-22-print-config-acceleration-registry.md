# M75 Spec: PrintConfig acceleration option registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` acceleration and accel-to-decel option-definition slice into `ares-core` option registry metadata by adding registry coverage for `inner_wall_acceleration`, `travel_acceleration`, `top_surface_acceleration`, `outer_wall_acceleration`, `bridge_acceleration`, `sparse_infill_acceleration`, `internal_solid_infill_acceleration`, `initial_layer_acceleration`, `initial_layer_travel_acceleration`, `accel_to_decel_enable`, and `accel_to_decel_factor`.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1044`: `inner_wall_acceleration` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3068-3075`: `inner_wall_acceleration` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1048`: `travel_acceleration` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3077-3084`: `travel_acceleration` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1045`: `top_surface_acceleration` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3086-3093`: `top_surface_acceleration` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1043`: `outer_wall_acceleration` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3095-3102`: `outer_wall_acceleration` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1047`: `bridge_acceleration` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3104-3112`: `bridge_acceleration` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1049`: `sparse_infill_acceleration` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3114-3122`: `sparse_infill_acceleration` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1050`: `internal_solid_infill_acceleration` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3124-3132`: `internal_solid_infill_acceleration` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1046`: `initial_layer_acceleration` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3134-3141`: `initial_layer_acceleration` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1422`: `initial_layer_travel_acceleration` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3143-3150`: `initial_layer_travel_acceleration` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1419`: `accel_to_decel_enable` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3152-3157`: `accel_to_decel_enable` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1420`: `accel_to_decel_factor` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3159-3167`: `accel_to_decel_factor` option definition.

Related upstream behavior explicitly deferred:

- UI label/category/tooltip/sidetext/min/max/mode/ratio metadata beyond the current registry boundary.
- Acceleration resolution and accel-to-decel runtime behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3169+`: `default_jerk`, `default_junction_deviation`, wall/infill jerk, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/early.rs`: add sorted definitions for `accel_to_decel_enable`, `accel_to_decel_factor`, and `bridge_acceleration`.
- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: add sorted definitions for `initial_layer_acceleration`, `initial_layer_travel_acceleration`, `inner_wall_acceleration`, and `internal_solid_infill_acceleration`.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: add sorted definition for `outer_wall_acceleration`.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add sorted definitions for `sparse_infill_acceleration`, `top_surface_acceleration`, and `travel_acceleration`.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/speed.rs`: extend source metadata assertions for the eleven options.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_speed.rs`: extend public lookup coverage for the eleven options.
- `docs/roadmap.md` and `docs/milestones/m75-print-config-acceleration-registry.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `inner_wall_acceleration` (`coFloat`, default `10000`, field at `PrintConfig.hpp:1044`, definition lines 3068-3075, Ares kind `Float`)
- `travel_acceleration` (`coFloat`, default `10000`, field at `PrintConfig.hpp:1048`, definition lines 3077-3084, Ares kind `Float`)
- `top_surface_acceleration` (`coFloat`, default `500`, field at `PrintConfig.hpp:1045`, definition lines 3086-3093, Ares kind `Float`)
- `outer_wall_acceleration` (`coFloat`, default `500`, field at `PrintConfig.hpp:1043`, definition lines 3095-3102, Ares kind `Float`)
- `bridge_acceleration` (`coFloatOrPercent`, default `50%`, field at `PrintConfig.hpp:1047`, definition lines 3104-3112, Ares kind `FloatOrPercent`)
- `sparse_infill_acceleration` (`coFloatOrPercent`, default `100%`, field at `PrintConfig.hpp:1049`, definition lines 3114-3122, Ares kind `FloatOrPercent`)
- `internal_solid_infill_acceleration` (`coFloatOrPercent`, default `100%`, field at `PrintConfig.hpp:1050`, definition lines 3124-3132, Ares kind `FloatOrPercent`)
- `initial_layer_acceleration` (`coFloat`, default `300`, field at `PrintConfig.hpp:1046`, definition lines 3134-3141, Ares kind `Float`)
- `initial_layer_travel_acceleration` (`coFloatOrPercent`, default `100%`, field at `PrintConfig.hpp:1422`, definition lines 3143-3150, Ares kind `FloatOrPercent`)
- `accel_to_decel_enable` (`coBool`, default `true`, field at `PrintConfig.hpp:1419`, definition lines 3152-3157, Ares kind `Bool`)
- `accel_to_decel_factor` (`coPercent`, default `50`, field at `PrintConfig.hpp:1420`, definition lines 3159-3167, Ares kind `Percent`)

## Functional requirements

1. Add the missing options to existing sorted definition shards using `Bool`, `Percent`, `Float`, and `FloatOrPercent`.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not alter the existing `default_acceleration` registration in this milestone.
6. Do not add typed parsing/accessors, acceleration behavior, accel-to-decel behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter `default_jerk`, `default_junction_deviation`, wall/infill jerk, or following options outside the included slice.
8. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
10. Keep modified Rust files under 400 LOC.

## Deferred behavior

- Upstream UI, validation, mode, ratio, and GUI metadata from `PrintConfig.cpp:3068-3167` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Acceleration resolution, accel-to-decel behavior, slicing, extrusion, and G-code behavior are deferred to later source-cited milestones.
- `default_jerk`, `default_junction_deviation`, wall/infill jerk, and following options from `PrintConfig.cpp:3169+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all eleven new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all eleven new keys.
- Plan/spec explicitly account for deferred UI/bounds/ratio metadata, acceleration behavior, accel-to-decel behavior, slicing/extrusion/G-code behavior, and following jerk scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
