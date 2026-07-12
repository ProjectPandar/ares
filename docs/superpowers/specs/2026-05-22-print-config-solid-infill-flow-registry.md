# M35 Spec: PrintConfig solid infill flow ratio option registry slice

## Goal
Port the next adjacent FFF-specific `libslic3r::PrintConfigDef::init_fff_params` option-definition slice into `ares-core` option registry metadata, covering top and bottom solid infill flow ratio options without changing flow planning or extrusion behavior.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1193-1194`: typed top/bottom solid infill flow ratio fields.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1286-1305`: `PrintConfigDef::init_fff_params()` top and bottom solid infill flow ratio option definitions.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table.rs`: sorted source-cited definition table.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata.rs`: registry metadata tests.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public lookup/count coverage.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `top_solid_infill_flow_ratio` (`coFloat`, default `1`, lines 1286-1295)
- `bottom_solid_infill_flow_ratio` (`coFloat`, default `1`, lines 1297-1305)

## Functional requirements

1. Add the included options to the sorted definition table using existing `OptionValueKind::Float`.
2. Preserve public API: `OptionDefinition`, `option_definitions()`, and `option_definition(key)` remain unchanged.
3. Preserve binary-search lookup and sorted/no-duplicate test coverage.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, flow planning, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add or alter `set_other_flow_ratios` or following per-role flow-ratio options from `PrintConfig.cpp:1307+`.
7. Do not add legacy key remapping from `initial_layer_flow_ratio` to `bottom_solid_infill_flow_ratio` from `PrintConfig.cpp:8003-8005`; Ares has no legacy fallback and this milestone is registry-only.
8. Do not add a new pipeline stage, crate, dependency, filesystem behavior, network behavior, or UI behavior.
9. Update roadmap and milestone docs so E2E parity moves to M36, or verify those docs if the rename already exists in the current worktree.
10. Modified Rust files must remain under 400 LOC.

## Deferred behavior

- Upstream label/category/tooltip/min/max/mode metadata from `PrintConfig.cpp:1286-1305` is explicitly deferred because the current `OptionDefinition` boundary stores only key, value kind, default value, and source citation.
- The legacy compatibility remap from `initial_layer_flow_ratio` to `bottom_solid_infill_flow_ratio` in `PrintConfig.cpp:8003-8005` is explicitly not implemented because Ares has no legacy fallback and M35 is registry-only.
- `set_other_flow_ratios`, `first_layer_flow_ratio`, and following per-role flow-ratio options from `PrintConfig.cpp:1307+` are deferred.
- Actual top/bottom solid infill flow planning, extrusion behavior, and G-code behavior are deferred to later source-cited milestones.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove both new keys, kinds, default values, and source line references.
- Plan/spec explicitly account for deferred upstream UI metadata and the rejected `initial_layer_flow_ratio` legacy remap.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
