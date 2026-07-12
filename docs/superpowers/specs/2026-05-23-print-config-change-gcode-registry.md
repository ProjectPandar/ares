# M130 Spec: PrintConfig change G-code registry slice

## Goal
Port the adjacent filament-change and extrusion-role-change G-code option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1392`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6516-6523`: `change_filament_gcode` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1393`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6525-6532`: `change_extrusion_role_gcode` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1395`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6534-6541`: `filament_change_extrusion_role_gcode` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/multiline/full-width/height/mode metadata beyond the current registry metadata boundary.
- Filament-change G-code insertion, extrusion-role-change G-code insertion, tool-change behavior, active-filament-specific behavior, placeholder expansion, slicing behavior, extrusion behavior, and G-code behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6543+`: `top_surface_line_width`, `top_surface_speed`, `top_shell_layers`, and following options.
- Filesystem behavior, network behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/early.rs`: add `change_extrusion_role_gcode` and `change_filament_gcode` after `chamber_temperature` and before `close_additional_fan_first_x_layers`, preserving sorted order.
- `crates/ares-core/src/options/registry/definitions/table/pre_middle_filament.rs`: add `filament_change_extrusion_role_gcode` after `filament_adhesiveness_category` and before `filament_change_length`, preserving sorted order.
- `crates/ares-core/src/options/registry/tests/keys/first.rs`: add all three covered expected keys in the same sorted positions.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/change_gcode.rs`: add metadata assertions for all three definitions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_change_gcode.rs`: add public lookup assertions for all three definitions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs` and `crates/ares-core/src/options/tests/registry_helpers/known_count/values/tail_values.rs`: add known-count fixture values and expected counts without touching near-limit `values.rs`.
- `docs/roadmap.md` and `docs/milestones/m130-print-config-change-gcode-registry.md`: milestone sequencing docs.

## Included option definitions

- `change_extrusion_role_gcode` (`coString`, default empty string, field at `PrintConfig.hpp:1393`, definition lines 6525-6532, Ares kind `String`)
- `change_filament_gcode` (`coString`, default empty string, field at `PrintConfig.hpp:1392`, definition lines 6516-6523, Ares kind `String`)
- `filament_change_extrusion_role_gcode` (`coStrings`, default single empty string represented by Ares registry default string `""`, field at `PrintConfig.hpp:1395`, definition lines 6534-6541, Ares kind `Strings`)

## Functional requirements

1. Add the three missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, filament-change G-code behavior, extrusion-role-change G-code behavior, placeholder expansion, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `top_surface_line_width`, `top_surface_speed`, `top_shell_layers`, or following options from `PrintConfig.cpp:6543+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove all three covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all three covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:6543+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
