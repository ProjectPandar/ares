# M132 Spec: PrintConfig surface-density registry slice

## Goal
Port the adjacent top and bottom surface-density option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1088`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6586-6596`: `top_surface_density` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1089`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6598-6607`: `bottom_surface_density` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/category/sidetext/min/max metadata beyond the current registry metadata boundary.
- Top/bottom surface density runtime interpretation, surface pattern application, extrusion planning, slicing behavior, geometry behavior, and G-code behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6610+`: `travel_speed`, `travel_speed_z`, and following options.
- Filesystem behavior, network behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/early.rs`: add `bottom_surface_density` in lexicographic order before the existing `bottom_surface_pattern` definition.
- `crates/ares-core/src/options/registry/definitions/table/tail_terminal_suffix.rs`: add `top_surface_density` in lexicographic order between `top_surface_acceleration` and `top_surface_jerk`.
- `crates/ares-core/src/options/registry/tests/keys/first.rs` and `crates/ares-core/src/options/registry/tests/keys/second.rs`: add the covered expected keys in sorted positions.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/surface_density.rs`: add metadata assertions for both definitions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_surface_density.rs`: add public lookup assertions for both definitions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count/values/tail_values.rs`: add known-count fixture values for both covered keys.
- `docs/roadmap.md` and `docs/milestones/m132-print-config-surface-density-registry.md`: milestone sequencing docs.

## Included option definitions

- `top_surface_density` (`coPercent`, default `100`, field at `PrintConfig.hpp:1088`, definition lines 6586-6596, Ares kind `Percent`)
- `bottom_surface_density` (`coPercent`, default `100`, field at `PrintConfig.hpp:1089`, definition lines 6598-6607, Ares kind `Percent`)

## Functional requirements

1. Add the two missing options using the existing `Percent` value kind only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, surface-density runtime interpretation, surface pattern application, slicing behavior, geometry behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `travel_speed_z` or following options from `PrintConfig.cpp:6618+`; `travel_speed` is already present and remains unchanged.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove both covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for both covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/geometry/extrusion/G-code behavior, and following `PrintConfig.cpp:6610+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
