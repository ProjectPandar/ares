# Task 22O.100 architecture decision record

## Status

Accepted for implementation. Decision date: 2026-08-14.

## Decision

Port OrcaSlicer commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `src/libslic3r/Arachne/utils/HalfEdge.hpp`;
- `src/libslic3r/Arachne/utils/HalfEdgeNode.hpp`;
- `src/libslic3r/Arachne/utils/HalfEdgeGraph.hpp`;
- `src/libslic3r/Arachne/SkeletalTrapezoidationEdge.hpp`;
- `src/libslic3r/Arachne/SkeletalTrapezoidationJoint.hpp`;
- `src/libslic3r/Arachne/SkeletalTrapezoidationGraph.hpp/.cpp`.

The crate-private `ares-core::arachne::skeletal` boundary owns stable node and
edge identities, optional half-edge links, source payloads and weak shared
storage, topology queries, rib/node insertion, and source small-edge collapse.
`Vec<Option<_>>` arenas replace C++ stable-list addresses without pointer
aliasing or identity reuse. Link absence is represented exhaustively with
`Option`; active identities remain stable across removals. Transition payload
vectors preserve source order and weak ownership only; later transition
filtering that relies on C++ stable list iterators must introduce explicit safe
element identities rather than retaining `Vec` references.

## Boundary

O100 is inactive outside focused tests. The full `SkeletalTrapezoidation.*`
Voronoi-to-graph builder, central filtering, transition generation and
propagation, `WallToolPaths`, variable-width entity conversion,
`FillConcentricInternal`, lifecycle, motion, and G-code are deferred. No prior
Ares skeletal graph scaffold exists; O100 extends the inactive O98/O99 source
prerequisites without a fallback or alternate graph algorithm.

Thirteen source-worked tests pass. Rustfmt, strict all-target core Clippy, diff,
macro, and LOC gates pass; largest production/test shards are 299/294 LOC.
