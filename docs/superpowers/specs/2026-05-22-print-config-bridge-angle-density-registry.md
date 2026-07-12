# M34 Spec: PrintConfig bridge angle and density option registry slice

## Goal
Port the next adjacent FFF-specific `libslic3r::PrintConfigDef::init_fff_params` option-definition slice into `ares-core` option registry metadata, covering bridge angle and bridge density options without changing bridge planning or extrusion behavior.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:991`: typed `ConfigOptionPercent` field for `internal_bridge_density`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1081-1082`: typed bridge angle fields.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1189`: typed `ConfigOptionPercent` field for `bridge_density`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1213-1264`: `PrintConfigDef::init_fff_params()` bridge angle and bridge density option definitions.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table.rs`: sorted source-cited definition table.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata.rs`: registry metadata tests.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public lookup/count coverage.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `bridge_angle` (`coFloat`, default `0`, lines 1213-1223)
- `internal_bridge_angle` (`coFloat`, default `0`, lines 1226-1235)
- `bridge_density` (`coPercent`, default `100`, lines 1237-1250)
- `internal_bridge_density` (`coPercent`, default `100`, lines 1252-1264)

## Functional requirements

1. Add the included options to the sorted definition table using existing `OptionValueKind` variants.
2. Preserve public API: `OptionDefinition`, `option_definitions()`, and `option_definition(key)` remain unchanged.
3. Preserve binary-search lookup and sorted/no-duplicate test coverage.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, bridge angle planning, bridge density planning, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not alter already registered `bridge_flow` or `internal_bridge_flow`; those existing definitions are outside this milestone's implementation scope.
7. Do not add a new pipeline stage, crate, dependency, filesystem behavior, network behavior, or UI behavior.
8. Update roadmap and milestone docs so E2E parity moves to M35.
9. Modified Rust files must remain under 400 LOC.

## Deferred behavior

- `bridge_flow` and `internal_bridge_flow` behavior remain as previously implemented/registered and are not changed here.
- `top_solid_infill_flow_ratio`, `bottom_solid_infill_flow_ratio`, and following flow options from `PrintConfig.cpp:1286+` are deferred.
- Actual bridge angle selection, bridge density spacing, extrusion behavior, and bridge G-code behavior are deferred to later source-cited milestones.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all four new keys, kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
