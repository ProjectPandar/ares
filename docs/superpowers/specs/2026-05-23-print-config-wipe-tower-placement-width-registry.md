# M136 Spec: PrintConfig wipe-tower placement and width registry slice

## Goal
Port the adjacent wipe-tower placement and prime-tower width option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1577`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6694-6700`: `wipe_tower_x` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1578`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6702-6708`: `wipe_tower_y` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1579`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6710-6716`: `prime_tower_width` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/sidetext/min/mode metadata beyond the current registry metadata boundary.
- Wipe-tower X/Y placement behavior, partplate placement logic, prime-tower width use, prime tower generation, slicing behavior, geometry behavior, extrusion behavior, and G-code behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6718+`: `wipe_tower_rotation_angle`, `prime_tower_brim_width`, `wipe_tower_cone_angle`, and following options.
- Filesystem behavior, network behavior, UI behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/late_tail_final.rs`: add `prime_tower_width` in lexicographic order after `prime_tower_enable_framework` and before `prime_volume`.
- `crates/ares-core/src/options/registry/definitions/table/tail_terminal_suffix.rs`: add `wipe_tower_x` and `wipe_tower_y` in lexicographic order after `wipe_tower_type` and before `wrapping_detection_gcode`.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add covered expected keys in sorted positions.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/wipe_tower_placement_width.rs`: add metadata assertions for all three definitions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_wipe_tower_placement_width.rs`: add public lookup assertions for all three definitions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/tail_values.rs`: add known-count fixture values for all three covered keys.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`: update expected known and total counts by three.
- `docs/roadmap.md` and `docs/milestones/m136-print-config-wipe-tower-placement-width-registry.md`: milestone sequencing docs.

## Included option definitions

- `wipe_tower_x` (`coFloats`, default `15`, field at `PrintConfig.hpp:1577`, definition lines 6694-6700, Ares kind `Floats`)
- `wipe_tower_y` (`coFloats`, default `220`, field at `PrintConfig.hpp:1578`, definition lines 6702-6708, Ares kind `Floats`)
- `prime_tower_width` (`coFloat`, default `60`, field at `PrintConfig.hpp:1579`, definition lines 6710-6716, Ares kind `Float`)

## Functional requirements

1. Add the three missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, wipe-tower placement behavior, prime-tower width behavior, prime tower behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `wipe_tower_rotation_angle`, `prime_tower_brim_width`, `wipe_tower_cone_angle`, or following options from `PrintConfig.cpp:6718+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove all three covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all three covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/geometry/extrusion/G-code behavior, and following `PrintConfig.cpp:6718+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
