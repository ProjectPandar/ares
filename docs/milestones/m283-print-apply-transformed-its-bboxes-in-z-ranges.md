# M283: PrintApply transformed indexed-triangle bboxes in Z ranges

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `transformed_its_bboxes_in_z_ranges(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:595-661`, with predecessor bbox helper context from `PrintApply.cpp:582-593`, indexed-triangle context from `OrcaSlicer/deps_src/admesh/stl.h:42-44` and `stl.h:219-235`, `t_layer_height_range` context from `OrcaSlicer/src/libslic3r/Slicing.hpp:150`, `PrintObjectRegions::BoundingBox` context from `OrcaSlicer/src/libslic3r/Print.hpp:216-223`, `Transform3f` / `Vec2f` / `Vec3f` and `to_2d` / `to_3d` context from `OrcaSlicer/src/libslic3r/Point.hpp:84` and `Point.hpp:136-144`, and `coordf_t` / `EPSILON` context from `OrcaSlicer/src/libslic3r/libslic3r.h:46-52`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned print pipeline.

## Exit criteria

- Add a private staged helper equivalent to `transformed_its_bboxes_in_z_ranges(...)` over the existing staged indexed-triangle set, f32 transform, Z ranges, and bbox state.
- Preserve assigning one `(BoundingBox, false)` output entry per input Z range before processing.
- Preserve transforming each triangle's three vertices once before checking all Z ranges.
- Preserve per-range bbox initialization on first extension and extension on subsequent points.
- Preserve edge traversal order with `iprev = 2`, then `iedge = 0..2`, sorting edge endpoints by Z.
- Preserve slab skip behavior for edges fully outside `p2.z <= range.first || p1.z >= range.second`.
- Preserve lower-bound crossing behavior, including two-intersection handling when an edge spans both range bounds.
- Preserve upper-bound crossing behavior and inside-edge behavior.
- Preserve final expansion of every bbox by `[offset, offset, EPSILON]`, with bool flags identifying populated boxes.
- Add tests for empty Z ranges, no triangle/range intersection, fully inside triangle, lower-bound crossing, upper-bound crossing, two-intersection spanning edge, multiple ranges, transform application, and offset/EPSILON expansion.
- Defer full mesh import/storage, face properties, print-object-region invalidation, volume filtering, public APIs, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
