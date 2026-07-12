# M36 Spec: PrintConfig other-flow gate option registry slice

## Goal
Port the next adjacent FFF-specific `libslic3r::PrintConfigDef::init_fff_params` option-definition slice into `ares-core` option registry metadata, covering the other-flow-ratio enable gate and first-layer flow ratio without changing flow planning or extrusion behavior.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:978`: typed `set_other_flow_ratios` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1214-1215`: other flow-ratio section and typed `first_layer_flow_ratio` field.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1307-1323`: `PrintConfigDef::init_fff_params()` option definitions for `set_other_flow_ratios` and `first_layer_flow_ratio`.

Related upstream behavior explicitly deferred:

- `OrcaSlicer/src/libslic3r/GCode.cpp:6415` and `GCode.cpp:6436`: runtime flow-ratio application in G-code generation.
- `OrcaSlicer/src/libslic3r/Preset.cpp:1186-1187`: preset option-list behavior.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:1397`: object option override handling.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table.rs`: sorted source-cited definition table.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata.rs`: registry metadata tests.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public lookup/count coverage.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `set_other_flow_ratios` (`coBool`, default `false`, lines 1307-1312)
- `first_layer_flow_ratio` (`coFloat`, default `1`, lines 1314-1323)

## Functional requirements

1. Add the included options to the sorted definition table using existing `OptionValueKind::Bool` and `OptionValueKind::Float`.
2. Preserve public API: `OptionDefinition`, `option_definitions()`, and `option_definition(key)` remain unchanged.
3. Preserve binary-search lookup and sorted/no-duplicate test coverage.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, flow planning, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add or alter following per-role flow-ratio options from `PrintConfig.cpp:1324+`.
7. Do not add preset-list behavior, object override behavior, new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Update roadmap and milestone docs so E2E parity moves to M37, or verify those docs if the rename already exists in the current worktree.
9. Modified Rust files must remain under 400 LOC. This milestone may bring `table.rs` close to the limit but must not exceed it.

## Deferred behavior

- Upstream label/category/tooltip/min/max/mode metadata from `PrintConfig.cpp:1307-1323` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- Runtime flow scaling in `GCode.cpp:6415` and `GCode.cpp:6436` is deferred to a later source-cited G-code/extrusion milestone.
- Preset option-list behavior in `Preset.cpp:1186-1187` and object override handling in `PrintObject.cpp:1397` are deferred.
- `outer_wall_flow_ratio`, `inner_wall_flow_ratio`, and following per-role flow-ratio options from `PrintConfig.cpp:1324+` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove both new keys, kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Plan/spec explicitly account for deferred upstream UI metadata, runtime flow behavior, preset-list behavior, and object override behavior.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
