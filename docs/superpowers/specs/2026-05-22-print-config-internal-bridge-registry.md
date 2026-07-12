# M49 Spec: PrintConfig internal bridge option registry slice

## Goal
Port the adjacent FFF-specific `libslic3r::PrintConfigDef::init_fff_params` bridge/internal-bridge option-definition slice into `ares-core` option registry metadata, covering existing bridge flags and missing internal bridge registry keys without changing bridge detection, filtering, layer generation, support decisions, slicing, extrusion, or G-code behavior.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:231-238`: `InternalBridgeFilter` and `EnableExtraBridgeLayer` enum definitions.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:928`: `bridge_no_support` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:986-990`: `thick_bridges`, `thick_internal_bridges`, `dont_filter_internal_bridges`, and `enable_extra_bridge_layer` fields.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:932`: `max_bridge_length` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:377-390`: enum key maps for `InternalBridgeFilter` and `EnableExtraBridgeLayer`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1847-1938`: `PrintConfigDef::init_fff_params()` option definitions for this slice.

Related upstream behavior explicitly deferred:

- Bridge support decisions and bridge-length support behavior.
- Internal bridge detection/filtering and extra bridge layer generation.
- Bridge geometry, layer generation, extrusion behavior, support generation, and G-code behavior.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1940+`: `machine_end_gcode` and following options.
- UI labels, enum labels, mode behavior, preset/profile behavior, filesystem/network integrations, slicing, extrusion, and G-code behavior.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/early.rs`: sorted definitions for `dont_filter_internal_bridges` and `enable_extra_bridge_layer`, plus updated citations for existing early bridge metadata.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: sorted definitions for `max_bridge_length`, existing moved `max_layer_height` / `max_travel_detour_distance`, and `thick_internal_bridges`, plus updated citations for existing `thick_bridges`.
- `crates/ares-core/src/options/registry/definitions/table.rs`: merged `OPTION_DEFINITIONS` boundary must remain unchanged except for including the same sorted definitions from adjusted shards.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/quality.rs`: focused metadata assertions for bridge/internal-bridge keys.
- `crates/ares-core/src/options/tests/registry_helpers.rs` and `registry_lookup.rs`: public lookup/count coverage.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Existing registry keys to keep and source-cover:

- `bridge_no_support` (`coBool`, default `false`, field at `PrintConfig.hpp:928`, definition lines 1847-1853)
- `thick_bridges` (`coBool`, default `false`, field at `PrintConfig.hpp:986`, definition lines 1855-1861)

Add registry metadata for these exact upstream options and default values:

- `thick_internal_bridges` (`coBool`, default `true`, field at `PrintConfig.hpp:987`, lines 1863-1869)
- `enable_extra_bridge_layer` (`coEnum`, default `disabled`, enum at `PrintConfig.hpp:236-238`, field at `PrintConfig.hpp:990`, enum map lines 384-390, definition lines 1871-1900)
- `dont_filter_internal_bridges` (`coEnum`, default `disabled`, enum at `PrintConfig.hpp:231-233`, field at `PrintConfig.hpp:988`, enum map lines 377-382, definition lines 1902-1928)
- `max_bridge_length` (`coFloat`, default `10`, field at `PrintConfig.hpp:932`, lines 1931-1938)

## Functional requirements

1. Add missing included options to sorted definition shards using existing `OptionValueKind::Bool`, `Enum`, and `Float`.
2. Update existing `bridge_no_support` and `thick_bridges` source citations to include their `PrintConfig.hpp` field lines without changing kind/default.
3. Preserve public API: `OptionDefinition`, `option_definitions()`, and `option_definition(key)` remain unchanged.
4. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
5. Preserve sorted/no-duplicate test coverage across the merged table.
6. Preserve `SliceOptions` unknown-value storage and current public slicing API.
7. Keep modified Rust files under 400 LOC by moving existing `max_layer_height` and `max_travel_detour_distance` definitions from `early.rs` to the start of `late.rs`, and inserting `max_bridge_length` before them. This is a shard-only move; key/kind/default/source for the moved existing definitions must stay unchanged.
8. Do not add typed parsing/accessors, bridge support behavior, internal bridge filtering behavior, extra bridge layer generation, max-bridge support behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
9. Do not add or alter `machine_end_gcode` or following options from `PrintConfig.cpp:1940+`.
10. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
11. Update roadmap and milestone docs so E2E parity moves to M50, or verify those docs if the rename already exists in the current worktree.

## Deferred behavior

- Upstream label/category/tooltip/enum-label/sidetext/min/mode metadata from `PrintConfig.cpp:1847-1938` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Bridge support decisions, internal bridge filtering, extra bridge layer generation, max-bridge support behavior, bridge geometry, slicing behavior, support generation, extrusion behavior, and G-code behavior are deferred to later source-cited bridge/print lifecycle milestones.
- `machine_end_gcode` and following options from `PrintConfig.cpp:1940+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all four new keys plus the two existing covered bridge keys have expected kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The merged definition stream remains sorted and binary-search compatible after the shard-only movement of existing max definitions.
- Public lookup coverage exists for all four new keys and the updated existing bridge keys.
- Plan/spec explicitly account for deferred upstream UI metadata, bridge filtering/generation/support behavior, slicing/extrusion/G-code behavior, and following `machine_end_gcode` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
