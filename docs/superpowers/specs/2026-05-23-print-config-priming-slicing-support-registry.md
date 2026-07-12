# M115 Spec: PrintConfig priming, slicing mode, Z offset, and support-enable registry slice

## Goal
Port the adjacent single-extruder priming, slice gap closing, slicing mode, Z offset, and support-enable option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1390`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5863-5867`: `single_extruder_multi_material_priming` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:946`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5869-5877`: `slice_closing_radius` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:162-170`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:305-310`, `OrcaSlicer/src/libslic3r/PrintConfig.hpp:947`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5879-5891`: `slicing_mode` enum and option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1609`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5893-5901`: `z_offset` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:948`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5903-5908`: `enable_support` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/categories/sidetext/minimums/modes beyond the current registry metadata boundary.
- Single-extruder multi-material priming, slice gap closing, slicing-mode polygon fill behavior, Z-offset application, support generation, and current Ares runtime behavior changes.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5910+`: `support_type`, support/object distances, support styles, and following options.
- Slicing, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`: add `enable_support` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/tail_raft.rs`: add `single_extruder_multi_material_priming`, `slice_closing_radius`, and `slicing_mode` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/tail_terminal.rs`: add `z_offset` in sorted order.
- Registry key, metadata, fixture-count, and public lookup tests cover all five definitions.
- `docs/roadmap.md` and `docs/milestones/m115-print-config-priming-slicing-support-registry.md`: milestone sequencing docs.

## Included option definitions

- `single_extruder_multi_material_priming` (`coBool`, default `false`, field at `PrintConfig.hpp:1390`, definition lines 5863-5867, Ares kind `Bool`)
- `slice_closing_radius` (`coFloat`, default `0.049`, field at `PrintConfig.hpp:946`, definition lines 5869-5877, Ares kind `Float`)
- `slicing_mode` (`coEnum`, `SlicingMode`, default `regular`, enum keys `regular`/`even_odd`/`close_holes`, enum lines `PrintConfig.hpp:162-170` and `PrintConfig.cpp:305-310`, field at `PrintConfig.hpp:947`, definition lines 5879-5891, Ares kind `Enum`)
- `z_offset` (`coFloat`, default `0`, field at `PrintConfig.hpp:1609`, definition lines 5893-5901, Ares kind `Float`)
- `enable_support` (`coBool`, default `false`, field at `PrintConfig.hpp:948`, definition lines 5903-5908, Ares kind `Bool`)

## Functional requirements

1. Add the five missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, priming behavior, mesh gap-closing behavior, slicing-mode behavior, Z-offset behavior, support behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `support_type` or following options from `PrintConfig.cpp:5910+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the five covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts covered keys while preserving unknown options.
- Public lookup coverage exists for all five covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:5910+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
