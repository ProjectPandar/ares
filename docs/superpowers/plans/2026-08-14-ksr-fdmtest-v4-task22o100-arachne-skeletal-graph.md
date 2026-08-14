# Task 22O.100 implementation plan

## Source boundary

Port OrcaSlicer commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`
`src/libslic3r/Arachne/utils/{HalfEdge,HalfEdgeNode,HalfEdgeGraph}.hpp`,
`src/libslic3r/Arachne/{SkeletalTrapezoidationEdge,
SkeletalTrapezoidationJoint}.hpp`, and
`src/libslic3r/Arachne/SkeletalTrapezoidationGraph.hpp/.cpp` into crate-private
`ares-core::arachne::skeletal`. Include payloads, stable identity/link storage,
source topology queries, insertion, collapse, and removal. Defer the full
`SkeletalTrapezoidation.*` builder, later Arachne stages, project lifecycle,
motion, and G-code. No previous Ares graph scaffold exists.

## Steps

1. Add source-worked tests for payload lifetime, twin/incident traversal,
   upward/tie behavior, stable removals, insertion, and both collapse shapes.
2. Add stable node/edge identities and optional half-edge topology.
3. Port edge/joint payloads and weak shared ownership.
4. Port graph queries, source projection, ribs, node insertion, and small-edge
   collapse with source rewiring order.
5. Run focused Nextest, rustfmt, strict core Clippy, diff, macro, and LOC gates.
6. Keep O100 inactive and record validation evidence before integration.

## Completed evidence

Thirteen source-worked tests pass, including recursive equal-distance ascent,
large-integer endpoint projection, exact/partial/combined collapse branches,
and the source 1,001-rewire cap. Rustfmt, strict all-target core Clippy, diff,
macro, and LOC gates pass; largest production/test shards are 299/294 LOC.
