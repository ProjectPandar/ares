# Task 22O.101 — Arachne skeletal trapezoidation builder

Port OrcaSlicer commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`
`Arachne/SkeletalTrapezoidation.hpp` and
`SkeletalTrapezoidation.cpp:98-745`, through both
`filterNoncentralRegions` overloads and stopping before
`generateTransitioningRibs`. Include only used behavior from
`Arachne/utils/PolygonsPointIndex.hpp`, `PolygonsSegmentIndex.hpp`, and
`Geometry/VoronoiUtils.cpp`.

Requirements:

- preserve polygon/point/segment site order and source-category decoding;
- preserve point- and segment-cell range selection and polygon-corner tests;
- preserve Voronoi coordinate rounding, half-edge allocation/link order, twin
  reconstruction, ribs, pointy start-node separation, and small-edge collapse;
- preserve parabolic and point/point discretization arithmetic and markings;
- preserve central-angle arithmetic, both recursive filters, outer filtering,
  local-maximum bead counts, and beading strategy calls;
- reuse O100 stable graph and O99 strategies with ordinary Rust modules below
  400 LOC;
- cover rectangles, holes, both curve categories, pointy splitting,
  equidistant central recursion, noncentral dissolution, and both coordinate
  scales in separate source-worked tests;
- keep the boundary crate-private and inactive.

Deferred: all behavior beginning with `generateTransitioningRibs`, later
`SkeletalTrapezoidation.cpp` stages, `WallToolPaths`, variable-width entities,
`FillConcentricInternal`, lifecycle, motion, and G-code. No alternate builder or
fixture-specific branch is permitted.
