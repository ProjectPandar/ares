# Task 22O.67 architecture decision record

## Status

Accepted and implemented; final independent implementation review pending.

## Decision

Port pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`OrcaSlicer/src/libslic3r/PrintObject.cpp:3345-3350`, as one private,
production-unwired internal-infill rebuild operation in ordinary module
`project_slice/prepare_infill/bridge_over_infill/internal_infill_rebuild.rs`.

## Source boundary and dependencies

Included behavior is exactly: stable `stInternal` filtering at
`SurfaceCollection.cpp:45-52`; one default NonZero/no-safety
`diff_ex(internal_infills, cut_from_infill)`; one default NonZero/no-safety
`diff_ex(first_result, additional_ensuring)`; then source-order construction of
fresh `Surface(stInternal, ep)` records. Direct dependencies are
`Surface.hpp:9-77,116-157`, `ClipperUtils.hpp:183-220,265-307,442-455`, and
`ClipperUtils.cpp:734-769`. Rust uses `RegionSurface`,
`difference_ex_polygons`, and `difference_ex`.

## Exact private seam

```rust
pub(in crate::project_slice) fn rebuild_internal_infills(
    fill_surfaces: &[RegionSurface],
    cut_from_infill: &[Polygon],
    additional_ensuring: &[ExPolygon],
) -> Result<Vec<RegionSurface>, ClipperError>;
```

Inputs are the current region predecessor surfaces, O65 cut polygons, and O66
ensuring ExPolygons. Output owns only freshly rebuilt Internal surfaces.

## Required semantics

- Select exact `RegionSurfaceKind::Internal` entries in input order. Clone their
  ExPolygons only to adapt the borrowed C++ provider to the existing Rust API;
  preserve component and contour/hole order.
- Execute exactly once and sequentially: selected internals minus O65 cut, then
  that owned result minus O66 ensuring. Both are default NonZero/no-safety.
- Materialize every final ExPolygon in engine order with
  `RegionSurface::new(Internal, ep)`, resetting metadata to thickness `-1.0`,
  thickness layers `1`, bridge angle `-1.0`, extra perimeters `0`.
- No early empty gate: empty selected input, empty clips, or complete first
  erosion still reaches both operations. Return the first Clipper error; no
  partial output escapes. Preserve every borrowed value and allocation.
- Add no union, safety offset, sort, deduplication, validation, fallback,
  option lookup, kind expansion, region mutation, composer, or lifecycle.

Production trusts current-layer normalized region surfaces, exact O65 current
cut polygons, and exact O66 current-region ensuring ExPolygons. Coordinates and
topology are Clipper-safe. O67 performs no validation.

## Included and deferred

Included only `PrintObject.cpp:3345-3350` and its direct closure. Deferred:
`new_surfaces` context at 3339; O66 at 3341-3343; bridge conversion at
3352-3367; solid recomposition 3368-3374; debug 3376-3383; destructive region
replacement 3385-3386; map/layer traversal; second pass 3391+; composer,
lifecycle, extrusion, motion, G-code, CLI, and golden parity.

## Verification constraints

Behavioral RED must freeze exact kind filtering/order/hole topology, two
operation calls and operands, metadata reset, empty traversal, first-error
precedence, natural range errors, output order, repeatability, and complete
input allocation preservation. Compiling mutations must kill wrong/missing
filter, reorder, skipped/repeated/per-surface/batched/reversed/safety
operations, wrong second subject, swapped order, early empty, ignored errors,
output sort/reorientation, metadata preservation/wrong kind, then restore
byte-exact. Compiler failures are invalid evidence.

Every Rust source is at most 399 LOC, tests use ordinary child modules, and
include macros may not split source.

Behavioral RED is preserved in `/tmp/task22o67-behavioral-red.log`. Focused
6/6 and dependency 782/782 pass in `/tmp/pi-unified-exec-981-8fed4331.log`;
workspace 6,442/6,442 with two skipped passes in
`/tmp/pi-unified-exec-982-893c4a5a.log`; strict Clippy, rustfmt, wasm32, four
desktop targets, LOC/static, pinned Orca and no-staged gates pass in
`/tmp/pi-unified-exec-984-8aad9912.log`.

The compile-validating 18-mutation audit script SHA-256 is
`7eb0d586597d4a836ae9b10f4d0be5463b68eafac43ceebf406cab9d6bdb8cf8`;
output SHA-256 is
`95555f3fe5a1065c9cdff033f9367239c250f7c72952e62d78132273326f329b`.
All 18 mutations die and production restores byte-exact at
`e6cb24825b1727c8509af9645b204040d468d8dc09ad26118d40dbc055bbcb96`.
Final independent six-axis review remains required.
