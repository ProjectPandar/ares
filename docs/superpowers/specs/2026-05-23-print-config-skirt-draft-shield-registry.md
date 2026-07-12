# M109 Spec: PrintConfig skirt and draft-shield registry slice

## Goal
Port the adjacent skirt and draft-shield option-definition slice from `libslic3r::PrintConfigDef::init_fff_params` into `ares-core` option registry metadata.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1552`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5540-5547`: `skirt_distance` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:927`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5549-5557`: `skirt_start_angle` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1553`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5559-5565`: `skirt_height` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1557`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5567-5571`: `single_loop_draft_shield` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:290-292`, `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1512`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:443-447`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5573-5586`: `draft_shield` enum map and option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:286-288`, `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1555`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:437-441`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5588-5598`: `skirt_type` enum map and option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1554`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5600-5607`: `skirt_loops` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1556`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5609-5616`: `skirt_speed` option definition.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1558`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5618-5627`: `min_skirt_length` option definition.

Related upstream behavior explicitly deferred:

- UI full-label/tooltip/sidetext/min/max/mode/category metadata beyond the current registry boundary.
- Skirt generation behavior, skirt start-angle placement, draft-shield geometry, single-loop-after-first-layer behavior, combined/per-object skirt behavior, minimum-skirt-length loop calculation, and current Ares skirt runtime behavior changes.
- Typed accessors or behavior changes for the newly registered keys.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5629+`: `slow_down_layer_time`, `minimum_sparse_infill_area`, solid infill filament, and following options.
- Slicing, extrusion, G-code behavior, filesystem behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`: add `draft_shield` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/late_tail.rs`: add `min_skirt_length` in sorted order.
- `crates/ares-core/src/options/registry/definitions/table/tail.rs`: add `single_loop_draft_shield`, `skirt_start_angle`, and `skirt_type` in sorted order; update existing `skirt_distance`, `skirt_height`, `skirt_loops`, and `skirt_speed` source citations only.
- Registry key, metadata, fixture-count, and public lookup tests cover all nine definitions.
- `docs/roadmap.md` and `docs/milestones/m109-print-config-skirt-draft-shield-registry.md`: milestone sequencing docs.

## Included option definitions

- `skirt_distance` (`coFloat`, default `2`, field at `PrintConfig.hpp:1552`, definition lines 5540-5547, Ares kind `Float`)
- `skirt_start_angle` (`coFloat`, default `-135`, field at `PrintConfig.hpp:927`, definition lines 5549-5557, Ares kind `Float`)
- `skirt_height` (`coInt`, default `1`, field at `PrintConfig.hpp:1553`, definition lines 5559-5565, Ares kind `Int`)
- `single_loop_draft_shield` (`coBool`, default `false`, field at `PrintConfig.hpp:1557`, definition lines 5567-5571, Ares kind `Bool`)
- `draft_shield` (`coEnum`, default `disabled`, enum at `PrintConfig.hpp:290-292`, field at `PrintConfig.hpp:1512`, enum map at `PrintConfig.cpp:443-447`, definition lines 5573-5586, Ares kind `Enum`)
- `skirt_type` (`coEnum`, default `combined`, enum at `PrintConfig.hpp:286-288`, field at `PrintConfig.hpp:1555`, enum map at `PrintConfig.cpp:437-441`, definition lines 5588-5598, Ares kind `Enum`)
- `skirt_loops` (`coInt`, default `1`, field at `PrintConfig.hpp:1554`, definition lines 5600-5607, Ares kind `Int`)
- `skirt_speed` (`coFloat`, default `50`, field at `PrintConfig.hpp:1556`, definition lines 5609-5616, Ares kind `Float`)
- `min_skirt_length` (`coFloat`, default `0`, field at `PrintConfig.hpp:1558`, definition lines 5618-5627, Ares kind `Float`)

## Functional requirements

1. Add the five missing options and update four existing skirt option source citations using existing value kinds only.
2. Preserve existing public API function signatures: `option_definitions()` and `option_definition(key)` remain unchanged.
3. Preserve the single sorted merged `OPTION_DEFINITIONS` slice used by binary-search lookup.
4. Preserve `SliceOptions` unknown-value storage and current public slicing/skirt API behavior.
5. Do not add typed parsing/accessors, skirt generation changes, draft-shield geometry behavior, per-object skirt behavior, minimum-skirt-length loop calculation, slicing behavior, extrusion behavior, or G-code behavior for these options in this milestone.
6. Do not add following slowdown / minimum sparse infill options from `PrintConfig.cpp:5629+`.
7. Do not add new pipeline stages, crates, dependencies, filesystem behavior, network behavior, or UI behavior.
8. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Registry tests prove the nine covered keys have expected kinds, default values, and source line references.
- The merged definition stream remains sorted and binary-search compatible.
- `known_definition_count()` counts covered keys while preserving unknown options.
- Public lookup coverage exists for all nine covered definitions.
- Plan/spec explicitly account for deferred UI metadata, current skirt runtime behavior, slicing/extrusion/G-code behavior, and following `PrintConfig.cpp:5629+` scope.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
