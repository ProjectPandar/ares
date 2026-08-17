# Spec: KSR FDM Test V4 task198 Arachne inner-contour separation

## Observable contract

After wall-line removal, zero-width inset groups are excluded from printable wall toolpaths. Closed even zero-width lines become inner-contour polygons; odd or open zero-width lines do not. Empty inset groups are excluded. Extracted polygons are normalized by an even-odd union so nested contours have usable winding-independent fill semantics.

The wall-toolpath interface returns printable lines and the separated inner contour together. Focused tests cover zero-width extraction and positive-width retention. Files remain below 400 LOC; geometry/wall tests, formatting, and Clippy remain clean.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Arachne/WallToolPaths.cpp:728-777`, `separateOutInnerContour`, and exposes the existing Clipper path executor with the source even-odd fill rule. Toolpath simplification, concentric-internal extrusion conversion, cooling, timing, and remaining exact G-code differences are deferred.
