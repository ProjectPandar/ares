# PrintApply intersection-id set comparison Spec

## Goal
Port the old/new intersection-id set comparison tail of OrcaSlicer's private `is_printable_filament_changed(...)` into `ares-core` as a private staged helper, using the existing M269 `find_intersection_ids(...)` helper and deferring concrete Clipper intersection.

## Rewrite gate mapping
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:335-337`: compute `old_poly_ids = find_intersections(old_poly, split_polys)`, compute `new_poly_ids = find_intersections(new_poly, split_polys)`, and return `old_poly_ids != new_poly_ids`.

Required predecessor context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:326-333`: M269 staged `find_intersections` traversal and sorted-set construction.

Context only:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:297-324`: deferred guard, printable/extruder polygon construction, diff split collection, and all-extruder intersection append.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:339-340`: enclosing function final false fallback remains deferred to full assembly.
- `OrcaSlicer/src/libslic3r/ClipperUtils.hpp:496-508` and `ClipperUtils.cpp:696-697`: polygon/polygon intersection backend context only.

## Approval gate
Do not begin Task 1, tests, implementation, or any code changes for M270 until this M270 plan/spec review returns `APPROVE`.

## Requirements
- Extend private module `crates/ares-core/src/print_apply.rs`; do not add public APIs.
- Add a private helper over scaled old/new polygons, split polygons, and an injected/private intersection callback, for example:
  - `fn printable_filament_intersection_ids_changed<F>(old_poly: &[ScaledPoint], new_poly: &[ScaledPoint], split_polys: &[Vec<ScaledPoint>], intersection: F) -> bool where F: FnMut(&[ScaledPoint], &[ScaledPoint]) -> Vec<Vec<ScaledPoint>>`.
- Call `find_intersection_ids(old_poly, split_polys, ...)` and `find_intersection_ids(new_poly, split_polys, ...)` exactly once each.
- Use the same `split_polys` slice for old and new polygons.
- Return `false` when the two sorted id sets are equal.
- Return `true` when the two sorted id sets differ.
- Preserve callback order implied by the upstream source: all old-polygon contour checks happen before all new-polygon contour checks.
- Do not introduce a geometry dependency or implement the actual Clipper intersection operation in this milestone.
- Do not implement printable-area parsing/scaling, split polygon assembly, full `is_printable_filament_changed(...)`, public APIs, profile loading, UI runtime behavior, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.

## Non-goals
- No actual polygon boolean operation.
- No full printable-filament predicate assembly.
- No public `is_printable_filament_changed` API wiring.
- No profile loading, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.
