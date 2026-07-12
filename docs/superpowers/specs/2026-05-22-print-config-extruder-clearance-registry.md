# M52 Spec: PrintConfig extruder clearance option registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` extruder-clearance and nozzle-height option-definition slice into `ares-core` option registry metadata by adding registry coverage for `extruder_clearance_height_to_rod`, `extruder_clearance_height_to_lid`, `extruder_clearance_radius`, and `nozzle_height`, while splitting registry shards to keep every modified Rust file under 400 LOC.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1513`: `extruder_clearance_height_to_rod` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1514`: `extruder_clearance_height_to_lid` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1515`: `extruder_clearance_radius` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1516`: `nozzle_height` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2127-2134`: `extruder_clearance_height_to_rod` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2137-2144`: `extruder_clearance_height_to_lid` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2146-2152`: `extruder_clearance_radius` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2154-2160`: `nozzle_height` option definition.

Related upstream behavior explicitly deferred:

- Collision-avoidance behavior for by-object printing.
- Nozzle-height runtime behavior.
- Typed accessors or behavior changes for the newly registered keys.
- UI label/category/tooltip/sidetext/min/mode metadata.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2112-2125`: `extruder`, because this registry boundary requires an explicit default value and upstream does not set one there.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2162+`: `bed_mesh_min` and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table.rs`: add a `middle` shard to the existing merge while preserving the public `OPTION_DEFINITIONS` slice and binary-search compatibility.
- `crates/ares-core/src/options/registry/definitions/table/early.rs`: add sorted `extruder_clearance_height_to_lid`, `extruder_clearance_height_to_rod`, and `extruder_clearance_radius`; move the existing `first_layer_flow_ratio` through `internal_solid_infill_pattern` block into the new middle shard so `early.rs` stays under 400 LOC.
- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: new sorted shard containing the moved `first_layer_flow_ratio` through `internal_solid_infill_pattern` definitions with exact existing metadata.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: add sorted `nozzle_height` after `nozzle_diameter`.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/hardware.rs`: new metadata assertions for extruder clearance/nozzle height so existing metadata files are not bloated.
- `crates/ares-core/src/options/registry/tests/metadata.rs`: include the new hardware metadata test module.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_hardware.rs`: new public lookup coverage file.
- `crates/ares-core/src/options/tests.rs`: include the new lookup test module.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `extruder_clearance_height_to_rod` (`coFloat`, default `40`, field at `PrintConfig.hpp:1513`, definition lines 2127-2134)
- `extruder_clearance_height_to_lid` (`coFloat`, default `120`, field at `PrintConfig.hpp:1514`, definition lines 2137-2144)
- `extruder_clearance_radius` (`coFloat`, default `40`, field at `PrintConfig.hpp:1515`, definition lines 2146-2152)
- `nozzle_height` (`coFloat`, default `2.5`, field at `PrintConfig.hpp:1516`, definition lines 2154-2160)

## Functional requirements

1. Add the included missing options to sorted definition shards using existing `OptionValueKind::Float`.
2. Preserve public API: `OptionDefinition`, `option_definitions()`, and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve sorted/no-duplicate test coverage across the merged table.
5. Preserve exact metadata for all definitions moved from `early.rs` into `middle.rs`.
6. Preserve `SliceOptions` unknown-value storage and current public slicing API.
7. Do not add typed parsing/accessors, collision-avoidance behavior, by-object scheduling, nozzle-height behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
8. Do not add or alter `extruder`, `bed_mesh_min`, or following options outside the included slice.
9. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
10. Update roadmap and milestone docs so E2E parity moves to M53.
11. Keep modified Rust files under 400 LOC by adding a middle shard and new focused test files instead of growing near-limit files.

## Deferred behavior

- Upstream UI and validation metadata from `PrintConfig.cpp:2127-2160` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Collision avoidance, by-object scheduling, nozzle-height behavior, slicing behavior, extrusion behavior, and G-code behavior are deferred to later source-cited milestones.
- `extruder` from `PrintConfig.cpp:2112-2125` is deferred until the registry can represent an upstream option without an explicit default or a later source-cited decision defines its default semantics.
- `bed_mesh_min` and following options from `PrintConfig.cpp:2162+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all four new keys have expected kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The merged definition stream remains sorted and binary-search compatible.
- Public lookup coverage exists for all four new keys in a new focused test module.
- Moved definitions from `early.rs` to `middle.rs` keep exact key/kind/default/source metadata.
- Plan/spec explicitly account for deferred upstream UI metadata, collision avoidance, nozzle-height behavior, slicing/extrusion/G-code behavior, skipped `extruder`, and following bed-mesh scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
