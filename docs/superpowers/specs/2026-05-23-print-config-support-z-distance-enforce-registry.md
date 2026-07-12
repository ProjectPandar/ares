# M117 Spec: PrintConfig support Z-distance and enforced layers registry slice

## Goal
Port the adjacent support top/bottom Z-distance and enforced-support-layers option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:956`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5981-6000`: `support_top_z_distance` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:957`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6002-6011`: `support_bottom_z_distance` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:958`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6013-6025`: `enforce_support_layers` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/categories/sidetext/minimums/maximums/modes beyond the current registry metadata boundary.
- Support Z-gap application, independent support layer-height rounding, enforced support material generation, and support geometry.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6027+`: `support_filament` and following support options.
- Slicing, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`: add `enforce_support_layers` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/tail_terminal.rs`: add `support_bottom_z_distance` and `support_top_z_distance` in sorted order while keeping the file below 400 LOC.
- Registry key, metadata, fixture-count, and public lookup tests cover all three definitions.
- `docs/roadmap.md` and `docs/milestones/m117-print-config-support-z-distance-enforce-registry.md`: milestone sequencing docs.

## Included option definitions

- `support_top_z_distance` (`coFloat`, default `0.2`, field at `PrintConfig.hpp:956`, definition lines 5981-6000, Ares kind `Float`)
- `support_bottom_z_distance` (`coFloat`, default `0.2`, field at `PrintConfig.hpp:957`, definition lines 6002-6011, Ares kind `Float`)
- `enforce_support_layers` (`coInt`, default `0`, field at `PrintConfig.hpp:958`, definition lines 6013-6025, Ares kind `Int`)

## Functional requirements

1. Add the three missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, support Z-gap behavior, support layer-height rounding, enforced support generation, support geometry, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `support_filament` or following options from `PrintConfig.cpp:6027+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the three covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts covered keys while preserving unknown options.
- Public lookup coverage exists for all three covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:6027+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
