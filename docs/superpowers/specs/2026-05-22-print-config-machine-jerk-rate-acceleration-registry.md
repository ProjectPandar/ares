# M94 Spec: PrintConfig machine jerk, min-rate, and acceleration PRT registry slice

## Goal
Port the adjacent machine XYZE jerk, junction deviation, minimum feedrate, and M204 P/R/T acceleration option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4377-4390`: `AxisDefault` names and default jerk values for axes `x`, `y`, `z`, and `e`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1265-1268`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4429-4446`: `machine_max_jerk_x`, `machine_max_jerk_y`, `machine_max_jerk_z`, and `machine_max_jerk_e` option definitions.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1270`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4449-4458`: `machine_max_junction_deviation` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1274`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4460-4468`: `machine_min_extruding_rate` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1272`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4470-4478`: `machine_min_travel_rate` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1260`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4480-4491`: `machine_max_acceleration_extruding` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1261`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4494-4503`: `machine_max_acceleration_retracting` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1262`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4505-4514`: `machine_max_acceleration_travel` option definition.

Related upstream behavior explicitly deferred:

- UI full-label/tooltip/category/sidetext/min/max/mode/readonly metadata beyond the current registry boundary.
- Machine-limit emission behavior and M204/M205 G-code output.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4516+`: resonance avoidance, input shaping, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/late.rs`: add sorted definitions for the ten machine limit keys.
- `crates/ares-core/src/options/registry/tests/keys.rs`: preserve registry key coverage and sorted/no-duplicate tests while delegating the large expected-key list to submodules.
- `crates/ares-core/src/options/registry/tests/keys/first.rs`: first sorted chunk of expected registry keys.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: second sorted chunk of expected registry keys, including the new M94 keys.
- `crates/ares-core/src/options/registry/tests/metadata.rs`: add `mod machine_limit_rates;`.
- `crates/ares-core/src/options/registry/tests/metadata/machine_limit_rates.rs`: source metadata assertions for all ten options.
- `crates/ares-core/src/options/tests.rs`: add `mod registry_lookup_machine_limit_rates;`.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_machine_limit_rates.rs`: public lookup coverage for all ten options.
- `docs/roadmap.md` and `docs/milestones/m94-print-config-machine-jerk-rate-acceleration-registry.md`: milestone sequencing docs.

## Included option definitions

- `machine_max_jerk_e` (`coFloats`, default `2.5,2.5`, field at `PrintConfig.hpp:1268`, generated from lines 4389 and 4429-4446, Ares kind `Floats`)
- `machine_max_jerk_x` (`coFloats`, default `10,10`, field at `PrintConfig.hpp:1265`, generated from lines 4386 and 4429-4446, Ares kind `Floats`)
- `machine_max_jerk_y` (`coFloats`, default `10,10`, field at `PrintConfig.hpp:1266`, generated from lines 4387 and 4429-4446, Ares kind `Floats`)
- `machine_max_jerk_z` (`coFloats`, default `0.2,0.4`, field at `PrintConfig.hpp:1267`, generated from lines 4388 and 4429-4446, Ares kind `Floats`)
- `machine_max_junction_deviation` (`coFloats`, default `0.01`, field at `PrintConfig.hpp:1270`, definition lines 4449-4458, Ares kind `Floats`)
- `machine_min_extruding_rate` (`coFloats`, default `0,0`, field at `PrintConfig.hpp:1274`, definition lines 4460-4468, Ares kind `Floats`)
- `machine_min_travel_rate` (`coFloats`, default `0,0`, field at `PrintConfig.hpp:1272`, definition lines 4470-4478, Ares kind `Floats`)
- `machine_max_acceleration_extruding` (`coFloats`, default `1500,1250`, field at `PrintConfig.hpp:1260`, definition lines 4480-4491, Ares kind `Floats`)
- `machine_max_acceleration_retracting` (`coFloats`, default `1500,1250`, field at `PrintConfig.hpp:1261`, definition lines 4494-4503, Ares kind `Floats`)
- `machine_max_acceleration_travel` (`coFloats`, default `0,0`, field at `PrintConfig.hpp:1262`, definition lines 4505-4514, Ares kind `Floats`)

## Functional requirements

1. Add the included missing options to sorted definition shards using existing value kinds only.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, machine-limit emission behavior, M204/M205 generation behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add or alter resonance avoidance, input shaping, or following options from `PrintConfig.cpp:4516+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Keep modified Rust files under 400 LOC by splitting the large registry key expectation list into focused modules before adding M94 keys.

## Acceptance checks

- Registry tests prove all ten new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all ten new keys.
- Plan/spec explicitly account for deferred UI metadata, runtime behavior, slicing/extrusion/G-code behavior, and following resonance/input-shaping scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
