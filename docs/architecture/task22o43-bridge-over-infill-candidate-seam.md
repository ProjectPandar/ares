# Task 22O.43 architecture decision record

## Status

Accepted.

## Decision

The next `prepare_infill` seam is an owned internal-bridge candidate inventory
after `PreparedPostExternalSurfaces`.

The pinned `clip_fill_surfaces` call is documented as an identity operation,
not represented by a shallow Rust lifecycle type. The following
`bridge_over_infill` candidate section is represented by one deep in-process
module: callers supply the already prepared project graph, and the module
returns candidates owned by a successor. Candidate identity is stable indices
`(object, layer, region, surface)`, never a raw pointer or a reference into a
surface vector.

## Rationale

The polygon morphology, policy gates, area thresholds, scaling rules, and
error order belong behind one interface and have only in-process dependencies.
Retaining the result gives later bridge-angle and commit slices a useful seam;
computing and discarding it would be shallow. Stable indices preserve Orca's
surface association across the later destructive rewrite without a defensive
clone of the project graph.

An identity lifecycle wrapper for `clip_fill_surfaces` would add interface
without behavior. A whole-function `bridge_over_infill` port would prematurely
couple candidate discovery to missing exact CrossHatch anchor generation,
Lightning state, AABB line queries, and final mutation. Reusing Ares' legacy
CrossHatch scaffold would violate source parity.

## Consequences

- O43 can activate and retain useful state while public slicing continues to
  return `ProjectSlicingIncomplete`.
- Geometry failures dispose the owned O42 predecessor; read-only discovery
  requires no rollback copy.
- The current one-compatible-region graph remains explicit. Candidate identity
  carries the region index, but O43 adds no speculative multi-region adapter.
- Orca's TBB scheduling, compile-disabled timing instrumentation, separate
  caller cancellation checks, logging, and debug SVG output remain deferred
  host concerns. The platform-neutral core preserves the candidate semantics
  and failure boundary sequentially.
- Exact anchor generation and final mutation remain separate source-cited
  milestones behind the same `bridge_over_infill` module.
