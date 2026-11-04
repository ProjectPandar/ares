# Classic small-loop exact chaining

Bounded public qualification only. Implementation depends on the shared
geometry exact solver; do not duplicate the dynamic program.

## Owning upstream boundary

OrcaSlicer beta `345efec64c7b95a05ce5cf96df0dde66a158835c`:

- `ShortestPath.cpp:103-137::chain_segments_exact`: subset states include the
  final segment and allowed orientation; optional starting point and free end;
  transition and final selection use strict `<`, preserving source tie order.
- `:164-176::chain_segments_greedy_constrained_reversals_`: dispatch two through
  four segments to that exact solver. Empty/single cases and larger greedy
  chains are outside the changed dispatch.
- `:1429-1475::chain_extrusion_entities_impl/chain_extrusion_entities` uses the
  same dispatch without enabling the separate improvement pass.
- `PerimeterGenerator.cpp:280-315::traverse_loops` chains each sibling level
  from explicit local `(0,0)` before descending into children and emitting them.

Rust destinations: `project_slice/perimeters/classic/shortest_path/chain.rs`
currently goes straight from the 0/1 special cases to the KD/multifragment path;
`materialize/tree.rs::level_chain` routes a closed-loop level through
`chain_loops_by_first_points`, then `chain_multiple_constrained`. Open extrusion
paths and constrained reversals also reach that shared boundary.

The shared Rust owner is geometry's exact kernel being supplied for chain_points.
Point-only calls are the coincident-endpoint special case; open paths require
paired entry/exit points, allowed reversal flags and optional start. Do not fake
those by flattening endpoints into independent points. Inspect the supplied
shared contract before wiring this adapter.

## Included and deferred behavior

Include only the <=4 dispatch and exact source floating-cost/tie semantics through
one shared implementation. Preserve orientation masks, optional start, output
indices and the legacy larger-chain path. The separate <=8 improvement pass,
local search/2-opt/or-opt, ordering policy, geometry, seam scoring and C2 arithmetic
are deferred. Existing Ares chain records are a compatibility shell around the
upstream segment/end-point concept, not a new pipeline or TSP design.

## First bounded witness

Use an ordinary Classic plate containing three well-separated square through
holes, one perimeter, no thin walls, fill, brim, support, overhang detection or
paint. Hole centers in local coordinates are `(-12,0)`, `(0,3)`, `(10,0)`; the
outer rectangle is centered on `(0,0)`. This creates a sibling level with three
closed hole loops. A first-layer witness avoids clipping and the known positive-
overhang numeric decision. Geometry remains ordinary planar manifold input.

Native-only input 1 has stable double replay and a 927-byte first-layer object
block: right hole, central hole, left hole, then the outer contour. Input SHA-256:
`cee9fb9113a3d5fc024eb3fc4f2fddd1ac84ff9708606773fbd65690a0bd70ac`.
No Native output is committed. The public test's noncentral-start band is an input
geometry precondition, not a distance tolerance or a forced corner; the complete
object-block equality remains byte-strict.

API must succeed on identical bytes and show a different first-hole/order choice;
compare the complete original first-layer object block strictly. Do not commit
Native G-code, infer private node counts from a debugger or assert a hand-computed
tour as the expected result. Runtime Native is the independent expected output.
Failure to observe the proposed ordering is not a production RED.

Limit this qualification to at most two distinct inputs. No solver code before
the geometry shared commit; no sorting, forced tie keys, contour rotation,
comparison tolerances or fixture-specific production changes. Public RED
precedes any source-port GREEN.
