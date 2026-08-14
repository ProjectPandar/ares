# Task 22O.100 — Arachne skeletal graph

Port OrcaSlicer commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`
`Arachne/utils/{HalfEdge,HalfEdgeNode,HalfEdgeGraph}.hpp` and
`Arachne/{SkeletalTrapezoidationEdge,SkeletalTrapezoidationJoint}.hpp`, plus
`Arachne/SkeletalTrapezoidationGraph.hpp/.cpp`, into crate-private
`ares-core::arachne::skeletal`.

Requirements:

- preserve graph identity and twin/next/prev/from/to topology with safe stable
  indices;
- preserve edge/joint payload defaults, central state, transition/end/junction
  weak shared storage, and propagated beading state;
- preserve upward, local-maximum, multi-intersection, distance, and unconnected
  traversal behavior;
- preserve source rib/node insertion and small-edge middle/side collapse
  rewiring and removals;
- leave removed slots vacant so active identities never alias or get reused;
- use ordinary Rust modules under 400 LOC without include/source macros;
- keep source-worked tests separate and do not activate project slicing.

Deferred: the `SkeletalTrapezoidation.*` builder and later filtering/transition
stages, `WallToolPaths`, variable-width entities, `FillConcentricInternal`,
lifecycle, motion, and G-code. No earlier Ares skeletal graph scaffold is
retained or wrapped.
