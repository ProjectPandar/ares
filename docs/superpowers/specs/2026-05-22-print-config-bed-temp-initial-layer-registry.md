# M30 Spec: PrintConfig bed temperature initial-layer option registry slice

## Goal
Port the next adjacent FFF-specific `libslic3r::PrintConfigDef::init_fff_params` option-definition slice into `ares-core` option registry metadata, covering build-plate first-layer bed temperature options without adding temperature G-code behavior.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1496-1501`: typed `ConfigOptionInts` fields for first-layer plate temperatures.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:984-1041`: `PrintConfigDef::init_fff_params()` first-layer bed temperature option definitions.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions.rs`: registry definitions module facade.
- `crates/ares-core/src/options/registry/definitions/table.rs`: sorted source-cited definition table.
- `crates/ares-core/src/options/registry/tests.rs`: registry table tests.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public lookup/count coverage.
- `docs/roadmap.md` and `docs/milestones/*.md`: milestone sequencing docs.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `supertack_plate_temp_initial_layer` (`coInts`, default `35`, lines 984-992)
- `cool_plate_temp_initial_layer` (`coInts`, default `35`, lines 994-1002)
- `textured_cool_plate_temp_initial_layer` (`coInts`, default `40`, lines 1004-1012)
- `eng_plate_temp_initial_layer` (`coInts`, default `45`, lines 1014-1022)
- `hot_plate_temp_initial_layer` (`coInts`, default `45`, lines 1024-1031)
- `textured_plate_temp_initial_layer` (`coInts`, default `45`, lines 1033-1041)

## Functional requirements

1. Add the included options to the sorted definition table using existing `OptionValueKind::Ints` for upstream `coInts`.
2. Preserve public API: `OptionDefinition`, `option_definitions()`, and `option_definition(key)` remain unchanged.
3. Preserve binary-search lookup and sorted/no-duplicate test coverage.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors or behavior for these options in this milestone.
6. Do not add a new pipeline stage, crate, dependency, temperature G-code behavior, filesystem behavior, network behavior, or UI behavior.
7. Update roadmap and milestone docs so E2E parity moves to M31.
8. Modified Rust files must remain under 400 LOC; because `definitions.rs` is near the limit, this milestone must first move the table into `definitions/table.rs` and store entries through a compact local macro before adding the six definitions.

## Deferred behavior

- `curr_bed_type` and following fan/overhang options from `PrintConfig.cpp:1043+` are deferred.
- Actual bed-temperature G-code emission and material/plate selection behavior are deferred to later source-cited milestones.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove all six new keys, `Ints` kind, default values, and source line references, and the LOC check proves the split table files remain below 400 LOC after `cargo fmt --check`.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
