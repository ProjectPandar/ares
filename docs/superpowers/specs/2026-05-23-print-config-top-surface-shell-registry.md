# M131 Spec: PrintConfig top-surface and top-shell registry slice

## Goal
Port the adjacent top-surface line-width/speed and top-shell layers/thickness option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1166`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6543-6553`: `top_surface_line_width` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1169`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6555-6562`: `top_surface_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1167`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6564-6573`: `top_shell_layers` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1168`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6575-6584`: `top_shell_thickness` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/category/sidetext/full-label/min/max/mode/ratio-over metadata beyond the current registry metadata boundary.
- Top-surface line-width computation over nozzle diameter, top-surface speed planning, top-shell layer adjustment from thickness, slicing behavior, geometry behavior, extrusion behavior, and G-code behavior.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6586+`: `top_surface_density`, `bottom_surface_density`, and following options.
- Filesystem behavior, network behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/tail_terminal_suffix.rs`: add `top_shell_layers`, `top_shell_thickness`, `top_surface_line_width`, and `top_surface_speed` in lexicographic order around the existing top-surface/top-solid definitions, preserving sorted order.
- `crates/ares-core/src/options/registry/tests/keys/second.rs`: add all four covered expected keys in the same sorted positions.
- `crates/ares-core/src/options/registry/tests/metadata.rs` and `crates/ares-core/src/options/registry/tests/metadata/top_surface_shell.rs`: add metadata assertions for all four definitions.
- `crates/ares-core/src/options/tests.rs` and `crates/ares-core/src/options/tests/registry_lookup_top_surface_shell.rs`: add public lookup assertions for all four definitions.
- `crates/ares-core/src/options/tests/registry_helpers/known_count.rs` and `crates/ares-core/src/options/tests/registry_helpers/known_count/values/tail_values.rs`: add known-count fixture values and expected counts without touching near-limit `values.rs`.
- `docs/roadmap.md` and `docs/milestones/m131-print-config-top-surface-shell-registry.md`: milestone sequencing docs.

## Included option definitions

- `top_shell_layers` (`coInt`, default `4`, field at `PrintConfig.hpp:1167`, definition lines 6564-6573, Ares kind `Int`)
- `top_shell_thickness` (`coFloat`, default `0.6`, field at `PrintConfig.hpp:1168`, definition lines 6575-6584, Ares kind `Float`)
- `top_surface_line_width` (`coFloatOrPercent`, default `0`, field at `PrintConfig.hpp:1166`, definition lines 6543-6553, Ares kind `FloatOrPercent`)
- `top_surface_speed` (`coFloat`, default `100`, field at `PrintConfig.hpp:1169`, definition lines 6555-6562, Ares kind `Float`)

## Functional requirements

1. Add the four missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, line-width computation, top-shell layer adjustment, speed-planning behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `top_surface_density`, `bottom_surface_density`, or following options from `PrintConfig.cpp:6586+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove all four covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all four covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/geometry/extrusion/G-code behavior, and following `PrintConfig.cpp:6586+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
