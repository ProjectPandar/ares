# M270: PrintApply intersection-id set comparison

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the old/new `find_intersections(...)` call pair and set comparison inside `is_printable_filament_changed(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:335-337`, with the staged `find_intersections` lambda boundary from `PrintApply.cpp:326-333` as required predecessor context. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned print pipeline.

## Exit criteria

- Add a private `ares-core` staged helper that accepts scaled old/new polygons, already assembled split polygons, and an injected/private polygon intersection operation.
- Preserve upstream control-flow semantics from `PrintApply.cpp:335-337`: compute old polygon intersection ids with the staged M269 helper, compute new polygon intersection ids with the same split polygons, and return whether the two sorted id sets differ.
- Reuse the M269 `find_intersection_ids(...)` helper instead of duplicating contour traversal logic.
- Keep actual Clipper polygon/polygon intersection implementation deferred; this milestone stages only the old/new set comparison semantics around the injected callback.
- Do not implement Clipper `ctIntersection`, fill rules, safety offsets, printable-area parsing/scaling, split polygon assembly, full `is_printable_filament_changed(...)`, public API wiring, profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
