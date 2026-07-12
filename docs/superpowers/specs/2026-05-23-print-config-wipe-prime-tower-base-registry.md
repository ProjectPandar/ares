# M134 Spec: PrintConfig wipe and prime-tower base registry slice

## Goal
Port the adjacent wipe and prime-tower base option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1569`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6628-6633`: `wipe` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1573`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6635-6644`: `wipe_distance` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1574`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6646-6651`: `enable_prime_tower` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1575`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6653-6657`: `prime_tower_enable_framework` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/sidetext/min/mode metadata beyond the current registry metadata boundary.
- Wipe-while-retracting movement behavior, wipe-distance movement planning, prime tower generation, prime tower internal-rib behavior, slicing behavior, geometry behavior, extrusion behavior, and G-code behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6659+`: `flush_volumes_vector`, `flush_volumes_matrix`, `flush_multiplier`, `prime_volume`, wipe-tower placement/dimension options, and following options.
- Filesystem behavior, network behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`: add `enable_prime_tower` in lexicographic order after `enable_pressure_advance` and before `enable_support`.
- `crates/ares-core/src/options/registry/definitions/table/late_tail_final.rs`: add `prime_tower_enable_framework` after `pressure_advance` and before `print_compatible_printers`.
- `crates/ares-core/src/options/registry/definitions/table/tail_terminal_suffix.rs`: add `wipe` and `wipe_distance` in lexicographic order around the existing wipe definitions.
- `crates/ares-core/src/options/registry/definitions/table/tail_z.rs` and `table.rs`: mechanically move existing `z_hop`, `z_hop_types`, `z_offset`, `zaa_dont_alternate_fill_direction`, `zaa_enabled`, `zaa_min_z`, and `zaa_minimize_perimeter_height` definitions out of `tail_terminal_suffix.rs` into a following shard to keep files under 400 LOC; moved definition bodies must remain unchanged.
- `crates/ares-core/src/options/registry/tests/keys/first.rs` and `crates/ares-core/src/options/registry/tests/keys/second.rs`: add covered expected keys in sorted positions.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/wipe_prime_tower_base.rs`: add metadata assertions for all four definitions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_wipe_prime_tower_base.rs`: add public lookup assertions for all four definitions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/tail_values.rs`: add known-count fixture values for all four covered keys.
- `docs/roadmap.md` and `docs/milestones/m134-print-config-wipe-prime-tower-base-registry.md`: milestone sequencing docs.

## Included option definitions

- `wipe` (`coBools`, default `false`, field at `PrintConfig.hpp:1569`, definition lines 6628-6633, Ares kind `Bools`)
- `wipe_distance` (`coFloats`, default `1`, field at `PrintConfig.hpp:1573`, definition lines 6635-6644, Ares kind `Floats`)
- `enable_prime_tower` (`coBool`, default `false`, field at `PrintConfig.hpp:1574`, definition lines 6646-6651, Ares kind `Bool`)
- `prime_tower_enable_framework` (`coBool`, default `false`, field at `PrintConfig.hpp:1575`, definition lines 6653-6657, Ares kind `Bool`)

## Functional requirements

1. Add the four missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, wipe behavior, prime tower behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `flush_volumes_vector`, `flush_volumes_matrix`, `flush_multiplier`, `prime_volume`, or following options from `PrintConfig.cpp:6659+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC; mechanical shard split is allowed only for LOC compliance and must not change moved definitions.

## Acceptance checks

- Registry tests prove all four covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all four covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/geometry/extrusion/G-code behavior, and following `PrintConfig.cpp:6659+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
