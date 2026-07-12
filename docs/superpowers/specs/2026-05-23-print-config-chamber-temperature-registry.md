# M127 Spec: PrintConfig chamber temperature registry slice

## Goal
Port the adjacent chamber-temperature control option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1636`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6448-6455`: `activate_chamber_temp_control` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1637`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6457-6476`: `chamber_temperature` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/full labels/sidetext/minimums/maximums/modes beyond the current registry metadata boundary.
- Chamber temperature control behavior, M191/M141 emission, chamber-temperature start-G-code variable handling, firmware capability behavior, and heat-soak behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6478+`: `nozzle_temperature`, `nozzle_temperature_range_low`, `nozzle_temperature_range_high`, and following nozzle-temperature options.
- Slicing, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/early.rs`: add `activate_chamber_temp_control` after `activate_air_filtration_on_completion` and before `adaptive_bed_mesh_margin`; add `chamber_temperature` after `calib_flowrate_topinfill_special_order` and before `close_additional_fan_first_x_layers`, preserving sorted order.
- `crates/ares-core/src/options/registry/tests/keys/first.rs`: add the two covered expected keys in the same sorted positions.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/chamber_temperature.rs`: add metadata assertions for both definitions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_chamber_temperature.rs`: add public lookup assertions for both definitions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs` and `crates/ares-core/src/options/tests/registry_helpers/known_count/values/tail_values.rs`: add known-count fixture values and expected counts without touching near-limit `values.rs`.
- `docs/roadmap.md` and `docs/milestones/m127-print-config-chamber-temperature-registry.md`: milestone sequencing docs.

## Included option definitions

- `activate_chamber_temp_control` (`coBools`, default `[false]`, field at `PrintConfig.hpp:1636`, definition lines 6448-6455, Ares kind `Bools`)
- `chamber_temperature` (`coInts`, default `[0]`, field at `PrintConfig.hpp:1637`, definition lines 6457-6476, Ares kind `Ints`)

## Functional requirements

1. Add the two missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, chamber temperature control behavior, M191/M141 emission behavior, start-G-code variable handling, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `nozzle_temperature`, `nozzle_temperature_range_low`, `nozzle_temperature_range_high`, or following options from `PrintConfig.cpp:6478+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove both covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for both covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:6478+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
