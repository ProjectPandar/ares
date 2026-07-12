# M37 Spec: PrintConfig wall flow ratio option registry slice

## Goal
Port the next adjacent FFF-specific `libslic3r::PrintConfigDef::init_fff_params` option-definition slice into `ares-core` option registry metadata, covering outer and inner wall flow ratio options while splitting the registry definition table so future source-cited PrintConfig slices can continue under the 400 LOC rule.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1214-1217`: other flow-ratio section plus typed `outer_wall_flow_ratio` and `inner_wall_flow_ratio` fields.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1324-1343`: `PrintConfigDef::init_fff_params()` option definitions for `outer_wall_flow_ratio` and `inner_wall_flow_ratio`.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/GCode.cpp:6415` and nearby flow-ratio application: runtime flow-ratio behavior.
- `OrcaSlicer/src/libslic3r/Preset.cpp:1186+`: preset option-list behavior for flow ratios.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:1397+`: object option override handling.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions.rs`: keep the public registry definition boundary and expose one sorted `OPTION_DEFINITIONS` slice.
- `crates/ares-core/src/options/registry/definitions/table.rs`: become the small merge point for definition shards while preserving `option_definitions()` and `option_definition()` behavior.
- `crates/ares-core/src/options/registry/definitions/table/early.rs`: first sorted registry shard.
- `crates/ares-core/src/options/registry/definitions/table/late.rs`: second sorted registry shard.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata.rs`: registry metadata tests.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public lookup/count coverage.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `outer_wall_flow_ratio` (`coFloat`, default `1`, lines 1324-1333)
- `inner_wall_flow_ratio` (`coFloat`, default `1`, lines 1334-1343)

## Functional requirements

1. Split the existing `OPTION_DEFINITIONS` table into two sorted shards without changing public API shape: `OptionDefinition`, `option_definitions()`, and `option_definition(key)` remain available with the same behavior.
2. Preserve a single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
3. Add the included options to the sorted definition shards using existing `OptionValueKind::Float`.
4. Preserve sorted/no-duplicate test coverage across the merged table.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors, flow planning, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add or alter following per-role flow-ratio options from `PrintConfig.cpp:1344+`.
8. Do not add preset-list behavior, object override behavior, new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Update roadmap and milestone docs so E2E parity moves to M38, or verify those docs if the rename already exists in the current worktree.
10. Modified Rust files must remain under 400 LOC after the split.

## Deferred behavior

- Upstream label/category/tooltip/min/max/mode metadata from `PrintConfig.cpp:1324-1343` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Runtime flow scaling in `GCode.cpp` is deferred to a later source-cited G-code/extrusion milestone.
- Preset option-list behavior in `Preset.cpp` and object override handling in `PrintObject.cpp` are deferred.
- `overhang_flow_ratio`, `sparse_infill_flow_ratio`, `internal_solid_infill_flow_ratio`, `gap_fill_flow_ratio`, support flow ratios, and following per-role flow-ratio options are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove both new keys, kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Split shards merge into the same sorted definition stream used by `option_definition()` binary search.
- Plan/spec explicitly account for deferred upstream UI metadata, runtime flow behavior, preset-list behavior, object override behavior, and following per-role flow ratios.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
