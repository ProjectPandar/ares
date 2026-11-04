# Arachne available-wall candidate order: open before closed

Status: accepted (`26da8d624`); RED/GREEN and validation complete.
See [validation](2026-10-08-arachne-open-wall-order-validation.md).
Baseline `113a351e3aedeab8c73fba7a9c4cb9dd78bdb837`.
Pinned upstream `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Problem and owning upstream boundary

The six retained narrow-neck inputs in
[the finding](2026-10-08-seam-narrow-neck-finding.md) show a first-layer
perimeter sequence divergence. Both producers use `wall_generator=arachne`
and label the first extrusion Outer wall. Native emits the open narrow-neck
perimeter first; Ares emits a closed lobe perimeter first. This occurs in
Nearest as well as AlignedBack mode, so no C visibility-order implementation
is included. Complete output differences, including startup M73 changes,
remain part of the RED and GREEN criteria.

The owning rewrite boundary is
`PerimeterGenerator::process_arachne`, specifically available-candidate
sorting and its consumption in extrusion ordering, not SeamPlacer scoring.
Upstream `PerimeterGenerator.cpp`:

- `:2287–2293`: flatten generated walls in configured inset order.
- `:2295–2307`: build blocked/blocking constraints from
  `Arachne::WallToolPaths::getRegionOrder` (`WallToolPaths.cpp:809–903`).
- `:2310–2311`: initialize current_position from the first flattened wall.
- `:2319–2324`: collect only unprocessed, unblocked candidates.
- **`:2326–2328`: std::sort by `a.is_closed < b.is_closed`, open(false)
  before closed(true).**
- `:2333–2349`: retain empty-path handling; evaluate actual double norm
  despite the distance_sqr name; replace the current choice only on strictly
  smaller distance and the existing closed/finite-distance predicate.
- `:2352–2363`: append the selected wall, unblock successors, update current
  position to the first point for closed walls or last point for open walls.
- `:2471`: pass ordered_extrusions to traverse_extrusions.

`traverse_extrusions` at `:525–566` keeps the selected sequence while
materializing closed lines as ExtrusionLoop and open lines as
ExtrusionMultiPath (split only at actual discontinuities).
`GCode.cpp:6131–6146` consumes stored region perimeter entities in order.

Ares destination is
`crates/ares-core/src/project_slice/perimeters/arachne/order.rs::order_walls`.
At `:54–58` it filters the same blocked/processed candidates but sorts with
`!is_closed`, the reverse direction. At `:76–82` equal-distance candidates
cannot replace the selected wall, so scan order is observable. This is a
source-established discrepancy consistent with the finding; the Native
whole-file GREEN must still establish that it explains each chosen witness.

`arachne/materialize.rs:171` calls order_walls before traversal;
`:195` onward passes the ordered walls to traverse_extrusions. The existing
materialization/typed entity vocabulary remains the compatibility shell
around the cited upstream concepts. No new Ares scheduling pipeline is
introduced.

## Included port and exact limits

Change only the available-candidate boolean key from `!is_closed` to
`is_closed`, plus the directly misleading source comments in order.rs.
Keep filtering, flattened source order, constraint graph, starting position,
strict distance comparison, empty-path handling, current-position update,
wall sequence/sandwich handling and materialization unchanged.

This does **not** mean every open wall precedes every closed wall in emitted
G-code: dependencies and strictly nearer candidates can override scan order.
Do not replace the upstream greedy algorithm with a global open-first
entity reorder or a special first-layer rule. Open/closed priority affects
the existing candidate scan, including cross-class distance ties.

### std::sort versus Rust stable sort

Upstream uses **unstable std::sort**, not stable_sort. For the identified
libstdc++ implementation, candidate counts at most16 use insertion sort,
which preserves equivalent-key order; stable Rust boolean sorting is
therefore equivalent for this branch, with the same input candidate order.
With more than16 available candidates, introsort can reorder equal
is_closed groups. Stable Rust sorting preserves their input order and is
not generally equivalent to that upstream permutation. This also matters
when same-class candidates have equal distance.

**Explicitly deferred:** reproducing libstdc++ introsort permutations for
more than16 candidates, and a greater-than16-candidate Native witness.
The six narrow-neck witnesses are intended to exercise the small candidate
set (one open neck plus two closed lobes); that count is not inferred solely
from G-code entity counts, because materialization can split paths. Record
candidate cardinality if direct diagnosis of the existing Native binary is
needed; do not claim general std::sort parity from the boolean-key fix.
No comparator/sort abstraction or broad sorting-algorithm port is included.
A later large-set port needs its own source-cited spec and real Native RED.

Deferred also: generated Arachne geometry/widths, getRegionOrder graph
construction, reversal and equal-class tie policies, seam/visibility/
signed-distance arithmetic, flow/tool mapping, gap-fill ordering and generic
entity chaining. Do not modify an independent boundary to erase residuals.
Existing isolated ordering tests are not the oracle; do not rewrite their
assertions to manufacture GREEN. In particular the existing test named
closed/open equal distance has unequal starting positions and cannot prove
the cross-class tie rule or justify this implementation.

## RED before production

Copy the six frozen 3MF inputs into the
validation fixtures without modifying the originals. Pin their exact SHAs
from the finding. Add public byte-slicing E2E coverage with fresh identified
Native output, explicit Arachne/object-mode/height checks and the existing
whole-byte comparator (only generated-by normalization). Prefer the Nearest
43° witness for the primary RED: its first difference is actual first-layer
travel at line62, Native `G1 X108.51 Y108.61` versus Ares
`G1 X110.827 Y105.726`.

Retain all six results separately; earlier M73-first inputs retain their
true first difference and additionally record layer1 Z0.2 entity context.
Keep a wider-neck sibling (already whole-PASS) as an unchanged control.
Neither the prior investigation log nor a scalar/isolated sort example
replaces the new committed Native E2E RED. Commit RED alone
before changing order.rs. If the current-main witness disappears,
report it instead of editing production without RED.

## GREEN and merge gates

All six unchanged inputs must match complete Native bytes, including M73,
extrusion widths, travel, wipes and timing. An open-first prefix alone is
insufficient. If some inputs retain independent residual differences,
retain/report them and request revised scope; do not relax assertions or
silently add a second fix. The wider-neck sibling must remain whole-PASS.

Run fresh same-round base/candidate seeded and official control sets with
independent caches and targets.
List every changed status/reason and require candidate failures
to be a subset of base failures, with oracle instability separately evidenced.
A new failure from large candidate sets cannot be hidden by
the deferred sorting scope: retain it and report before merge.
Run workspace nextest, strict workspace/all-target/all-feature clippy, fmt
and wasm32 checks.
Rebase conflict-free and fast-forward only. No production change precedes
the committed RED.
