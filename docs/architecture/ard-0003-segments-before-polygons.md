# ARD-0003: Slice segments before polygons

## Status
Accepted for M4.

## Decision
Ares ports mesh slicing in two steps: first deterministic triangle-plane line segments, then later contour stitching and polygon repair. M4 introduces public segment-level geometry in `ares-core` and emits segment metadata through the existing byte-in/byte-out `slice` API.

The implementation keeps coplanar triangle filled-area handling out of scope and explicitly ignores whole coplanar triangles. This avoids inventing polygon semantics before the contour-stitching milestone.

## OrcaSlicer structure evidence
- `OrcaSlicer/src/libslic3r/PrintObjectSlice.cpp` performs volume slicing after layer creation.
- `OrcaSlicer/src/libslic3r/TriangleMeshSlicer.*` is the libslic3r area that converts triangle meshes at Z values into 2D slice geometry.
- Later `LayerRegion` and perimeter/infill paths consume repaired polygonal slices, which Ares defers until segment output is stable.

## Consequences
- `slice` output starts reflecting XY mesh geometry without yet claiming printable extrusion paths.
- Segment geometry gives future milestones a tested boundary for contour stitching and polygon repair.
- Core remains platform-neutral and free of filesystem dependencies.
