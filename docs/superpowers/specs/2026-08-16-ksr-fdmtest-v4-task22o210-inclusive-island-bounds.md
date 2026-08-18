# Spec: KSR FDM Test V4 task210 inclusive island bounds prefilter

## Observable contract

Extrusion-island assignment treats points on every contour edge as candidates for that island. The bounding-box prefilter is only an acceleration structure and must not reject points on maximum X or Y before `Polygon::contains`, whose source-compatible boundary result is inside. Nested islands remain tested smallest area first.

A focused nested-boundary test places a thin-fill endpoint on the inner island's maximum X and requires selection of that inner island rather than an enclosing island. Fixture gap blocks and first differing layer order remain unchanged, excluding this prefilter as their cause. Files remain below 400 LOC; island/fill tests, formatting, and Clippy remain clean.

## Upstream boundary

This slice deepens the Ares association seam needed to preserve OrcaSlicer `LayerRegion` fill ownership from `Fill/Fill.cpp:1234-1368`; it does not invent geometry. The current Ares spatial compatibility shell replaces source-owned entity association and must be boundary-inclusive. Medial topology, arc input geometry, exact E, timing, and remaining differences are deferred.
