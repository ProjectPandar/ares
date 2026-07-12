# M41 Spec: PrintConfig one-wall quality option registry slice

## Goal
Port the next adjacent FFF-specific `libslic3r::PrintConfigDef::init_fff_params` option-definition slice into `ares-core` option registry metadata, covering one-wall and overhang-extra-perimeter quality options without changing wall planning, overhang detection, extrusion, or G-code behavior.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1176`: `only_one_wall_top` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1179-1180`: `min_width_top_surface` and `only_one_wall_first_layer` fields.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1188`: `precise_outer_wall` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1200`: `extra_perimeters_on_overhangs` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1404-1444`: `PrintConfigDef::init_fff_params()` option definitions for this slice.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/PerimeterGenerator.*`: wall spacing, one-wall top/first-layer decisions, and extra perimeter generation.
- `OrcaSlicer/src/libslic3r/Surface.*` and print/layer surface classification used by one-wall threshold behavior.
- `OrcaSlicer/src/libslic3r/GCode.cpp` and extrusion path emission affected by perimeter planning.
- `OrcaSlicer/src/libslic3r/Preset.cpp` preset option-list behavior and UI visibility rules.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp` object option override handling.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/early.rs`: sorted definition shard for `extra_perimeters_on_overhangs`.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: sorted definition shard for `min_width_top_surface`, `only_one_wall_first_layer`, `only_one_wall_top`, and `precise_outer_wall`.
- `crates/ares-core/src/options/registry/definitions/table.rs`: merged `OPTION_DEFINITIONS` boundary must remain unchanged.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/quality.rs`: focused quality metadata tests.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public lookup/count coverage.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `precise_outer_wall` (`coBool`, default `true`, lines 1404-1409)
- `only_one_wall_top` (`coBool`, default `false`, lines 1411-1415)
- `min_width_top_surface` (`coFloatOrPercent`, default `300%`, lines 1418-1431)
- `only_one_wall_first_layer` (`coBool`, default `false`, lines 1433-1437)
- `extra_perimeters_on_overhangs` (`coBool`, default `false`, lines 1439-1444)

## Functional requirements

1. Add the included options to sorted definition shards using existing `OptionValueKind::Bool` and `OptionValueKind::FloatOrPercent`.
2. Preserve public API: `OptionDefinition`, `option_definitions()`, and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve sorted/no-duplicate test coverage across the merged table.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, wall planning, one-wall top/first-layer behavior, top-surface width analysis, overhang extra perimeter generation, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter `overhang_reverse`, `overhang_reverse_internal_only`, or later quality/wall options from `PrintConfig.cpp:1446+`.
8. Do not add preset-list behavior, object override behavior, new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Update roadmap and milestone docs so E2E parity moves to M42, or verify those docs if the rename already exists in the current worktree.
10. Modified Rust files must remain under 400 LOC.

## Deferred behavior

- Upstream label/category/tooltip/sidetext/min/max/ratio-over/mode metadata from `PrintConfig.cpp:1404-1444` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Wall-spacing precision behavior is deferred to a later source-cited perimeter planning milestone.
- One-wall top/first-layer behavior and top-surface threshold behavior are deferred to later source-cited perimeter/surface milestones.
- Extra perimeter generation over overhangs is deferred to a later source-cited overhang/perimeter milestone.
- Preset option-list behavior in `Preset.cpp` and object override handling in `PrintObject.cpp` are deferred.
- `overhang_reverse`, `overhang_reverse_internal_only`, and following quality/wall options are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all five new keys, kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The merged definition stream remains sorted and binary-search compatible.
- Plan/spec explicitly account for deferred upstream UI metadata, wall planning, one-wall behavior, top-surface threshold behavior, extra perimeter generation, preset-list behavior, object override behavior, and following quality/wall options.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
