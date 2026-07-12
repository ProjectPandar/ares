# M93 Spec: PrintConfig machine speed and acceleration limit registry slice

## Goal
Port the adjacent machine XYZE maximum speed and maximum acceleration option-definition slice generated inside `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4377-4390`: `AxisDefault` names and default values for axes `x`, `y`, `z`, and `e`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1254-1257`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4391-4410`: `machine_max_speed_x`, `machine_max_speed_y`, `machine_max_speed_z`, and `machine_max_speed_e` option definitions.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1249-1252`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4411-4428`: `machine_max_acceleration_x`, `machine_max_acceleration_y`, `machine_max_acceleration_z`, and `machine_max_acceleration_e` option definitions.

Related upstream behavior explicitly deferred:

- UI full-label/tooltip/category/sidetext/min/mode/readonly metadata beyond the current registry boundary.
- Machine-limit emission behavior and M201/M203 G-code output.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4429+`: machine jerk, junction-deviation, min-rate, acceleration P/R/T, resonance, input-shaping, and following options.
- Slicing, extrusion, G-code behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/late.rs`: add sorted definitions for the eight machine limit keys and keep the shard below 400 LOC.
- `crates/ares-core/src/options/registry/definitions/table/late_tail.rs`: split existing `max_bridge_length` and following late definitions out of `late.rs` without changing unrelated moved metadata.
- `crates/ares-core/src/options/registry/definitions/table.rs`: merge the new `late_tail` shard between `late` and `tail` to preserve sorted lookup order.
- `crates/ares-core/src/options/registry/tests/keys.rs`: registry key coverage and sorted/no-duplicate tests.
- `crates/ares-core/src/options/registry/tests/metadata.rs`: add `mod machine_limits;`.
- `crates/ares-core/src/options/registry/tests/metadata/machine_limits.rs`: source metadata assertions for all eight options.
- `crates/ares-core/src/options/tests.rs`: add `mod registry_lookup_machine_limits;`.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public count/unknown preservation.
- `crates/ares-core/src/options/tests/registry_lookup_machine_limits.rs`: public lookup coverage for all eight options.
- `docs/roadmap.md` and `docs/milestones/m93-print-config-machine-speed-acceleration-registry.md`: milestone sequencing docs.

## Included option definitions

- `machine_max_acceleration_e` (`coFloats`, default `5000,5000`, field at `PrintConfig.hpp:1252`, generated from lines 4389 and 4411-4428, Ares kind `Floats`)
- `machine_max_acceleration_x` (`coFloats`, default `1000,1000`, field at `PrintConfig.hpp:1249`, generated from lines 4386 and 4411-4428, Ares kind `Floats`)
- `machine_max_acceleration_y` (`coFloats`, default `1000,1000`, field at `PrintConfig.hpp:1250`, generated from lines 4387 and 4411-4428, Ares kind `Floats`)
- `machine_max_acceleration_z` (`coFloats`, default `500,200`, field at `PrintConfig.hpp:1251`, generated from lines 4388 and 4411-4428, Ares kind `Floats`)
- `machine_max_speed_e` (`coFloats`, default `120,120`, field at `PrintConfig.hpp:1257`, generated from lines 4389 and 4391-4410, Ares kind `Floats`)
- `machine_max_speed_x` (`coFloats`, default `500,200`, field at `PrintConfig.hpp:1254`, generated from lines 4386 and 4391-4410, Ares kind `Floats`)
- `machine_max_speed_y` (`coFloats`, default `500,200`, field at `PrintConfig.hpp:1255`, generated from lines 4387 and 4391-4410, Ares kind `Floats`)
- `machine_max_speed_z` (`coFloats`, default `12,12`, field at `PrintConfig.hpp:1256`, generated from lines 4388 and 4391-4410, Ares kind `Floats`)

## Functional requirements

1. Add the included missing options to sorted definition shards using existing value kinds only.
2. Preserve public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing API.
5. Do not add typed parsing/accessors, machine-limit emission behavior, M201/M203 generation behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add or alter machine jerk, junction-deviation, min-rate, acceleration P/R/T, resonance, input-shaping, or following options from `PrintConfig.cpp:4429+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Keep modified Rust files under 400 LOC by splitting `late` into a focused `late_tail` shard when M93 pushes the existing file over the limit; create focused tests instead of growing unrelated near-limit files.

## Acceptance checks

- Registry tests prove all eight new keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- Public lookup coverage exists for all eight new keys.
- Plan/spec explicitly account for deferred UI metadata, runtime behavior, slicing/extrusion/G-code behavior, and following machine-jerk/junction/resonance/input-shaping scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
