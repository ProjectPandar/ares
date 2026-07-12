# M29 Spec: PrintConfig bed temperature other-layers option registry slice

## Goal
Port the next FFF-specific `libslic3r::PrintConfigDef::init_fff_params` option-definition slice into `ares-core` option registry metadata, covering build-plate “other layers” bed temperature options without adding temperature G-code behavior.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1490-1495`: typed `ConfigOptionInts` fields for other-layer plate temperatures.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:923-982`: `PrintConfigDef::init_fff_params()` other-layer bed temperature option definitions.

Rust destination boundary:

- `crates/ares-core/src/options/registry.rs`: `OptionValueKind` vocabulary.
- `crates/ares-core/src/options/registry/definitions.rs`: sorted source-cited definition table.
- `crates/ares-core/src/options/registry/tests.rs`: registry table tests.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public lookup/count coverage.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `supertack_plate_temp` (`coInts`, default `35`, lines 924-932)
- `cool_plate_temp` (`coInts`, default `35`, lines 934-942)
- `textured_cool_plate_temp` (`coInts`, default `40`, lines 944-952)
- `eng_plate_temp` (`coInts`, default `45`, lines 954-962)
- `hot_plate_temp` (`coInts`, default `45`, lines 964-972)
- `textured_plate_temp` (`coInts`, default `45`, lines 974-982)

## Functional requirements

1. Extend `OptionValueKind` only as needed for `coInts` (`Ints`).
2. Add the included options to the sorted definition table.
3. Preserve public API: `OptionDefinition`, `option_definitions()`, and `option_definition(key)` remain unchanged.
4. Preserve binary-search lookup and sorted/no-duplicate test coverage.
5. Preserve `SliceOptions` unknown-value storage and current public slicing API.
6. Do not add typed parsing/accessors or behavior for these options in this milestone.
7. Do not add a new pipeline stage, crate, dependency, temperature G-code behavior, filesystem behavior, network behavior, or UI behavior.
8. Update M29/M30 roadmap and milestone docs so E2E parity moves to M30.
9. Modified Rust files must remain under 400 LOC.

## Deferred behavior

- Initial-layer bed temperature options from `PrintConfig.cpp:984-1041` are deferred.
- Actual bed-temperature G-code emission and material/plate selection behavior are deferred to later source-cited milestones.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove new keys, `Ints` kind, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
