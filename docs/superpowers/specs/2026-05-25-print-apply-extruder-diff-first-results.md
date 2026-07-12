# PrintApply extruder diff first-result collection Spec

## Goal
Port the control-flow semantics of OrcaSlicer's per-extruder `diff(printable_poly, poly)` loop into `ares-core` as a private staged helper, while deferring the actual Clipper boolean operation.

## Rewrite gate mapping
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:317-320`: iterate `extruder_polys`, call `diff(printable_poly, poly)`, append `res[0]` only when the result is non-empty.

Context only:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:323-340`: deferred all-extruder intersection, split-polygon tail append, intersection-id comparison, and final changed result.
- `OrcaSlicer/src/libslic3r/ClipperUtils.hpp:429-433`: `diff(...)` declarations and default `ApplySafetyOffset::No` context.
- `OrcaSlicer/src/libslic3r/ClipperUtils.cpp:676-679`: polygon/polygon `diff(...)` implementation delegates to `_clipper(... ctDifference ...)`.

## Approval gate
Do not begin Task 1, tests, implementation, or any code changes for M267 until this M267 plan/spec review returns `APPROVE`.

## Requirements
- Extend private module `crates/ares-core/src/print_apply.rs`; do not add public APIs.
- Add a private helper that works over M266 scaled polygons and an injected/private difference callback, for example:
  - `fn collect_extruder_diff_first_results<F>(polygons: &ScaledPrintableAreaPolygons, diff: F) -> Vec<Vec<ScaledPoint>> where F: FnMut(&[ScaledPoint], &[ScaledPoint]) -> Vec<Vec<ScaledPoint>>`.
- Call the difference callback once per extruder polygon, in extruder source order, with `printable` as subject and the current extruder polygon as clip.
- If a callback result is empty, append nothing.
- If a callback result has one or more polygons, append only the first result polygon.
- Preserve append order across extruder polygons.
- Do not introduce a geometry dependency or implement the actual Clipper difference operation in this milestone.
- Do not implement Clipper `ctDifference`, fill rules, safety offsets, full split polygon assembly, all-extruder intersection, intersection-id comparison, public APIs, profile loading, UI runtime behavior, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.

## Non-goals
- No actual polygon boolean operation.
- No all-extruder intersection branch.
- No public API or UI-facing API.
- No profile loading, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.
