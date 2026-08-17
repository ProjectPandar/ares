# Spec: KSR FDM Test V4 task196 Arachne wall-line stitching

## Observable contract

Raw variable-width lines are stitched independently per inset. Endpoint candidates within `inner_spacing - 1` are selected by shortest adjusted distance; exact/near endpoints snap without duplication, odd lines may reverse, even lines retain direction, and odd/even lines never join. A chain closes only when it has more than two junctions and its length plus closure exceeds three stitch distances. Closed chains reconnect a near-but-distinct endpoint and set `is_closed`.

A four-edge even rectangle becomes one closed five-junction extrusion line. Focused Arachne tests, formatting, and Clippy remain clean; the stitch implementation and tests live in child modules below 400 LOC.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Arachne/WallToolPaths.cpp:573-669` and `Arachne/utils/PolylineStitcher.hpp:62-225` plus its extrusion-line reversal/connect traits. Outline preprocessing, small-line removal, inner-contour separation, simplification, concentric-internal conversion, cooling, timing, and remaining exact G-code differences are deferred.
