# Spec: KSR FDM Test V4 task211 preserve classic surface ownership

## Observable contract

Classic perimeter collections and gap-fill entities retain their originating perimeter surface index through layer-region materialization and fill staging. During extrusion-island assembly, perimeters establish source-surface-to-island candidates; owned gap fills choose the nearest candidate. When one source surface spans disconnected components, all perimeter components participate rather than collapsing ownership to the first. Spatial containment remains only when no perimeter candidate exists.

A real-fixture layer test previously observes all 120 layer-two thin entities in one island; they now occupy multiple islands. Fixture gap feature blocks increase from 206 to 211 versus reference 470 and first-differing-layer interleaving moves toward the golden. Files remain below 400 LOC; ownership/island tests, formatting, and Clippy remain clean.

## Upstream boundary

This slice restores ownership already explicit in OrcaSlicer 2.4.2 `PerimeterGenerator.cpp:1573-1624` and `Fill/Fill.cpp:1360-1368`, where gap fills remain inside their source `LayerRegion`/surface collections. The Ares flattening compatibility shell had discarded `PreparedGapExtrusionSurface::source_index`. Geometry generation, arc input geometry, exact E, timing, and remaining differences are deferred.
