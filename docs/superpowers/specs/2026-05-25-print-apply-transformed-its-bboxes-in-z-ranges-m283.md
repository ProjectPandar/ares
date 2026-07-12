# PrintApply transformed indexed-triangle bboxes in Z ranges Spec

## Goal

Port OrcaSlicer's private `transformed_its_bboxes_in_z_ranges(...)` helper into `ares-core` as a staged private helper for later print-object-region invalidation milestones.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:595-661`: output vector assignment, transformed triangle vertices, per-range edge clipping, bbox initialization/extension, and final offset/EPSILON bbox expansion.

Required context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:582-593`: predecessor `transformed_its_bbox2d(...)` bbox expansion semantics staged by M282.
- `OrcaSlicer/deps_src/admesh/stl.h:42-44`: `stl_vertex` is 3-float vector; `stl_triangle_vertex_indices` is 3-int vector.
- `OrcaSlicer/deps_src/admesh/stl.h:219-235`: `indexed_triangle_set` owns `indices` and `vertices` vectors.
- `OrcaSlicer/src/libslic3r/Slicing.hpp:150`: `t_layer_height_range` is `std::pair<coordf_t, coordf_t>`.
- `OrcaSlicer/src/libslic3r/Print.hpp:216-223`: `PrintObjectRegions::BoundingBox` is `Eigen::AlignedBox<float, 3>` for transformed ModelVolume bounds.
- `OrcaSlicer/src/libslic3r/Point.hpp:84` and `Point.hpp:136-144`: `Transform3f`, `Vec2f`, `Vec3f`, `to_2d`, and `to_3d` behavior used for XY interpolation at clipped Z values.
- `OrcaSlicer/src/libslic3r/libslic3r.h:46-52`: `coordf_t = double` and Orca `EPSILON = 1e-4`.

## Requirements

- Extend private `ares-core` PrintApply staged mesh implementation; do not add public APIs.
- Add a staged Z range representation equivalent to `t_layer_height_range` using f64 bounds matching `coordf_t`, converted to f32 only for generated clipped points.
- Add a private helper equivalent to `transformed_its_bboxes_in_z_ranges(its, transform, z_ranges, offset)`.
- The helper must return/assign one `(bbox, populated)` result per input Z range, initialized as unpopulated before triangle processing.
- For each triangle, the helper must compute the three transformed f32 points once before iterating ranges.
- For each range and each triangle edge, the helper must follow upstream edge order `iprev = 2`, `iedge = 0,1,2` and sort edge endpoints by Z.
- The helper must skip edges fully outside a slab when `p2.z <= range.start || p1.z >= range.end`.
- If `p1.z < range.start` and `p2.z > range.end`, the helper must add two interpolated XY points at `range.start` and `range.end`.
- If `p1.z < range.start` and `p2.z <= range.end`, the helper must add the lower-bound intersection and `p2`.
- If `p1.z >= range.start` and `p2.z > range.end`, the helper must add the upper-bound intersection and `p1`.
- Otherwise, the helper must add both in-range endpoints.
- Bbox extension must initialize min/max from the first added point for a range and extend after that.
- After all triangles/ranges, the helper must expand every bbox by subtracting `[offset, offset, EPSILON]` from min and adding `[offset, offset, EPSILON]` to max, preserving populated flags.
- Add unit tests for empty Z ranges, no triangle/range intersection, fully inside triangle, lower-bound crossing, upper-bound crossing, two-intersection spanning edge, multiple ranges, transform application, and offset/EPSILON expansion.
- Do not implement full mesh import/storage, face properties, print-object-region invalidation, volume filtering, public APIs, profile loading, UI runtime behavior, slicing, extrusion, G-code generation, new crates, new dependencies, or independent Ares pipeline behavior.
