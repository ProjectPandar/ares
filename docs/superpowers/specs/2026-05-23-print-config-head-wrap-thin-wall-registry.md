# M129 Spec: PrintConfig head-wrap detect zone and thin-wall registry slice

## Goal
Port the adjacent head-wrap detect zone and thin-wall detection option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1485`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6503-6506`: `head_wrap_detect_zone` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1165`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6508-6514`: `detect_thin_wall` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/category/mode/gui metadata beyond the current registry metadata boundary.
- Head-wrap/clumping detection behavior, probe-zone behavior, thin-wall geometric detection, single-line thin-wall generation, extrusion behavior, and G-code behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6516+`: `change_filament_gcode`, `change_extrusion_role_gcode`, `filament_change_extrusion_role_gcode`, top-surface options, and following options.
- Slicing, geometry, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`: add `detect_thin_wall` after `detect_overhang_wall` and before `different_settings_to_system`, preserving sorted order.
- `crates/ares-core/src/options/registry/definitions/table/middle.rs`: add `head_wrap_detect_zone` after `has_scarf_joint_seam` and before `high_current_on_filament_swap`, preserving sorted order.
- `crates/ares-core/src/options/registry/tests/keys/first.rs`: add both covered expected keys in the same sorted positions.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/head_wrap_thin_wall.rs`: add metadata assertions for both definitions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_head_wrap_thin_wall.rs`: add public lookup assertions for both definitions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs` and `crates/ares-core/src/options/tests/registry_helpers/known_count/values/tail_values.rs`: add known-count fixture values and expected counts without touching near-limit `values.rs`.
- `docs/roadmap.md` and `docs/milestones/m129-print-config-head-wrap-thin-wall-registry.md`: milestone sequencing docs.

## Included option definitions

- `head_wrap_detect_zone` (`coPoints`, default empty points represented by Ares registry default string `"0x0"`, field at `PrintConfig.hpp:1485`, definition lines 6503-6506, Ares kind `Points`)
- `detect_thin_wall` (`coBool`, default `false`, field at `PrintConfig.hpp:1165`, definition lines 6508-6514, Ares kind `Bool`)

## Functional requirements

1. Add the two missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, head-wrap detection behavior, thin-wall slicing behavior, geometry behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `change_filament_gcode`, `change_extrusion_role_gcode`, `filament_change_extrusion_role_gcode`, top-surface options, or following options from `PrintConfig.cpp:6516+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove both covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for both covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/geometry/extrusion/G-code behavior, and following `PrintConfig.cpp:6516+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
