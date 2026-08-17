# Spec: Task 22O.160 retained fill arc-fitting payload

## Observable contract

When `enable_arc_fitting` is loaded from the project, arc fitting performed during fill-path simplification remains attached to the simplified fill path and is consumed by G-code emission. The KSR first bottom-surface contour therefore emits `G2 X105.847 Y89.053 I1.094 J1.245 E.01717` rather than replacing the retained source arc with the straight chord `G1 X105.847 Y89.053 E.01711`.

The fitting payload is geometry-derived and travels with each fill path; production code does not identify the fixture or infer arcs from rounded G-code coordinates.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `src/libslic3r/LayerRegion.cpp:1074-1084`, `src/libslic3r/ExtrusionEntity.cpp:41-47`, `src/libslic3r/Polyline.cpp:152-156`, `src/libslic3r/ArcFitter.cpp:97-150`, and `src/libslic3r/GCode.cpp:6990-7109`. `FillExtrusionPath` retains the `Polyline::fitting_result` equivalent produced during simplification, including reversal-safe indices and arc direction, and the G-code emitter consumes that retained payload.

Sub-micron extrusion differences, clipping-library contour parity, cooling, timing, and later exact G-code differences are deferred.
