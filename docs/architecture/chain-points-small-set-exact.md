# Exact chaining for small point sets

## Source owner and Rust boundary

Authority: OrcaSlicer v2.5.0-beta
`345efec64c7b95a05ce5cf96df0dde66a158835c`,
`src/libslic3r/ShortestPath.cpp:103–137::chain_segments_exact`,
`:164–176::chain_segments_greedy_constrained_reversals_`, and
`:1565–1575::chain_points`.

Rust destination: the shared `geometry::chain_points` dispatcher and its private
`geometry/chain_points/exact.rs` implementation. The existing greedy Rust chain
is a temporary compatibility shell for the old source boundary when it handles
all sets of two or more points. The beta boundary instead dispatches two through
four segments to the exact routine. C2 outside-in traversal must call this shared
boundary, not grow another local chaining implementation.

## Included behavior and non-negotiable decisions

Retain the existing empty/single-point return. For two through four points,
port the source subset dynamic program. As in the source `chain_points` adapter,
both endpoints of a segment are the same point, every segment may reverse, and
there is no `start_near` point. Keep both orientation states even though their
coordinates coincide; do not simplify away source tie behavior.

State index is `(mask * n + last) * 2 + orientation`. Initialize both orientations
of every singleton to zero. Iterate mask, state, candidate segment and orientation
in the original ascending order. Use the Euclidean norm, preserving conversion
of coordinates to double **before** subtraction and ordinary multiply/add/sqrt
evaluation. Neither squared distance, `hypot`, FMA nor integer-first subtraction
is a substitute for this source expression.

Only a strict `<` may replace a state's cost/predecessor. Scan the final states
in ascending order with the same strict comparison, then follow predecessors in
reverse output order. No new tie key, epsilon, canonicalization, ring rotation,
start-point preference or fixture-specific branch is allowed.

Sets larger than four retain the existing greedy/KD-tree/priority-queue path
unchanged. This is the upstream size dispatch, not an engine fallback.

## Shared open-chain kernel contract

Expose exactly one crate-visible source kernel for point and constrained segment
callers:

```rust
geometry::chain_segments_exact(
    endpoints: &[[f64; 2]],
    could_reverse: Option<&[bool]>,
    start_near: Option<[f64; 2]>,
) -> Vec<(usize, bool)>
```

Each segment retains its paired first/last endpoints; the endpoint vector is not
flattened into one representative point. `None` orientation mask means all
segments may reverse, matching the source unconstrained adapter. A supplied mask
admits only orientation zero for nonreversible segments. The optional start
initializes singleton entry costs with the source Euclidean distance; `None`
leaves them zero. Coordinates are already cast to scaled-coordinate doubles at
the caller, just as the upstream `end_points`/`start` arrays are.

The point adapter alone duplicates each point into a coincident endpoint pair,
calls the same kernel with both options `None`, and discards the returned
orientation. The classic perimeter constrained callers reuse the paired-endpoint
kernel with their real mask and start. There is no second copy of this DP. The separate
upstream `OrderingStrategies.cpp::tsp_exact_cycle` solves a closed cycle and must
not be replaced with this open-chain algorithm.

| Rust segment | Upstream segment |
| --- | --- |
| `chain_segments_exact` singleton initialization, mask/state walk, final scan and reconstruction | `chain_segments_exact`, `ShortestPath.cpp:103–137`, preserving optional start costs and allowed orientations |
| `extend_state` candidate/orientation loops and predecessor writes | The inner reachable-state transition block of the same `chain_segments_exact`; only helper extraction, with unchanged scan and arithmetic order |
| `distance` | Source `Vec2d::norm` expressions at singleton initialization and transition cost; preserve double subtraction, ordinary squared sum and sqrt |

## Deferred behavior

Do not migrate the classic caller dispatches in this slice; those changes remain
with the separate wall/seam rewrite. Do not add cycle postprocessing, change the
KD-tree or queue, or introduce a testing-only public geometry API. Global shared-
helper effects are verified by the integration batch's FULL gate, not a local
broad sweep.

## Failure modes and verification contract

The source audit must reject:

- dispatching five points to the exponential routine or leaving two through four
  on the old greedy branch;
- collapsing orientation states, initializing an artificial origin, or changing
  mask/state/candidate/orientation order;
- replacing Euclidean costs or changing coordinate cast/evaluation sites;
- using `<=`, adding a tie key, or selecting the last equally cheap final state;
- reconstructing a cycle, dropping a point, or consuming the initial predecessor
  sentinel before the final output item;
- changing the larger-set greedy implementation while adding the dispatcher.

Use the already approved, input-only public geometry RED cases and the unchanged
runtime official oracle; the pre-change eight-case run establishes the baseline.
Run the same seven geometry witnesses plus the imported fully-overhanging Nearest
seam witness after the isolated change. Inspect raw API/Native bytes without
weakening their comparators. The shared exact branch may be retained in a
separate source-cited commit only if at least one of the seven geometry cases
advances. Otherwise revert it and hand off the source discrepancy. Unrelated
footer/progress/feed failures remain strict REDs, not an inferred GREEN.

## Qualification and merge blocker

The shared dispatch advances the three approved round-offset witnesses: their
previous brim coordinate/traversal differences disappear before the object
perimeter section. This is directed progress, not full-stream GREEN.

The immutable nested-hole/island witness also exposes a regression in the first
printed island. The previous Rust stored order happened to agree with Native;
the source-correct small-set dispatch changes it. The change is **merge-blocked**
until the separately source-cited `GCode.cpp:7680–7720::process_layer` visit-island
ordering follow-up is qualified RED-first and reaches GREEN in the same batch.
Do not change the DP's source ties or restore the old greedy branch to mask it.

The seam witness's GREEN requires the separate seam-candidate slice through
`af0177a5a`; its authorized combination check is evidence of that dependency,
not permission to absorb seam-candidate production changes here.
