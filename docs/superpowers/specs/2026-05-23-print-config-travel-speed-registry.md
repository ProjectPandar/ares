# M133 Spec: PrintConfig travel-speed registry slice

## Goal
Port the adjacent travel-speed option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1396`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6610-6616`: `travel_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1397`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6618-6626`: `travel_speed_z` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/sidetext/min/mode metadata beyond the current registry metadata boundary.
- Travel speed and Z travel speed planning behavior, movement planning, slicing behavior, geometry behavior, extrusion behavior, and G-code behavior.
- Typed accessors or behavior changes for `travel_speed_z`; existing `travel_speed` runtime behavior remains unchanged.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6628+`: `wipe`, `wipe_distance`, `enable_prime_tower`, and following options.
- Filesystem behavior, network behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/tail_terminal_suffix.rs`: update existing `travel_speed` source citation to include `PrintConfig.hpp:1396`, and add `travel_speed_z` in lexicographic order after `travel_speed`.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add `travel_speed_z` after `travel_speed`.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/travel_speed.rs`: add metadata assertions for both definitions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_travel_speed.rs`: add public lookup assertions for both definitions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/tail_values.rs`: add known-count fixture value for `travel_speed_z`; existing `travel_speed` remains typed runtime behavior and is not added to the metadata-only fixture here unless already present elsewhere.
- `docs/roadmap.md` and `docs/milestones/m133-print-config-travel-speed-registry.md`: milestone sequencing docs.

## Included option definitions

- `travel_speed` (`coFloat`, default `120`, field at `PrintConfig.hpp:1396`, definition lines 6610-6616, Ares kind `Float`) — already present in registry, source citation is completed in this milestone.
- `travel_speed_z` (`coFloat`, default `0`, field at `PrintConfig.hpp:1397`, definition lines 6618-6626, Ares kind `Float`)

## Functional requirements

1. Preserve `travel_speed` kind/default while completing its source citation with `PrintConfig.hpp:1396`.
2. Add missing `travel_speed_z` using existing `Float` value kind only.
3. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
4. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
5. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
6. Do not add typed parsing/accessors, speed-planning behavior, Z-travel behavior, slicing behavior, geometry behavior, extrusion behavior, or G-code behavior for these options in this milestone.
7. Do not add `wipe`, `wipe_distance`, `enable_prime_tower`, or following options from `PrintConfig.cpp:6628+`.
8. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
9. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove both covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts `travel_speed_z` while preserving unknown options.
- Public lookup coverage exists for both covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/geometry/extrusion/G-code behavior, and following `PrintConfig.cpp:6628+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
