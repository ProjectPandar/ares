# ARD-0004: Contours before polygon repair

## Status
Accepted for M5.

## Decision
Ares introduces a simple contour-stitching boundary before polygon boolean repair. M5 stitches deterministic segment loops into closed `Contour` values and rejects open or branching graphs rather than silently repairing them.

This keeps the current milestone small and testable while matching the broad libslic3r flow: triangle/plane intersections become loops, then later stages convert loops into repaired polygons, regions, perimeters, and infill.

## OrcaSlicer structure evidence
- `OrcaSlicer/src/libslic3r/TriangleMeshSlicer.cpp` contains loop and `ExPolygon` construction after mesh slicing.
- `OrcaSlicer/src/libslic3r/PrintObjectSlice.cpp` consumes sliced `ExPolygons` before region-specific processing.
- `OrcaSlicer/src/libslic3r/Layer.cpp::make_slices` merges region slices into layer-level slice geometry.

## Consequences
- The core API exposes a stable contour boundary for future polygon repair/perimeter milestones.
- Invalid simple-loop assumptions fail loudly through typed errors.
- Full polygon repair remains an explicit later milestone instead of hidden behavior.
