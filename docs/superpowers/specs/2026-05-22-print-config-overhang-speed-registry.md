# M43 Spec: PrintConfig overhang speed option registry slice

## Goal
Port the next adjacent FFF-specific `libslic3r::PrintConfigDef::init_fff_params` option-definition slice into `ares-core` option registry metadata, covering overhang speed and curled-perimeter slowdown options without changing speed planning, extrusion, or G-code behavior.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1171-1175`: fields for `enable_overhang_speed` and `overhang_1_4_speed` through `overhang_4_4_speed`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1201`: field for `slowdown_for_curled_perimeters`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1500-1570`: `PrintConfigDef::init_fff_params()` option definitions for this slice.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/PerimeterGenerator.*`: overhang degree classification and overhang wall speed selection.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp` and print/layer analysis that detect curled or overhanging perimeter conditions.
- `OrcaSlicer/src/libslic3r/GCode.cpp`: downstream speed/extrusion/G-code effects.
- `OrcaSlicer/src/libslic3r/Preset.cpp`: preset option-list behavior and UI visibility rules.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/early.rs`: sorted definition shard for `enable_overhang_speed`.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: sorted definition shard for `overhang_1_4_speed`, `overhang_2_4_speed`, `overhang_3_4_speed`, `overhang_4_4_speed`, and `slowdown_for_curled_perimeters`.
- `crates/ares-core/src/options/registry/definitions/table.rs`: merged `OPTION_DEFINITIONS` boundary must remain unchanged.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/speed.rs`: focused speed metadata tests.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public lookup/count coverage.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `enable_overhang_speed` (`coBool`, default `true`, lines 1500-1505)
- `slowdown_for_curled_perimeters` (`coBool`, default `true`, lines 1507-1522)
- `overhang_1_4_speed` (`coFloatOrPercent`, default `0`, lines 1524-1534)
- `overhang_2_4_speed` (`coFloatOrPercent`, default `0`, lines 1536-1546)
- `overhang_3_4_speed` (`coFloatOrPercent`, default `0`, lines 1548-1558)
- `overhang_4_4_speed` (`coFloatOrPercent`, default `0`, lines 1560-1570)

## Functional requirements

1. Add the included options to sorted definition shards using existing `OptionValueKind::Bool` and `OptionValueKind::FloatOrPercent`.
2. Preserve public API: `OptionDefinition`, `option_definitions()`, and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve sorted/no-duplicate test coverage across the merged table.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, overhang speed selection, curled-perimeter slowdown behavior, speed planning, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter existing `bridge_speed`, `internal_bridge_speed`, or later speed options from `PrintConfig.cpp:1572+`.
8. Do not add preset-list behavior, object override behavior, new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Update roadmap and milestone docs so E2E parity moves to M44, or verify those docs if the rename already exists in the current worktree.
10. Modified Rust files must remain under 400 LOC.

## Deferred behavior

- Upstream label/category/full-label/tooltip/sidetext/min/ratio-over/mode metadata from `PrintConfig.cpp:1500-1570` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Overhang degree classification and speed selection are deferred to a later source-cited speed/perimeter milestone.
- Curled-perimeter slowdown behavior is deferred to a later source-cited perimeter/speed milestone.
- Speed assignment, extrusion behavior, and G-code behavior are deferred.
- Preset option-list behavior in `Preset.cpp` and object override handling are deferred.
- Existing `bridge_speed`, `internal_bridge_speed`, and following speed options are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all six new keys, kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The merged definition stream remains sorted and binary-search compatible.
- Plan/spec explicitly account for deferred upstream UI metadata, overhang speed behavior, curled-perimeter slowdown behavior, speed assignment, preset-list behavior, object override behavior, and following speed options.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
