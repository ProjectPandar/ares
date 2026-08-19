# Spec: KSR FDM Test V4 task22o225 compensation PolyTree ordering

## Observable contract

The first-layer compensated slice keeps deterministic PolyTree sibling order before surface-type clipping. Compensation inputs and union outputs use the same order, so later region slicing receives stable outer and hole sibling traversal; fixture identity and reference G-code are not production inputs.

The ordering is geometric and applies to every compensated ExPolygon: outer siblings follow the source union's descending positive contour-area order, while same-parent holes follow the source scan order by first point (descending Y, then ascending X). No coordinate, fixture name, digest, or known G-code constant enters production code.

## Upstream boundary

This slice ports the ordering retained by OrcaSlicer 2.4.2 `src/libslic3r/PrintObjectSlice.cpp:1274-1292` around `elephant_foot_compensation` plus `union_ex`, `src/libslic3r/ClipperUtils.cpp:169-204` `PolyTreeToExPolygons`, and the resulting subject order consumed by `src/libslic3r/LayerRegion.cpp:73-79` `intersection_ex(this_surfaces, this->fill_expolygons)`.

Included: compensated single-region ExPolygon and hole sibling order before downstream clipping. Deferred: later rectilinear geometry, travel/wipe ordering, topology, cooling, timing/M73, and remaining G-code parity.
