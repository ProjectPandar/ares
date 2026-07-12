# M42 Spec: PrintConfig overhang reversal option registry slice

## Goal
Port the next adjacent FFF-specific `libslic3r::PrintConfigDef::init_fff_params` option-definition slice into `ares-core` option registry metadata, covering overhang reversal and counterbore hole bridging options without changing perimeter planning, overhang detection, extrusion, or G-code behavior.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:401-403`: `CounterboreHoleBridgingOption` enum values.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1205-1208`: fields for `overhang_reverse`, `overhang_reverse_internal_only`, `overhang_reverse_threshold`, and `counterbore_hole_bridging`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:551-556`: counterbore enum key map.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1446-1498`: `PrintConfigDef::init_fff_params()` option definitions for this slice.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/PerimeterGenerator.*`: overhang reversal path ordering and counterbore bridge/perimeter generation.
- `OrcaSlicer/src/libslic3r/Layer.cpp:265+`: layer-level counterbore bridging checks.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:1544+`: object/layer counterbore handling.
- `OrcaSlicer/src/libslic3r/GCode.cpp`: downstream extrusion/G-code effects.
- `OrcaSlicer/src/libslic3r/Preset.cpp`: preset option-list behavior and UI visibility rules.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/early.rs`: sorted definition shard for `counterbore_hole_bridging`.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: sorted definition shard for `overhang_reverse`, `overhang_reverse_internal_only`, and `overhang_reverse_threshold`.
- `crates/ares-core/src/options/registry/definitions/table.rs`: merged `OPTION_DEFINITIONS` boundary must remain unchanged.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata/quality.rs`: focused quality metadata tests.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public lookup/count coverage.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `overhang_reverse` (`coBool`, default `false`, lines 1446-1452)
- `overhang_reverse_internal_only` (`coBool`, default `false`, lines 1454-1465)
- `counterbore_hole_bridging` (`coEnum`, default `none`, lines 1467-1483, enum map lines 551-556)
- `overhang_reverse_threshold` (`coFloatOrPercent`, default `50%`, lines 1485-1498)

## Functional requirements

1. Add the included options to sorted definition shards using existing `OptionValueKind::Bool`, `OptionValueKind::Enum`, and `OptionValueKind::FloatOrPercent`.
2. Preserve public API: `OptionDefinition`, `option_definitions()`, and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve sorted/no-duplicate test coverage across the merged table.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, overhang reversal behavior, counterbore bridge/perimeter behavior, wall planning, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter `enable_overhang_speed`, `slowdown_for_curled_perimeters`, or later speed/quality options from `PrintConfig.cpp:1500+`.
8. Do not add preset-list behavior, object override behavior, new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Update roadmap and milestone docs so E2E parity moves to M43, or verify those docs if the rename already exists in the current worktree.
10. Modified Rust files must remain under 400 LOC.

## Deferred behavior

- Upstream enum labels, label/category/tooltip/sidetext/min/max/ratio-over/mode metadata from `PrintConfig.cpp:1446-1498` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Overhang reversal path ordering, internal-only filtering, and reversal threshold behavior are deferred to a later source-cited perimeter planning milestone.
- Counterbore hole bridge/perimeter generation is deferred to a later source-cited perimeter/layer milestone.
- Preset option-list behavior in `Preset.cpp` and object override handling in `PrintObject.cpp` are deferred.
- `enable_overhang_speed`, `slowdown_for_curled_perimeters`, and following speed/quality options are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all four new keys, kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The merged definition stream remains sorted and binary-search compatible.
- Plan/spec explicitly account for deferred upstream enum labels/UI metadata, overhang reversal behavior, counterbore bridge behavior, preset-list behavior, object override behavior, and following speed/quality options.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
