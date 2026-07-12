# M76 Spec: PrintConfig wall, infill, and travel jerk option registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` wall, infill, top surface, first layer, travel, and first-layer travel jerk option-definition slice into `ares-core` option registry metadata by adding registry coverage for `outer_wall_jerk`, `inner_wall_jerk`, `top_surface_jerk`, `infill_jerk`, `initial_layer_jerk`, `travel_jerk`, and `initial_layer_travel_jerk`.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1053`: `outer_wall_jerk` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3188-3195`: `outer_wall_jerk` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1054`: `inner_wall_jerk` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3197-3204`: `inner_wall_jerk` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1056`: `top_surface_jerk` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3206-3213`: `top_surface_jerk` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1055`: `infill_jerk` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3215-3222`: `infill_jerk` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1057`: `initial_layer_jerk` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3224-3231`: `initial_layer_jerk` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1058`: `travel_jerk` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3233-3240`: `travel_jerk` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1423`: `initial_layer_travel_jerk` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3242-3249`: `initial_layer_travel_jerk` option definition.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3169-3186`: `default_jerk` and `default_junction_deviation`, deferred because sorted `default_*` insertion requires a separate pre-middle registry shard split to preserve the 400 LOC rule.
- UI label/category/tooltip/sidetext/min/mode/ratio metadata beyond the current registry boundary.
- Jerk resolution and first-layer travel jerk ratio runtime behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3251+`: `initial_layer_line_width`, `initial_layer_print_height`, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: add sorted definitions for `infill_jerk`, `initial_layer_jerk`, `initial_layer_travel_jerk`, and `inner_wall_jerk`.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: add sorted definition for `outer_wall_jerk`.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add sorted definitions for `top_surface_jerk` and `travel_jerk`.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/speed.rs`: extend source metadata assertions for the seven options.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_speed.rs`: extend public lookup coverage for the seven options.
- `docs/roadmap.md` and `docs/milestones/m76-print-config-jerk-registry.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `outer_wall_jerk` (`coFloat`, default `9`, field at `PrintConfig.hpp:1053`, definition lines 3188-3195, Ares kind `Float`)
- `inner_wall_jerk` (`coFloat`, default `9`, field at `PrintConfig.hpp:1054`, definition lines 3197-3204, Ares kind `Float`)
- `top_surface_jerk` (`coFloat`, default `9`, field at `PrintConfig.hpp:1056`, definition lines 3206-3213, Ares kind `Float`)
- `infill_jerk` (`coFloat`, default `9`, field at `PrintConfig.hpp:1055`, definition lines 3215-3222, Ares kind `Float`)
- `initial_layer_jerk` (`coFloat`, default `9`, field at `PrintConfig.hpp:1057`, definition lines 3224-3231, Ares kind `Float`)
- `travel_jerk` (`coFloat`, default `12`, field at `PrintConfig.hpp:1058`, definition lines 3233-3240, Ares kind `Float`)
- `initial_layer_travel_jerk` (`coFloatOrPercent`, default `100%`, field at `PrintConfig.hpp:1423`, definition lines 3242-3249, Ares kind `FloatOrPercent`)

## Functional requirements

1. Add the missing options to existing sorted definition shards using `Float` and `FloatOrPercent`.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not alter `default_jerk` or `default_junction_deviation` in this milestone.
6. Do not add typed parsing/accessors, jerk behavior, ratio behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter `initial_layer_line_width`, `initial_layer_print_height`, or following options outside the included slice.
8. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
10. Keep modified Rust files under 400 LOC.

## Deferred behavior

- Upstream UI, validation, mode, ratio, and GUI metadata from `PrintConfig.cpp:3188-3249` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Jerk resolution, first-layer travel jerk ratio behavior, slicing, extrusion, and G-code behavior are deferred to later source-cited milestones.
- `default_jerk` and `default_junction_deviation` from `PrintConfig.cpp:3169-3186` are deferred to a separate source-cited registry-shard milestone.
- `initial_layer_line_width`, `initial_layer_print_height`, and following options from `PrintConfig.cpp:3251+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all seven new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all seven new keys.
- Plan/spec explicitly account for deferred UI/bounds/ratio metadata, jerk behavior, slicing/extrusion/G-code behavior, deferred default jerk/junction-deviation scope, and following initial-layer line-width scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
