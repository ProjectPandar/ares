# M135 Spec: PrintConfig flush and prime-volume registry slice

## Goal
Port the adjacent flush-volume and prime-volume option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1591`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6659-6667`: `flush_volumes_vector` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1590`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6669-6677`: `flush_volumes_matrix` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1608`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6679-6683`: `flush_multiplier` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1607`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6686-6692`: `prime_volume` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/sidetext/min/mode metadata beyond the current registry metadata boundary.
- Flush-volume vector/matrix interpretation, flush multiplier application, prime-volume use, prime tower generation, slicing behavior, geometry behavior, extrusion behavior, and G-code behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6694+`: `wipe_tower_x`, `wipe_tower_y`, `prime_tower_width`, and following options.
- Filesystem behavior, network behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: add `flush_multiplier`, `flush_volumes_matrix`, and `flush_volumes_vector` in lexicographic order after `first_x_layer_fan_speed` and before `full_fan_speed_layer`.
- `crates/ares-core/src/options/registry/definitions/table/late_tail_final.rs`: add `prime_volume` in lexicographic order after `prime_tower_enable_framework` and before `print_compatible_printers`.
- `crates/ares-core/src/options/registry/tests/keys/first.rs` and `crates/ares-core/src/options/registry/tests/keys/second.rs`: add covered expected keys in sorted positions.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/flush_prime_volume.rs`: add metadata assertions for all four definitions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_flush_prime_volume.rs`: add public lookup assertions for all four definitions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/tail_values.rs`: add known-count fixture values for all four covered keys.
- `docs/roadmap.md` and `docs/milestones/m135-print-config-flush-prime-volume-registry.md`: milestone sequencing docs.

## Included option definitions

- `flush_volumes_vector` (`coFloats`, default `140,140,140,140,140,140,140,140`, field at `PrintConfig.hpp:1591`, definition lines 6659-6667, Ares kind `Floats`)
- `flush_volumes_matrix` (`coFloats`, default `0,280,280,280,280,0,280,280,280,280,0,280,280,280,280,0`, field at `PrintConfig.hpp:1590`, definition lines 6669-6677, Ares kind `Floats`)
- `flush_multiplier` (`coFloats`, default `0.3`, field at `PrintConfig.hpp:1608`, definition lines 6679-6683, Ares kind `Floats`)
- `prime_volume` (`coFloat`, default `45`, field at `PrintConfig.hpp:1607`, definition lines 6686-6692, Ares kind `Float`)

## Functional requirements

1. Add the four missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, flush-volume behavior, prime-volume behavior, prime tower behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `wipe_tower_x`, `wipe_tower_y`, `prime_tower_width`, or following options from `PrintConfig.cpp:6694+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove all four covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all four covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/geometry/extrusion/G-code behavior, and following `PrintConfig.cpp:6694+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
