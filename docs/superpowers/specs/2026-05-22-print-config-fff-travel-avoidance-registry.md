# M28 Spec: PrintConfig FFF travel avoidance option registry slice

## Goal
Port the first FFF-specific `libslic3r::PrintConfigDef::init_fff_params` option-definition slice into `ares-core` option registry metadata, covering travel wall-avoidance parameters without adding travel-planning behavior.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1479-1480`: typed config fields for `reduce_crossing_wall` and `max_travel_detour_distance`.
- `OrcaSlicer/src/libslic3r/PrintConfigConstants.hpp:7`: `INITIAL_REDUCE_CROSSING_WALL false` default source.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:897-921`: first `PrintConfigDef::init_fff_params()` option definitions.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions.rs`: sorted source-cited definition table.
- `crates/ares-core/src/options/registry/tests.rs`: registry table tests.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public lookup/count coverage.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `reduce_crossing_wall` (`coBool`, default `false`, `PrintConfig.cpp:904-909`, default macro `PrintConfigConstants.hpp:7`)
- `max_travel_detour_distance` (`coFloatOrPercent`, default `0`, `PrintConfig.cpp:911-921`)

## Functional requirements

1. Add the included options to the sorted definition table.
2. Preserve public API: `OptionValueKind`, `OptionDefinition`, `option_definitions()`, and `option_definition(key)` remain unchanged.
3. Preserve binary-search lookup and sorted/no-duplicate test coverage.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors or behavior for these options in this milestone.
6. Do not add a new pipeline stage, crate, dependency, travel planning behavior, filesystem behavior, network behavior, or G-code behavior.
7. Update M28/M29 roadmap and milestone docs so E2E parity moves to M29.
8. Modified Rust files must remain under 400 LOC.

## Deferred behavior

- Actual travel detour / avoid-crossing-wall path planning is deferred to a later source-cited travel-planning milestone.
- FFF bed-temperature options starting at `PrintConfig.cpp:923` are deferred.
- Full FFF option registry parity remains incremental.

## Acceptance checks

- Registry tests prove new keys, kinds, default values, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
