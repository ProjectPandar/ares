# M282: PrintApply transformed indexed-triangle bbox2d

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `transformed_its_bbox2d(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:582-593`, with `stl_vertex` / `stl_triangle_vertex_indices` / `indexed_triangle_set` context from `OrcaSlicer/deps_src/admesh/stl.h:42-44` and `stl.h:219-235`, `PrintObjectRegions::BoundingBox` context from `OrcaSlicer/src/libslic3r/Print.hpp:216-223`, `Transform3f` context from `OrcaSlicer/src/libslic3r/Point.hpp:84`, and `EPSILON` context from `OrcaSlicer/src/libslic3r/libslic3r.h:48-52`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned print pipeline.

## Exit criteria

- Add private staged indexed-triangle-set and bounding-box helpers sufficient for `transformed_its_bbox2d(...)`.
- Preserve the upstream non-empty triangle assertion behavior as an internal panic for empty indices.
- Preserve initializing the bbox from the first triangle's first transformed vertex.
- Preserve extending the bbox with every transformed vertex of every indexed triangle in source order.
- Preserve applying `min -= Vec3f(offset, offset, EPSILON)` and `max += Vec3f(offset, offset, EPSILON)` after extension.
- Preserve f32 transform and bbox storage semantics for this staged boundary.
- Add tests for single-triangle bbox expansion, multi-triangle extension, transform application, offset/EPSILON expansion, and empty-index panic.
- Defer full mesh import/storage, face properties, robust index validation beyond test fixtures, z-range clipping, print-object-region invalidation, public APIs, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
