# PrintApply all-extruder intersection append Spec

## Goal
Port the control-flow semantics of OrcaSlicer's all-extruder `intersection({printable_poly}, extruder_polys)` branch into `ares-core` as a private staged helper, while deferring the actual Clipper boolean operation.

## Rewrite gate mapping
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:323-324`: compute `intersection({printable_poly}, extruder_polys)` and append `all_extruder_polys[0]` only when the result is non-empty.

Context only:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:326-340`: deferred intersection-id lambda, old/new intersection-id comparison, and final changed result.
- `OrcaSlicer/src/libslic3r/ClipperUtils.hpp:496-508`: `intersection(...)` declarations and default `ApplySafetyOffset::No` context.
- `OrcaSlicer/src/libslic3r/ClipperUtils.cpp:702-703`: polygons/polygons `intersection(...)` delegates to `_clipper(... ctIntersection ...)`.

## Approval gate
Do not begin Task 1, tests, implementation, or any code changes for M268 until this M268 plan/spec review returns `APPROVE`.

## Requirements
- Extend private module `crates/ares-core/src/print_apply.rs`; do not add public APIs.
- Add a private helper that works over M266 scaled polygons, an existing split polygon vector from M267, and an injected/private intersection callback, for example:
  - `fn append_all_extruder_intersection_first_result<F>(polygons: &ScaledPrintableAreaPolygons, split_polys: &mut Vec<Vec<ScaledPoint>>, intersection: F) where F: FnOnce(&[Vec<ScaledPoint>], &[Vec<ScaledPoint>]) -> Vec<Vec<ScaledPoint>>`.
- Call the intersection callback exactly once, with a single-subject polygon slice equivalent to `{printable_poly}` and all extruder polygons as the clip polygons.
- If the callback result is empty, append nothing.
- If the callback result has one or more polygons, append only the first result polygon to the end of `split_polys`.
- Preserve all existing `split_polys` entries and their order before appending.
- Do not introduce a geometry dependency or implement the actual Clipper intersection operation in this milestone.
- Do not implement Clipper `ctIntersection`, fill rules, safety offsets, intersection-id comparison, final printable-filament changed result, public APIs, profile loading, UI runtime behavior, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.

## Non-goals
- No actual polygon boolean operation.
- No intersection-id comparison or final printable-filament changed result.
- No public API or UI-facing API.
- No profile loading, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.
