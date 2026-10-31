# Per-filament island visits for G-code emission

## Source owner and Rust destination

Authority: OrcaSlicer v2.5.0-beta
`345efec64c7b95a05ce5cf96df0dde66a158835c`.

- `GCode.cpp:7238–7362::process_layer`: island-level gate, per-filament
  `IslandOrderNode` tour and `InstanceVisit` production.
- `GCode.cpp:7680–7720::process_layer`: consume the visit's explicit island
  sequence; use the source fallback only when the visit has no explicit islands.
- `GCode/OrderingStrategies.cpp:501–550::order_points_with_strategy` and its
  cited ShortestPath/strategy helpers own point-tour selection.
- `Polygon.cpp:154–169::Polygon::centroid` owns contour centers of mass.

Rust destination: `project_slice/island_print_order.rs` and responsibility-sized
child modules for source visit data/production, plus
`project_slice/gcode_emit/motion/layer.rs` for consumption. Existing layer/instance
adapters provide source geometry, shift, filament, print-order and wipe-tower
inputs; they are temporary compatibility shells around `process_layer`, not a
new Ares-owned scheduling pipeline.

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

The immutable witness reaches one object, one instance, one filament and two
nested material islands, with no support, brim, sparse fill or shell fill. Port
its actual producer-to-consumer path, not a fixture-coordinate conditional.
Preserve the source model and grouping operations that this path exercises.

Explicitly defer cross-instance interleaving beyond the immutable witness and
the upstream invalid-visit-index error-log path. These deferrals are not a
license to silently discard islands, index invalid data, or claim those adjacent
behaviors as parity. Do not add terminal behavior to the production core.

## Qualified public RED contract

Use **one** existing input:
`tests/parity/clipper-wrapper-geometry/nested-hole-island.3mf`, unchanged.
A second input is permitted only if the first cannot qualify; do not expand the
search or modify the seven approved geometry cases.

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
