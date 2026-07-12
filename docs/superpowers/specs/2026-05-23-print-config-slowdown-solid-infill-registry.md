# M110 Spec: PrintConfig slowdown and solid-infill registry slice

## Goal
Port the adjacent slowdown and solid-infill option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1559`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5629-5637`: `slow_down_layer_time` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1160`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5639-5646`: `minimum_sparse_infill_area` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1161`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5648-5655`: `solid_infill_filament` option definition.

Related upstream behavior explicitly deferred:

- UI full-label/tooltip/sidetext/min/max/mode/category metadata beyond the current registry boundary.
- Layer-time slowdown behavior, sparse-area replacement with internal solid infill, solid-infill extruder selection, and current Ares runtime behavior changes.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5657+`: `internal_solid_infill_line_width`, `internal_solid_infill_speed`, spiral-mode options, and following options.
- Slicing, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/late_tail.rs`: add `minimum_sparse_infill_area` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add `slow_down_layer_time` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/tail_final.rs`: add `solid_infill_filament` in sorted order.
- Registry key, metadata, fixture-count, and public lookup tests cover all three definitions.
- `docs/roadmap.md` and `docs/milestones/m110-print-config-slowdown-solid-infill-registry.md`: milestone sequencing docs.

## Included option definitions

- `slow_down_layer_time` (`coFloats`, default `{ 5.0f }`, field at `PrintConfig.hpp:1559`, definition lines 5629-5637, Ares kind `Floats`)
- `minimum_sparse_infill_area` (`coFloat`, default `15`, field at `PrintConfig.hpp:1160`, definition lines 5639-5646, Ares kind `Float`)
- `solid_infill_filament` (`coInt`, default `1`, field at `PrintConfig.hpp:1161`, definition lines 5648-5655, Ares kind `Int`)

## Functional requirements

1. Add the three missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/skirt/solid-infill API behavior.
5. Do not add typed parsing/accessors, slowdown behavior, sparse-area solid-fill replacement behavior, solid-infill extruder selection behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add following internal-solid-infill / spiral options from `PrintConfig.cpp:5657+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the three covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts covered keys while preserving unknown options.
- Public lookup coverage exists for all three covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:5657+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
