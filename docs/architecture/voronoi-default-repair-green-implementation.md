# Default Voronoi repair GREEN implementation boundary

Follows the [prerequisite spec](voronoi-default-repair-prerequisite-spec.md)
and its RED. This document is an implementation record.

## Source ownership and non-negotiable decisions

`Geometry/Voronoi.cpp:28-73,90-178,185-362` default segment wrapper is ported
outside vendor into private `geometry/voronoi`. Its called
`VoronoiUtils.cpp:39-91,212-254,307-345`, `Point.hpp:236-247`, and
`VoronoiUtilsCgal.cpp:40-138,188-332` remain the owning boundaries. The unused
intersection alternative, point/mixed overloads, downstream closing retry and
Arachne default-consumer migration remain deferred. In particular,
`Arachne/SkeletalTrapezoidation.cpp:378` is NOT included in this landing.

- `mod.rs`: edge/cell/angular detector, states/issues, ordered PI/6,PI/5,PI/7,
  PI/11 attempts, source forward integer rotation and final validity.
- `diagram.rs`: indexed storage ownership and actual effective vertex view.
  Rust backend topology is already indexed and owned: MOVE it rather than
  copying/rebasing C++ pointers. Preserve every cell/edge bit, order, anchor and
  index. A modified diagram separately owns coordinates; an unmodified diagram
  does not allocate/copy coordinates. No implicit raw Diagram coercion: all real
  MedialAxis consumers have the private effective Diagram type. The vertex view
  borrows backend metadata; only test serialization reads its color directly.
  Vertex and
  edge_as_line return the EFFECTIVE coordinates, not retained backend positions.
- `cell_range.rs`: exact source incident/next traversal, last possible start,
  before/after-start endpoint selection, finite edges and llround equivalence.
- `rotation.rs`: source endpoint index/direction encoding, first nonzero mark
  ownership in cell order, inverse rotation only for unmarked vertices, direct
  original endpoint assignment otherwise, mark clearing for every vertex.
- `detect.rs`: finite-edge/parabola-focus issues precede cell-range/intersection
  issues, which precede angular planarity. Source cross-product expression is
  unchanged; no EPS/tolerance or substitute detector.
- `planarity.rs` and `predicates/*`: original linear/parabolic orientation,
  tangent polynomial, cyclic three-edge CCW tests. The source CGAL filtered
  kernel dependency is expressed as portable outward-rounded binary intervals
  with exact signed dyadic arithmetic when uncertain. Interval uncertainty is
  NEVER a geometry tolerance or an accepted sign. Exact MP_Float arithmetic
  uses finite-double integer significands and binary exponents, including
  cancellation, overflow/underflow of the fast filter and collinearity.
  No CGAL/C++ FFI, fenv mutation, native dependency or backend modifications.

The real `geometry/medial_axis/diagram.rs::build` now calls this private default
wrapper and checks its validity before existing topology validation. It cannot
silently return raw on failed repair. Existing Ares raw-only construction is
replaced, not retained as a compatibility fallback. Annotation/chaining/validation
algorithms are NOT redesigned: their diagram type and vertex access follow the
actual effective source diagram. This is not an Ares-owned pipeline milestone.

## Native RED and numeric closure

The Native RED existed before production code. Correctness is qualified by the
runtime Native E2E comparison, not post-hoc unit tests.

The prior bounded raw-rebuild diagnostic measured52 vertex-bit differences over
42 of102 raw rotation-stage diagrams; cells/edges matched throughout. Source
endpoint mapping was ported before declaring a backend blocker. The source candidate postprocess absorbed all52, marks cleared, and98 actual
successful LAST attempts matched COMPLETE ordered Native FINAL diagrams in both
i64 and i32. Four failed first attempts are preserved and NOT counted PASS.
This isolated closure used observed Native angles/sites only as postprocess input;
it is NOT the production detector/dispatcher or real-constructor GREEN evidence.
Any FINAL vertex-bit discrepancy
from the real constructor must STOP, with no vendor fix or tolerance.

Test serialization is adapted to a TEST-only generic snapshot view so both the
old raw incident test and real default consumer serialize their ACTUAL vertices.
The full cell/category/anchor/color, vertex bits/incident/color and edge topology/
flags schema stays intact, with no geometry sorting, renumbering or field ignore.
`DiagramSnapshot` and the default-repair test's implementation are mechanical
compile adaptations to the private effective Diagram type. For edge v1,
`edge_get_vertex1` and direct `twin.vertex0` are equivalent: vertex1 is defined
as the twin's vertex0 in Boost, and the unchanged Rust backend accessor follows
that definition. All 300 complete serialized diagrams remain strictly equal;
the adaptation changes neither schema nor expected values.

## Qualification gates

GREEN requires the real constructor RED and targeted consumer regressions to
pass. Qualification MUST run BOTH
`runtime_native_default_repair_complete_ordered_diagrams_match` (core unit test)
and `runtime_native_rectangle_complete_ordered_diagram_matches` (incident) in
BOTH reference modes with COMPLETE debug/source/resources/GDB environment and
NO SKIP. The orca_parity-only test run does not cover that core test.
