# Task 22O.101 architecture decision record

## Status

Accepted for inactive implementation. Decision date: 2026-08-14.

## Decision

Port OrcaSlicer commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`
`src/libslic3r/Arachne/SkeletalTrapezoidation.hpp` and
`SkeletalTrapezoidation.cpp:98-745`, stopping before
`generateTransitioningRibs`, together with only the used indexing and Voronoi
adapter behavior from `Arachne/utils/PolygonsPointIndex.hpp`,
`PolygonsSegmentIndex.hpp`, and `Geometry/VoronoiUtils.cpp`.

The crate-private `ares-core::arachne::trapezoidation` boundary owns source-
ordered polygon segment sites, inside-cell range selection, integer-rounded
Voronoi transfer, point/segment and point/point discretization, pointy-node
separation, and initial central/bead filtering. It extends O100's stable graph
rather than creating another topology. Boost edge/vertex indices are temporary
construction identities; resulting graph identities remain monotonic O100
indices.

## Boundary

O101 is inactive outside focused tests. Transition middle/end generation,
transition filtering and application, extra ribs, segments, beading
propagation, junction connection, `WallToolPaths`, variable-width conversion,
`FillConcentricInternal`, lifecycle, motion, and G-code are deferred to later
source-cited slices. The constructor returns a Rust error for external geometry
or Voronoi construction failure; valid internal topology follows the pinned
source without fallback behavior.
