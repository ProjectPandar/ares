# Spec: KSR FDM Test V4 task200 Arachne concentric-internal entities

## Observable contract

`ConcentricInternal` fill consumes each no-overlap ExPolygon with the source full flow spacing. It derives the loop count from the contour bounds, builds wall parameters from the minimum configured nozzle diameter and source constants, invokes the completed Arachne wall-toolpath pipeline, and converts retained variable-width lines into extrusion entities carrying the surface fill role and flow.

No fixture identity or reference G-code is consulted. A focused test observes multiple positive-width variable-flow entities for a rectangular internal surface. Existing classic gap conversion remains role-compatible. Files remain below 400 LOC; focused fill/Arachne tests, formatting, and Clippy remain clean.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Fill/FillConcentricInternal.cpp:12-46,84-97` and connects it to the Rust `Arachne/WallToolPaths.cpp` rewrite. Loop rotation, seam-gap clipping, and shortest-traverse reordering from lines 48-82 remain deferred to the next source-cited slice, along with cooling, timing, and remaining exact G-code differences.
