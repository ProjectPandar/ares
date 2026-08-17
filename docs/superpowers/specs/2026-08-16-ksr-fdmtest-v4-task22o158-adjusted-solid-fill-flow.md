# Spec: Task 22O.158 adjusted solid-fill flow

## Observable contract

When a full-density monotonic surface adjusts scanline spacing to fit its bounding width, emitted extrusion paths use the corresponding adjusted `Flow`: width changes by the spacing delta and volumetric extrusion is recomputed from that width. The processor emits the adjusted line-width marker before the KSR first bottom-surface path.

All inputs remain project-derived: initial-layer width, layer height, nozzle diameter, surface density, and generated surface bounds. No fixture identity or expected G-code constant enters production code.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `src/libslic3r/Fill/FillRectilinear.cpp:2814-2821`, `src/libslic3r/Fill/FillBase.cpp:146-181`, and `src/libslic3r/Flow.cpp:146-163`. The monotonic fill result exposes its actual spacing and entity conversion applies `Flow::with_spacing` semantics.

Extrusion formatting precision, cooling, timing, and later exact G-code differences are deferred.
