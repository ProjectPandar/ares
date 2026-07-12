# M128 Spec: PrintConfig nozzle temperature registry slice

## Goal
Port the adjacent nozzle-temperature option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1568`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6478-6485`: `nozzle_temperature` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1571`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6487-6493`: `nozzle_temperature_range_low` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1572`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6495-6501`: `nozzle_temperature_range_high` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/full labels/sidetext/minimums/maximums beyond the current registry metadata boundary.
- Nozzle temperature application, initial-layer/other-layer temperature scheduling, start-G-code variable handling, M104/M109 emission behavior, and temperature-range validation behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6503+`: `head_wrap_detect_zone`, `detect_thin_wall`, G-code option definitions, and following options.
- Slicing, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/late_tail.rs`: add `nozzle_temperature` before `nozzle_temperature_initial_layer`, then add `nozzle_temperature_range_high` and `nozzle_temperature_range_low` after `nozzle_temperature_initial_layer` and before `nozzle_type`, preserving sorted order.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add the three covered expected keys in the same sorted positions: `nozzle_temperature`, `nozzle_temperature_initial_layer`, `nozzle_temperature_range_high`, `nozzle_temperature_range_low`, `nozzle_type`.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/nozzle_temperature.rs`: add metadata assertions for all three definitions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_nozzle_temperature.rs`: add public lookup assertions for all three definitions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs` and `crates/ares-core/src/options/tests/registry_helpers/known_count/values/tail_values.rs`: add known-count fixture values and expected counts without touching near-limit `values.rs`.
- `docs/roadmap.md` and `docs/milestones/m128-print-config-nozzle-temperature-registry.md`: milestone sequencing docs.

## Included option definitions

- `nozzle_temperature` (`coInts`, default `[200]`, field at `PrintConfig.hpp:1568`, definition lines 6478-6485, Ares kind `Ints`)
- `nozzle_temperature_range_high` (`coInts`, default `[240]`, field at `PrintConfig.hpp:1572`, definition lines 6495-6501, Ares kind `Ints`)
- `nozzle_temperature_range_low` (`coInts`, default `[190]`, field at `PrintConfig.hpp:1571`, definition lines 6487-6493, Ares kind `Ints`)

## Functional requirements

1. Add the three missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, nozzle temperature behavior, temperature range validation, start-G-code variable handling, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `head_wrap_detect_zone`, `detect_thin_wall`, G-code option definitions, or following options from `PrintConfig.cpp:6503+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove all three covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all three covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:6503+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
