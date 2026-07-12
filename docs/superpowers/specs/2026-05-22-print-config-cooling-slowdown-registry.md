# M58 Spec: PrintConfig cooling slowdown option registry slice

## Goal
Port the adjacent FFF `libslic3r::PrintConfigDef::init_fff_params` cooling-slowdown option-definition slice into `ares-core` option registry metadata by adding registry coverage for `reduce_fan_stop_start_freq` and `dont_slow_down_outer_wall`, while splitting the registry definition table so all Rust files remain under 400 LOC.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1519`: `reduce_fan_stop_start_freq` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1520`: `dont_slow_down_outer_wall` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2334-2338`: `reduce_fan_stop_start_freq` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2340-2347`: `dont_slow_down_outer_wall` option definition.

Related upstream behavior explicitly deferred:

- Fan runtime behavior and layer-time slowdown behavior.
- Typed accessors or behavior changes for the newly registered keys.
- UI label/tooltip metadata beyond the current registry boundary.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2349+`: `fan_cooling_layer_time`, filament color/options, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table.rs`: include new shard modules and merge them without changing public APIs.
- `crates/ares-core/src/options/registry/definitions/table/early.rs`: keep only definitions through `compatible_process_expression_group`.
- `crates/ares-core/src/options/registry/definitions/table/pre_middle.rs`: new shard for definitions from `complete_print_exhaust_fan_speed` through `filament_flow_ratio`, including `dont_slow_down_outer_wall`.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: keep definitions from `is_infill_first` through `printhost_ssl_ignore_revoke`.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: new shard for definitions from `printhost_user` through `wall_sequence`, including `reduce_fan_stop_start_freq`.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/cooling.rs`: new metadata assertions for cooling-slowdown options.
- `crates/ares-core/src/options/registry/tests/metadata.rs`: include the new metadata module.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_cooling.rs`: public lookup coverage.
- `crates/ares-core/src/options/tests.rs`: include the new lookup module.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `dont_slow_down_outer_wall` (`coBools`, default `false`, field at `PrintConfig.hpp:1520`, definition lines 2340-2347)
- `reduce_fan_stop_start_freq` (`coBools`, default `false`, field at `PrintConfig.hpp:1519`, definition lines 2334-2338)

## Functional requirements

1. Add the included missing options to sorted definition shards using `OptionValueKind::Bools`.
2. Split the registry definition table so every modified Rust file remains under 400 LOC.
3. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
4. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
5. Preserve exact key, kind, default, and source metadata for all existing definitions moved during the shard split, including `filament_flow_ratio`; keep `middle.rs` starting at `first_layer_flow_ratio`.
6. Preserve sorted/no-duplicate test coverage across the merged table.
7. Preserve `SliceOptions` unknown-value storage and current public slicing API.
8. Do not add typed parsing/accessors, fan runtime behavior, layer-time slowdown behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
9. Do not add or alter `fan_cooling_layer_time`, filament color/options, or following options outside the included slice.
10. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
11. Update roadmap and milestone docs so E2E parity moves to M59.

## Deferred behavior

- Upstream UI metadata from `PrintConfig.cpp:2334-2347` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Fan runtime behavior, layer-time slowdown behavior, slicing behavior, extrusion behavior, and G-code behavior are deferred to later source-cited milestones.
- `fan_cooling_layer_time`, filament color/options, and following options from `PrintConfig.cpp:2349+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove both new keys have expected kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The merged definition stream remains sorted and binary-search compatible.
- Public lookup coverage exists for both new keys.
- Existing moved definitions keep exact metadata; the registry preserves all 173 existing definitions and adds exactly 2 new keys.
- Plan/spec explicitly account for shard splitting, deferred UI metadata, fan/layer-time runtime behavior, slicing/extrusion/G-code behavior, and following `fan_cooling_layer_time` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
