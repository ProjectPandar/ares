# M267: PrintApply extruder diff first-result collection

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the per-extruder difference loop inside `is_printable_filament_changed(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:317-320`, with Clipper difference context from `ClipperUtils.hpp:429-433` and `ClipperUtils.cpp:676-679`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned print pipeline.

## Exit criteria

- Add a private `ares-core` staged helper that iterates scaled extruder polygons in source order, calls an injected/private difference operation with the scaled printable polygon and current extruder polygon, and appends only the first result polygon when the difference result is non-empty.
- Preserve upstream control-flow semantics from `PrintApply.cpp:317-320`: one `diff(printable_poly, poly)` call per extruder polygon, skip empty results, append `res[0]` for non-empty results, and preserve append order.
- Keep the actual Clipper difference implementation deferred; this milestone stages call/collection semantics only so a later source-cited milestone can port or select the boolean backend.
- Do not implement Clipper `ctDifference`, fill rules, safety offsets, full split polygon assembly, all-extruder intersection, intersection-id comparison, public API wiring, profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
