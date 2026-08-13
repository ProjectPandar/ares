# Task 22O.76 architecture decision record

## Status

Accepted for implementation. Decision date: 2026-08-13.

## Decision

Port the first KSR-active `Layer::make_fills` vertical slice from pinned
OrcaSlicer commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `src/libslic3r/Fill/Fill.cpp:1213-1224,1234-1357` for obtaining complete
  grouped fills and the CrossHatch branch's ordered ExPolygon iteration; and
- `src/libslic3r/Fill/FillBase.cpp:133-184` for converting generated polylines
  into one extrusion-entity collection with role, flow, width, and height.

The bounded Rust seam is crate-private
`project_slice::fill_entities::generate_crosshatch_layer(prepared,
object_index, layer_index)`. It borrows the prepared external-surface graph,
calls the complete `group_fills`, processes only configured CrossHatch groups,
and returns owned ordered collections. Each group ExPolygon produces a
collection only when CrossHatch returns nonempty polylines, preserving source
order.

The result uses a fill-owned rendering-neutral entity vocabulary under
`project_slice::fill_entities`, not the public legacy top-level
`extrusion_entity` scaffold and not the perimeter loop tree. An extrusion path
owns fixed-coordinate `Polyline`, grouped `ExtrusionRole`, `mm3_per_mm`, width,
height, and source reverse/no-sort flags needed by later fill ordering.

## Included behavior

- complete O74 grouping and 3MF-derived options;
- source `float(0.01 * density)` conversion;
- CrossHatch generation with grouped z, spacing, overlap, angle, multiline,
  and anchor lengths;
- the ordinary internal-flow rule from `Fill.cpp:1259` and
  `FillBase.cpp:148-163`: sparse internal CrossHatch retains grouped Flow
  without recomputing it from adjusted spacing;
- one collection per nonempty source ExPolygon in group/ExPolygon order;
- exact grouped extrusion role, Flow `mm3_per_mm`, width, and height;
- deterministic owned output, immutable graph input, and atomic errors.

## Deferred behavior

All non-CrossHatch fillers, adjusted-solid Flow, bridge/internal-bridge density
overrides, elephant-foot density manipulation, no-overlap gap fill,
Concentric/Arachne, LockedZag, thin-fill append, fill ordering, lifecycle
activation, ironing, support, extrusion sequencing, motion, and G-code remain
later source-cited slices. The seam rejects a reached non-CrossHatch group with
`UnsupportedProjectFeature("fill_pattern")`; it has no silent skip or fallback.

This task adds no public API, filesystem access, UI, terminal behavior,
threading, OpenGL, native-only dependency, Cargo feature, hardcoded fixture
branch, or alternate option source. Public slicing remains
`ProjectSlicingIncomplete`.

## Verification

Three graph-native tests pass. They freeze a focused Internal CrossHatch
collection's ordered paths and exact metadata bits (`mm3_per_mm =
0x3fb4d7aca0000000`, width `0x3ee66666`, height `0x3e4ccccd`), repeatability,
immutability, non-CrossHatch non-fallback, and atomic grouping range errors.
Strict workspace all-target/all-feature Clippy, rustfmt, and diff checks pass.
The largest new Rust file is 58 LOC; tests remain separate ordinary modules and
no source-splitting macro was added.

The all-KSR entity oracle remains deferred until every KSR-active filler class
needed by `Layer::make_fills` is present; this milestone does not mislabel a
CrossHatch-only aggregate as complete layer fill output.
