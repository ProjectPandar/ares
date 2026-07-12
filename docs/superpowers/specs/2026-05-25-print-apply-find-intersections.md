# PrintApply find_intersections control flow Spec

## Goal
Port the control-flow semantics of OrcaSlicer's local `find_intersections` lambda into `ares-core` as a private staged helper, while deferring the actual Clipper boolean operation and the later old/new id-set comparison.

## Rewrite gate mapping
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:326-333`: define `find_intersections`, initialize a sorted result set, iterate `contours` by index, call `intersection(poly, contours[i])`, insert index `i` when the result is non-empty, and return the set.

Context only:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:335-340`: deferred old/new `find_intersections` calls, set comparison, and final changed result.
- `OrcaSlicer/src/libslic3r/ClipperUtils.hpp:496-508`: `intersection(...)` declarations and default `ApplySafetyOffset::No` context.
- `OrcaSlicer/src/libslic3r/ClipperUtils.cpp:696-697`: polygon/polygon `intersection(...)` delegates to `_clipper(... ctIntersection ...)`.

## Approval gate
Do not begin Task 1, tests, implementation, or any code changes for M269 until this M269 plan/spec review returns `APPROVE`.

## Requirements
- Extend private module `crates/ares-core/src/print_apply.rs`; do not add public APIs.
- Add a private helper that works over one scaled target polygon, a slice of scaled contour polygons, and an injected/private intersection callback, for example:
  - `fn find_intersection_ids<F>(poly: &[ScaledPoint], contours: &[Vec<ScaledPoint>], intersection: F) -> BTreeSet<usize> where F: FnMut(&[ScaledPoint], &[ScaledPoint]) -> Vec<Vec<ScaledPoint>>`.
- Iterate `contours` once in source/index order.
- Call the intersection callback with the target polygon as subject and the current contour polygon as clip.
- Insert only the zero-based contour index when the callback result is non-empty.
- Return a sorted deterministic set equivalent to upstream `std::set<int>`.
- Do not compare old/new id sets in this milestone; `PrintApply.cpp:335-340` remains deferred context.
- Do not introduce a geometry dependency or implement the actual Clipper intersection operation in this milestone.
- Do not implement Clipper `ctIntersection`, fill rules, safety offsets, old/new id comparison, final printable-filament changed result, public APIs, profile loading, UI runtime behavior, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.

## Non-goals
- No actual polygon boolean operation.
- No old/new intersection-id comparison or final printable-filament changed result.
- No public `is_printable_filament_changed` API wiring.
- No profile loading, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.
