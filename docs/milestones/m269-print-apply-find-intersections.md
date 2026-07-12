# M269: PrintApply find_intersections control flow

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the `find_intersections` lambda inside `is_printable_filament_changed(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:326-333`, with polygon/polygon intersection backend context from `ClipperUtils.hpp:496-508` and `ClipperUtils.cpp:696-697`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned print pipeline.

## Exit criteria

- Add a private `ares-core` staged helper that iterates contour polygons by index, calls an injected/private polygon intersection operation with the target polygon and each contour, and records the contour index when the intersection result is non-empty.
- Preserve upstream control-flow semantics from `PrintApply.cpp:326-333`: visit contours in index order, call `intersection(poly, contours[i])`, insert `i` into a sorted set only for non-empty results, and return that sorted set.
- Keep the actual Clipper polygon/polygon intersection implementation deferred; this milestone stages index-detection semantics only so a later source-cited milestone can port or select the boolean backend.
- Do not implement Clipper `ctIntersection`, fill rules, safety offsets, old/new id comparison, final printable-filament changed result, public API wiring, profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
