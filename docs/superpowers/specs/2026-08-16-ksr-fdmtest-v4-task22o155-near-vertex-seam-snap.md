# Spec: Task 22O.155 near-vertex seam snapping

## Observable contract

An aligned seam projection within 1.5 microns of an existing loop vertex reuses that vertex instead of inserting a near-duplicate point. The KSR outer-loop wipe consequently retracts in one move, `G1 X135.839 Y100.618 E-.4`; the artificial preliminary move to `X136.839 Y100.592` is absent.

The decision is geometric and scale-aware. It is derived from generated extrusion-loop vertices and the selected seam; production code must not inspect fixture coordinates or reference G-code.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `src/libslic3r/GCode/SeamPlacer.cpp:1622-1628` and `src/libslic3r/ExtrusionEntity.cpp:182-225`: `place_seam` first calls `split_at_vertex` with `scaled<double>(0.0015)` so G-code-resolution near-duplicate segments are not created, then inserts a projected point only when no existing vertex is within that radius.

Cooling, timing, object identifiers, spiral-lift geometry, and later exact G-code differences are deferred.
