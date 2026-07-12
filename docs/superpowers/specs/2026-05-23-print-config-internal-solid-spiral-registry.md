# M111 Spec: PrintConfig internal solid infill and spiral registry slice

## Goal
Port the adjacent internal-solid-infill and spiral option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1162`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5657-5667`: `internal_solid_infill_line_width` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1163`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5669-5676`: `internal_solid_infill_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1560`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5678-5684`: `spiral_mode` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1561`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5686-5691`: `spiral_mode_smooth` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1562`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5693-5704`: `spiral_mode_max_xy_smoothing` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1564`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5706-5715`: `spiral_starting_flow_ratio` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1563`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5717-5726`: `spiral_finishing_flow_ratio` option definition.

Related upstream behavior explicitly deferred:

- UI full-label/tooltip/sidetext/min/max/max_literal/ratio_over/mode/category metadata beyond the current registry boundary.
- Internal solid infill line-width/speed behavior, spiral-vase path generation, Z-move smoothing, XY smoothing, spiral starting/finishing flow transitions, and current Ares runtime behavior changes.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5728+`: `timelapse_type`, standby/preheat, and following options.
- Slicing, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/middle_tail.rs`: add `internal_solid_infill_line_width` and `internal_solid_infill_speed` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/tail_final.rs`: add `spiral_finishing_flow_ratio`, `spiral_mode`, `spiral_mode_max_xy_smoothing`, `spiral_mode_smooth`, and `spiral_starting_flow_ratio` in sorted order.
- Registry key, metadata, fixture-count, and public lookup tests cover all seven definitions.
- `docs/roadmap.md` and `docs/milestones/m111-print-config-internal-solid-spiral-registry.md`: milestone sequencing docs.

## Included option definitions

- `internal_solid_infill_line_width` (`coFloatOrPercent`, default `0`, field at `PrintConfig.hpp:1162`, definition lines 5657-5667, Ares kind `FloatOrPercent`)
- `internal_solid_infill_speed` (`coFloat`, default `100`, field at `PrintConfig.hpp:1163`, definition lines 5669-5676, Ares kind `Float`)
- `spiral_mode` (`coBool`, default `false`, field at `PrintConfig.hpp:1560`, definition lines 5678-5684, Ares kind `Bool`)
- `spiral_mode_smooth` (`coBool`, default `false`, field at `PrintConfig.hpp:1561`, definition lines 5686-5691, Ares kind `Bool`)
- `spiral_mode_max_xy_smoothing` (`coFloatOrPercent`, default `200%`, field at `PrintConfig.hpp:1562`, definition lines 5693-5704, Ares kind `FloatOrPercent`)
- `spiral_starting_flow_ratio` (`coFloat`, default `0`, field at `PrintConfig.hpp:1564`, definition lines 5706-5715, Ares kind `Float`)
- `spiral_finishing_flow_ratio` (`coFloat`, default `0`, field at `PrintConfig.hpp:1563`, definition lines 5717-5726, Ares kind `Float`)

## Functional requirements

1. Add the seven missing options using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/skirt/infill/speed API behavior.
5. Do not add typed parsing/accessors, internal-solid-infill runtime behavior, spiral-vase path generation, smoothing behavior, flow-transition behavior, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add following timelapse / standby / preheat options from `PrintConfig.cpp:5728+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the seven covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts covered keys while preserving unknown options.
- Public lookup coverage exists for all seven covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:5728+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
