# Spec: KSR FDM Test V4 task199 Arachne variable-width simplification

## Observable contract

Printable Arachne lines are simplified after short-line removal and before zero-width contour extraction. Endpoints remain stable; closed lines retain their duplicate closure endpoint and at least three junctions. A junction is removed only when the source segment-length, accumulated-area height, and extrusion-area deviation limits permit it. The source 5-micron collinearity branch and long-successor intersection adjustment are retained.

Focused tests cover collinear open-line removal and endpoint preservation. The wall pipeline uses the source default maximum resolution, deviation, and extrusion-area deviation at its coordinate scale. Files remain below 400 LOC; focused wall tests, formatting, and Clippy remain clean.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Arachne/WallToolPaths.cpp:702-719` and activates the existing Rust rewrite of `Arachne/utils/ExtrusionLine.cpp:43-256` at that source pipeline position. Concentric-internal extrusion conversion, cooling, timing, and remaining exact G-code differences are deferred.
