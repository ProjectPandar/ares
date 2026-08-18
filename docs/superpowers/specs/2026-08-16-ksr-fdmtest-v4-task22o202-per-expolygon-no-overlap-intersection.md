# Spec: KSR FDM Test V4 task202 per-expolygon no-overlap intersection

## Observable contract

Concentric-internal generation processes each narrow `SurfaceFill.expolygons` member independently. Before Arachne generation, its domain is the intersection of the grouped no-overlap polygons with that one fill ExPolygon. A partial narrow split therefore cannot regenerate the entire parent solid region, duplicate extrusion over the normal fill, or multiply complex full-region paths.

A focused test intersects a small fill polygon against a larger no-overlap domain and observes only the source ten-unit safety expansion of the small domain. No debug instrumentation remains. Files remain below 400 LOC; focused fill/Clipper tests, formatting, and Clippy remain clean.

## Upstream boundary

This slice ports the per-ExPolygon preparation loop at OrcaSlicer 2.4.2 `Fill/Fill.cpp:1326-1329`, which assigns `f->no_overlap_expolygons = intersection_ex(surface_fill.no_overlap_expolygons, {expoly}, ApplySafetyOffset::Yes)` before calling `FillConcentricInternal.cpp`. The existing Ares scaffold incorrectly passed the full grouped no-overlap set to every split narrow fill. The fixture differential exposed the next missing source boundary: `WallToolPaths.cpp:487-509` outline preparation, without which a reached repeated terminal vertex trips the skeletal transition invariant. That preprocessing, cooling, timing, and remaining G-code differences are deferred.
