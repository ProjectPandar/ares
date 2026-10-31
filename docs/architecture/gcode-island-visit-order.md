# Per-filament island visits for G-code emission

## Source owner and Rust destination

Authority: OrcaSlicer v2.5.0-beta
`345efec64c7b95a05ce5cf96df0dde66a158835c`.

- `GCode.cpp:7238–7362::process_layer`: island-level gate, per-filament
  `IslandOrderNode` tour and `InstanceVisit` production.
- `GCode.cpp:7680–7720::process_layer`: consume the visit's explicit island
  sequence; use the source fallback only when the visit has no explicit islands.
- `GCode/OrderingStrategies.cpp:501–550::order_points_with_strategy` owns the
  default strategy dispatch; `:230–266::tsp_exact_cycle` owns its closed-cycle
  exact solver, including the return edge to point zero.
- `ShortestPath.cpp:1624–1650::chain_points_with_postprocessing` owns the
  complete default branch and its cited 2-opt, or-opt and crossing helpers.
- `ShortestPath.cpp:2547::chain_print_object_instances` is a known second
  consumer of that same default kernel, using instance shifts as reference
  points. Default by-layer export calls it from `GCode.cpp:3658`; the two
  witnesses have one reference point, so that call reaches the identity case.
  Reuse the same kernel at that reached adapter; do not invent another solver.
- `Polygon.cpp:154–169::Polygon::centroid` owns contour centers of mass.

Rust destination: `geometry/point_ordering.rs` and responsibility-sized children
for the default tour kernel; `project_slice/island_print_order.rs` and children
for source visit data/production; the reached object-reference ordering adapter;
and `project_slice/gcode_emit/motion/layer.rs` for visit consumption. Existing
layer/instance adapters provide source geometry, shift, filament, print-order
and wipe-tower inputs; they are temporary compatibility shells around
`process_layer`, not a new Ares-owned scheduling pipeline.

## Problem and source-rewrite boundary

The existing Rust layer emitter directly iterates stored islands. The shared
source-correct small-set `chain_points` port exposes a public nested-island
ordering regression: preparing the physical-layer order is not equivalent to
producing the beta per-filament tour and consuming its visits.

Do **not** repair this by implementing only the runtime fallback. For chainable
islands the beta producer supplies explicit visit islands, so the fallback does
not own the qualified witness. Do not restore old greedy chaining, add a tie
key, or canonicalize island/contour order to hide the missing producer.

## Included behavior and non-negotiable decisions

1. Retain the island-level ordering gate: not by-object, not a fixed sequential
   instance, and not explicit AsObjectList ordering.
2. Produce real tour nodes for nonempty chainable islands. Use the source
   contour centroid **plus instance shift**, then its symmetric integer 1 mm
   quantization. Do not substitute bounding-box centers or widen tolerances.
3. Retain the source whole-instance stop when no chainable island node exists.
4. Run `order_points_with_strategy(node_points, print_order, &wt_pos)` with the
   upstream wipe-tower start input and conversion sites. This is not a tour
   starting from the current nozzle. Reverse the selected tour as upstream does.
   Port the **whole default branch**: identity for at most three points, exact
   closed-cycle DP for four through ten, and the source greater-than-ten chain
   followed by up to three alternating 2-opt/or-opt/crossing passes. Preserve
   start-near rotation, nearer-neighbor direction and no-start closing rotation.
   Do not add an unsupported node-count admission or fall back to old ordering.
   The closed-cycle DP is mathematically distinct from the shared paired-endpoint
   open-chain kernel; neither may be substituted for or duplicated as the other.
5. Group consecutive nodes of an instance into `InstanceVisit` records, retain
   first-visit identity, and append the trailing catch-all island to that
   instance's last explicit visit. An empty explicit list retains the source
   print-everything meaning; do not invent an always-empty placeholder mode.
6. Consume explicit visit indices in their produced order. Preserve nonempty
   island filtering, source off-path fallback and final catch-all behavior.
7. Recompute the tour per layer rather than porting `m_ordering_cache`: it is a
   pure memo. If recomputation changes output, stop and reconcile that evidence.
8. Reuse shared geometry at the owning upstream boundary without changing its
   comparison, scan, arithmetic or tie rules. The exact-two-through-four-point
   commit remains separate and merge-blocked until this follow-up is GREEN in
   the same integration batch.

## Included/deferred scope

The two immutable witnesses reach one object, one instance and one filament,
with two nested material islands or four disjoint prism islands, respectively.
Both have no support, brim, sparse fill or shell fill. Port their actual
producer-to-consumer path, not fixture-coordinate or fixture-node-count branches.
Preserve real source grouping and first-visit/catch-all semantics.

The default branch is included for **all** node counts, even though the public
witnesses exercise two and four. The greater-than-ten optimization branch requires
source audit and is not directly qualified by these two inputs.
Multi-island layers are ordinary admitted input: no fail-closed limit and no
legacy/greedy fallback for unported default branches are permitted.

Snake and BestOfStrategies remain configuration-boundary deferrals: the current
admitted enum has Default and AsObjectList only. Do not add enum modes in this
slice. Explicitly defer cross-instance interleaving beyond the witnesses and the
upstream invalid-visit-index error-log path. These deferrals are not a license to
silently discard islands, index invalid data, or claim adjacent behavior as
parity. Do not add terminal behavior to the production core. The known object-
reference consumer is reached only at one reference point here; broader instance
ordering remains an integration-batch verification responsibility.

## Qualified public RED contract

Use exactly **two** inputs:

1. The approved, unchanged
   `tests/parity/clipper-wrapper-geometry/nested-hole-island.3mf`.
2. `tests/parity/island-visit-order/four-disjoint-prisms.3mf`, generated by mesh-
   only replacement of that same admitted template. One instance/part contains
   four separated, closed, positive-area vertical prisms. Static input topology
   establishes four chainable horizontal sections; runtime Native must emit at
   least four inner/outer wall groups in the first object block. This qualifies
   the four-node closed-cycle DP, not merely the small-count identity branch.

Do not add more inputs or modify the seven approved geometry cases. Freeze and
qualify the additional four-island RED before the default tour/visit port.

The agreed seam is the existing public
`ares_core::slice_project_plate_validated` API against the runtime official beta
oracle, with stable Native double replay. Compare the **entire first printed
object-layer block**, beginning immediately after its object-start marker. This
must include the first travel into the object, not start after the differing
travel. Retain all moves, feeds, progress commands and role/width metadata in
that block. The expected block comes only from the runtime oracle.

Write full API bytes and the exact compared Native/API blocks into runtime
artifacts. Commit no Native streams, output fragments, goldens or source-derived
expectations. Register the new case only in the existing native-stream grouping;
keep parity root registration and all existing comparators unchanged.

Freeze the qualified RED before production changes. After the source port,
require exact block GREEN **and different full API bytes from the RED**. Source
probes are localization evidence only and are removed before any production
commit; they cannot replace the official public RED.

Run only the scoped island witness, the seven geometry cases and the imported
seam dependency witness, plus rustfmt and strict workspace/all-target Clippy.
Global and cross-instance effects belong to the integration batch's FULL gate.
