# M44 Spec: PrintConfig brim flow and combine option registry slice

## Goal
Port the next unregistered adjacent FFF-specific `libslic3r::PrintConfigDef::init_fff_params` brim option-definition slice into `ares-core` option registry metadata, covering brim flow, elephant-foot-compensated outline alignment, and brim combining without changing brim generation, extrusion, or G-code behavior.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:921-922`: fields for `brim_flow_ratio` and `brim_use_efc_outline`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1619`: field for `combine_brims`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1637-1663`: `PrintConfigDef::init_fff_params()` option definitions for this slice.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/Brim.*` and `SkirtBrim.*`: brim path generation, brim flow use, EFC outline alignment, and brim combining.
- `OrcaSlicer/src/libslic3r/ElephantFootCompensation.*`: compensated first-layer outline behavior.
- `OrcaSlicer/src/libslic3r/GCode.cpp`: downstream extrusion/G-code effects.
- `OrcaSlicer/src/libslic3r/Preset.cpp`: preset option-list behavior and UI visibility rules.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/early.rs`: sorted definition shard for `brim_flow_ratio`, `brim_use_efc_outline`, and `combine_brims`.
- `crates/ares-core/src/options/registry/definitions/table.rs`: merged `OPTION_DEFINITIONS` boundary must remain unchanged.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/brim.rs`: focused brim metadata tests.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public lookup/count coverage.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `brim_flow_ratio` (`coFloat`, default `1`, lines 1637-1646)
- `brim_use_efc_outline` (`coBool`, default `false`, lines 1648-1656)
- `combine_brims` (`coBool`, default `false`, lines 1658-1663)

## Functional requirements

1. Add the included options to sorted definition shards using existing `OptionValueKind::Float` and `OptionValueKind::Bool`.
2. Preserve public API: `OptionDefinition`, `option_definitions()`, and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve sorted/no-duplicate test coverage across the merged table.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, brim flow calculation, EFC outline alignment, brim combining, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter existing `brim_width`, `brim_type`, `brim_object_gap`, or later brim-ear options from `PrintConfig.cpp:1665+`.
8. Do not add preset-list behavior, object override behavior, new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Update roadmap and milestone docs so E2E parity moves to M45, or verify those docs if the rename already exists in the current worktree.
10. Modified Rust files must remain under 400 LOC.

## Deferred behavior

- Upstream label/category/tooltip/min/max/mode metadata from `PrintConfig.cpp:1637-1663` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Brim flow calculation, EFC outline alignment, and brim combining are deferred to later source-cited brim/skirt milestones.
- Extrusion behavior and G-code behavior are deferred.
- Preset option-list behavior in `Preset.cpp` and object override handling are deferred.
- Existing `brim_width`, `brim_type`, `brim_object_gap`, and following brim-ear options are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all three new keys, kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The merged definition stream remains sorted and binary-search compatible.
- Plan/spec explicitly account for deferred upstream UI metadata, brim behavior, EFC behavior, extrusion/G-code behavior, preset-list behavior, object override behavior, and following brim-ear options.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
