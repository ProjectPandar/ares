# Spec: Task 220.128 global seam visibility

## Observable contract

For project objects whose `seam_position` is `aligned`, Ares derives candidate visibility from the transformed 3MF model mesh and uses the visibility-aware OrcaSlicer seam comparator when choosing each layer perimeter's initial seam. The result must depend only on imported mesh geometry, transforms, typed options, and generated perimeter geometry.

A clear outward hemisphere has visibility `1`; rays occluded by another outward-facing model surface lower visibility. Sampling is deterministic across runs and supported on WASM, Windows, macOS, and Linux.

## Upstream boundary

Port OrcaSlicer 2.4.2 `src/libslic3r/TriangleSetSampling.cpp:9-68` and `src/libslic3r/GCode/SeamPlacer.cpp:40-227,405-529,624-705,742-797,916-928,1500-1628`. This slice includes transformed model-part mesh gathering, deterministic uniform triangle sampling, AABB-accelerated hemisphere ray casting, nearby-sample interpolation, visibility-aware initial candidate choice, nearest-perimeter association, and initial loop splitting. Negative volumes, seam enforcers/blockers, cross-layer spline alignment, and exact aligned final positions remain deferred source-cited slices.
