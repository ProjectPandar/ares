# Spec: KSR FDM Test V4 task201 concentric loop finalization

## Observable contract

Each closed Arachne concentric line starts at the stored point nearest the source origin, then every generated thick polyline is clipped from its end by the option-derived seam gap. Invalid clipped paths are discarded. The surviving paths are ordered by the source shortest-traverse endpoint chain and may reverse when that lowers travel.

`ThickPolyline` end clipping preserves source interpolation and truncation semantics. A focused test covers partial-segment clipping and width retention; concentric tests cover closed-loop rotation and deterministic shortest traversal. Files remain below 400 LOC; focused fill/geometry/shortest-path tests, formatting, and Clippy remain clean.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Fill/FillConcentricInternal.cpp:48-82`, `Polyline.cpp:51-82`, and the reached `ShortestPath.hpp:32-51` endpoint-chain call. Cooling, timing, and remaining exact G-code differences are deferred.
