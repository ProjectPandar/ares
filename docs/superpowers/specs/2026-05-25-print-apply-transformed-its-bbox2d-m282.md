# PrintApply transformed indexed-triangle bbox2d Spec

## Goal

Port OrcaSlicer's private `transformed_its_bbox2d(...)` helper into `ares-core` as staged private indexed-triangle and bounding-box helpers for later print-object-region invalidation milestones.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:582-593`: non-empty assertion, bbox initialization from transformed first vertex, extension over all triangle vertices, and offset/EPSILON bbox expansion.

Required context:
- `OrcaSlicer/deps_src/admesh/stl.h:42-44`: `stl_vertex` is 3-float vector; `stl_triangle_vertex_indices` is 3-int vector.
- `OrcaSlicer/deps_src/admesh/stl.h:219-235`: `indexed_triangle_set` owns `indices` and `vertices` vectors.
- `OrcaSlicer/src/libslic3r/Print.hpp:216-223`: `PrintObjectRegions::BoundingBox` is `Eigen::AlignedBox<float, 3>` for transformed ModelVolume bounds.
- `OrcaSlicer/src/libslic3r/Point.hpp:84`: `Transform3f` is a float 3D affine transform.
- `OrcaSlicer/src/libslic3r/libslic3r.h:48-52`: Orca `EPSILON = 1e-4`.

## Requirements

- Extend private `ares-core` PrintApply staged implementation; do not add public APIs.
- Add staged `StagedIndexedTriangleSet` with `vertices: Vec<[f32; 3]>` and `indices: Vec<[usize; 3]>` or equivalent private structure.
- Add staged `StagedBoundingBox3f` with min/max `[f32; 3]` accessors for tests and later staged helpers.
- Add a private helper equivalent to `transformed_its_bbox2d(its, transform, offset)`.
- The helper must panic for empty `indices`, matching the upstream internal assertion.
- The helper must initialize the bbox from `transform * vertices[indices[0][0]]`.
- The helper must iterate all triangle indices and all three vertex positions, transform each vertex, and extend min/max.
- The helper must subtract `[offset, offset, EPSILON]` from min and add `[offset, offset, EPSILON]` to max after all vertices are processed.
- The helper must use f32 transform, point, and bbox values for this staged boundary.
- Add unit tests for single-triangle bbox expansion, multi-triangle extension, transform application, offset/EPSILON expansion, and empty-index panic.
- Do not implement full mesh import/storage, face properties, robust index validation beyond internal trusted fixtures, z-range clipping, volume filtering, print-object-region invalidation, public APIs, profile loading, UI runtime behavior, slicing, extrusion, G-code generation, new crates, new dependencies, or independent Ares pipeline behavior.
