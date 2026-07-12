# M38 Spec: PrintConfig overhang and sparse infill flow ratio option registry slice

## Goal
Port the next adjacent FFF-specific `libslic3r::PrintConfigDef::init_fff_params` option-definition slice into `ares-core` option registry metadata, covering overhang and sparse infill flow ratio options without changing flow planning or extrusion behavior.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1214-1219`: other flow-ratio section plus typed `overhang_flow_ratio` and `sparse_infill_flow_ratio` fields.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1344-1363`: `PrintConfigDef::init_fff_params()` option definitions for `overhang_flow_ratio` and `sparse_infill_flow_ratio`.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/GCode.cpp:6415` and nearby flow-ratio application: runtime flow-ratio behavior.
- `OrcaSlicer/src/libslic3r/Preset.cpp:1186+`: preset option-list behavior for flow ratios.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:1397+`: object option override handling.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/late.rs`: sorted source-cited definition shard.
- `crates/ares-core/src/options/registry/definitions/table.rs`: merged `OPTION_DEFINITIONS` boundary must remain unchanged.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata.rs`: existing registry metadata test module, with flow-ratio metadata moved out to stay under 400 LOC.
- `crates/ares-core/src/options/registry/tests/metadata/flow.rs`: focused flow-ratio metadata assertions, including this milestone's new options.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public lookup/count coverage.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `overhang_flow_ratio` (`coFloat`, default `1`, lines 1344-1353)
- `sparse_infill_flow_ratio` (`coFloat`, default `1`, lines 1354-1363)

## Functional requirements

1. Add the included options to the sorted definition shard using existing `OptionValueKind::Float`.
2. Preserve public API: `OptionDefinition`, `option_definitions()`, and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve sorted/no-duplicate test coverage across the merged table.
5. Split flow-ratio metadata assertions into a focused test file before adding new cases so `metadata.rs` remains under 400 LOC.
6. Preserve `SliceOptions` unknown-value storage and current public slicing API.
7. Do not add typed parsing/accessors, flow planning, extrusion behavior, or G-code behavior for these options in this milestone.
8. Do not add or alter following per-role flow-ratio options from `PrintConfig.cpp:1364+`.
9. Do not add preset-list behavior, object override behavior, new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
10. Update roadmap and milestone docs so E2E parity moves to M39, or verify those docs if the rename already exists in the current worktree.
11. Modified Rust files must remain under 400 LOC.

## Deferred behavior

- Upstream label/category/tooltip/min/max/mode metadata from `PrintConfig.cpp:1344-1363` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Runtime flow scaling in `GCode.cpp` is deferred to a later source-cited G-code/extrusion milestone.
- Preset option-list behavior in `Preset.cpp` and object override handling in `PrintObject.cpp` are deferred.
- `internal_solid_infill_flow_ratio`, `gap_fill_flow_ratio`, support flow ratios, and following per-role flow-ratio options are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove both new keys, kinds, default values, and source line references.
- Flow-ratio metadata tests are split into a focused file so `metadata.rs` does not exceed 400 LOC.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- The merged definition stream remains sorted and binary-search compatible.
- Plan/spec explicitly account for deferred upstream UI metadata, runtime flow behavior, preset-list behavior, object override behavior, and following per-role flow ratios.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
