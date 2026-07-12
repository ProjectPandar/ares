# M268: PrintApply all-extruder intersection append

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the all-extruder intersection and first-result append branch inside `is_printable_filament_changed(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:323-324`, with intersection backend context from `ClipperUtils.hpp:496-508` and `ClipperUtils.cpp:702-703`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned print pipeline.

## Exit criteria

- Add a private `ares-core` staged helper that calls an injected/private intersection operation once with `{printable_poly}` as subject and all scaled extruder polygons as clip polygons.
- Preserve upstream control-flow semantics from `PrintApply.cpp:323-324`: skip empty intersection results; append only `all_extruder_polys[0]` to existing split polygons when non-empty; preserve existing split polygon order before the append.
- Keep the actual Clipper intersection implementation deferred; this milestone stages call/append semantics only so a later source-cited milestone can port or select the boolean backend.
- Do not implement Clipper `ctIntersection`, fill rules, safety offsets, intersection-id comparison, final printable-filament changed result, public API wiring, profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
