# M116 Spec: PrintConfig support type and support placement registry slice

## Goal
Port the adjacent support type, support/object placement gap, support pattern angle, and support placement filter option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:195-209`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:342-348`, `OrcaSlicer/src/libslic3r/PrintConfig.hpp:950`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5910-5925`: `support_type` enum and option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5927-5936`: `support_object_xy_distance` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5938-5947`: `support_object_first_layer_gap` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:952`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5949-5957`: `support_angle` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:953`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5959-5964`: `support_on_build_plate_only` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:954`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5967-5972`: `support_critical_regions_only` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:955`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5974-5979`: `support_remove_small_overhang` option definition.

Related upstream behavior explicitly deferred:

- UI labels/tooltips/categories/sidetext/minimums/maximums/modes beyond the current registry metadata boundary.
- Support generation, tree/normal/manual support selection behavior, support enforcer/blocker behavior, support geometry, and support pattern placement.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5981+`: `support_top_z_distance` and following support options.
- Slicing, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/tail_terminal.rs`: interleave seven `support_*` definitions with existing neighboring support keys in sorted order while keeping the file below 400 LOC.
- Registry key, metadata, fixture-count, and public lookup tests cover all seven definitions.
- `docs/roadmap.md` and `docs/milestones/m116-print-config-support-type-placement-registry.md`: milestone sequencing docs.

## Included option definitions

- `support_type` (`coEnum`, `SupportType`, default `normal(auto)`, enum keys `normal(auto)`/`tree(auto)`/`normal(manual)`/`tree(manual)`, enum lines `PrintConfig.hpp:195-209` and `PrintConfig.cpp:342-348`, field at `PrintConfig.hpp:950`, definition lines 5910-5925, Ares kind `Enum`)
- `support_object_xy_distance` (`coFloat`, default `0.35`, definition lines 5927-5936, Ares kind `Float`)
- `support_object_first_layer_gap` (`coFloat`, default `0.2`, definition lines 5938-5947, Ares kind `Float`)
- `support_angle` (`coFloat`, default `0`, field at `PrintConfig.hpp:952`, definition lines 5949-5957, Ares kind `Float`)
- `support_on_build_plate_only` (`coBool`, default `false`, field at `PrintConfig.hpp:953`, definition lines 5959-5964, Ares kind `Bool`)
- `support_critical_regions_only` (`coBool`, default `false`, field at `PrintConfig.hpp:954`, definition lines 5967-5972, Ares kind `Bool`)
- `support_remove_small_overhang` (`coBool`, default `true`, field at `PrintConfig.hpp:955`, definition lines 5974-5979, Ares kind `Bool`)

## Functional requirements

1. Add the seven missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/G-code API behavior.
5. Do not add typed parsing/accessors, support generation behavior, tree/normal/manual support behavior, support enforcer/blocker handling, support geometry, support pattern placement, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add `support_top_z_distance` or following options from `PrintConfig.cpp:5981+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the seven covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts covered keys while preserving unknown options.
- Public lookup coverage exists for all seven covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:5981+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
