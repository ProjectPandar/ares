# M120 Spec: PrintConfig support interface loop, filament, layers, and spacing registry slice

## Goal
Port the adjacent support interface loop-pattern, interface filament, top/bottom interface layers, and top interface spacing option definitions from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:962`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6055-6060`: `support_interface_loop_pattern` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:963`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6062-6070`: `support_interface_filament` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:964`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6072-6088`: `support_interface_top_layers` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:965`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6090-6102`: `support_interface_bottom_layers` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:966-967`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6104-6112`: `support_interface_spacing` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/categories/sidetext/minimums/enum UI values/modes beyond the current registry metadata boundary.
- Support interface loop generation, support interface filament routing, top/bottom interface layer-count behavior, top interface spacing behavior, solid interface forcing, and support geometry.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6114+`: `support_bottom_interface_spacing`, `support_interface_speed`, and following support options.
- Slicing, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/tail_support.rs`: add five `support_interface_*` definitions in sorted order.
- Registry key, metadata, fixture-count, and public lookup tests cover all five definitions.
- `docs/roadmap.md` and `docs/milestones/m120-print-config-support-interface-layers-spacing-registry.md`: milestone sequencing docs.

## Included option definitions

- `support_interface_loop_pattern` (`coBool`, default `false`, field at `PrintConfig.hpp:962`, definition lines 6055-6060, Ares kind `Bool`)
- `support_interface_filament` (`coInt`, default `0`, field at `PrintConfig.hpp:963`, definition lines 6062-6070, Ares kind `Int`)
- `support_interface_top_layers` (`coInt`, default `3`, field at `PrintConfig.hpp:964`, definition lines 6072-6088, Ares kind `Int`)
- `support_interface_bottom_layers` (`coInt`, default `0`, field at `PrintConfig.hpp:965`, definition lines 6090-6102, Ares kind `Int`)
- `support_interface_spacing` (`coFloat`, default `0.5`, field/comment at `PrintConfig.hpp:966-967`, definition lines 6104-6112, Ares kind `Float`)

## Functional requirements

1. Add the five missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, support interface loop generation, filament routing, layer-count behavior, spacing behavior, support geometry, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `support_bottom_interface_spacing`, `support_interface_speed`, or following options from `PrintConfig.cpp:6114+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the five covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts the covered keys while preserving unknown options.
- Public lookup coverage exists for all five covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:6114+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
