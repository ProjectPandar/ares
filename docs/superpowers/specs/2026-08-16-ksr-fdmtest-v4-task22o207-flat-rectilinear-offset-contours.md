# Spec: KSR FDM Test V4 task207 flat rectilinear offset contours

## Observable contract

Rectilinear preparation mirrors `ExPolygonWithOffset`: rotate a copy of the source, produce the outer contour set as flat paths, derive the inner contour set by shrinking those exact outer paths, then retain outer paths before inner paths. It must not group outer paths into ExPolygons and re-flatten them between offset stages, because grouping changes path ownership/order and may perturb subsequent clipping at micron boundaries.

The KSR fixture is the differential seam: after E and I/J normalization, task206 first differs at a one-micron bottom-surface arc endpoint. The flat staging keeps that divergence, excluding intermediate ExPolygon regrouping as its cause. Files remain below 400 LOC; rectilinear segment/fill tests, formatting, and Clippy remain clean.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Fill/FillRectilinear.cpp:386-429`, `ExPolygonWithOffset` outer flat offset and inner shrink construction, into `fill::rectilinear::segments::prepare_contours`. Stick/small-area cleanup, arc fitting, exact E, infill counts, timing, and remaining differences are deferred.
