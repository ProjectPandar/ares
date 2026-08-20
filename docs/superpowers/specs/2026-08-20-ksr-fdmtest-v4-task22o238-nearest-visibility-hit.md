# Spec: Task 22O238 nearest seam-visibility ray hit

## Observable contract

Aligned-seam visibility rays must classify occlusion using the nearest triangle intersected in the ray direction. A farther triangle whose bounding box begins before the current nearest hit must not replace that hit.

For KSR slicing, visibility remains derived from the transformed project mesh and deterministic source sampling; no fixture-specific seam coordinates are introduced.

## Upstream boundary

Port the nearest-hit contract used by `OrcaSlicer/src/libslic3r/GCode/SeamPlacer.cpp:177-185` through `AABBTreeIndirect::intersect_ray_first_hit` into `crates/ares-core/src/project_slice/seam_placement/spatial/bvh.rs::first_hit_recursive`.

Included: enforce the current nearest-distance limit at leaf intersections. Deferred: BVH construction parity, mesh decimation, negative-volume ray handling, candidate generation, and later G-code differences.
