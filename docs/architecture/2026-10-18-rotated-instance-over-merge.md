# Diagnostic: rotated-instance over-merge in the print-toolpath sharing port

Follow-up to the shared-entity reversal carryover chain
(`5e583f8da` → `b7dcaa169`); this is a corrective slice of that port, not a
new Ares feature.

## Witness

The crosshatch-ordering RED
target with the second build item's transform rotated 90° about Z
(`transform="0 -1 0 1 0 0 0 0 1 170 110 0.5"`), first item unchanged.
OrcaSlicer 2.4.2 official vs Ares `b7dcaa169`:

- Whole-output compare: 165 diff lines. After excluding the generator header
  and `M73` lines, the real motion divergence is one relocated 36-line
  corridor block (Native emits the `G1 X98.585 Y115.98` lane block at the
  position where Ares emits the `G1 X110.33 Y124.02` corridor, first object,
  layer 2 region, native lines 2461-2497 vs ares 2423-2458); every other
  diff is an `M73` R-boundary placement — the total-time drift symptom of the
  extra travel, same signature family as the original crosshatch RED.
- Smaller paired single-line moves appear from layer 1 of the ROTATED second
  object (e.g. `G1 X162.392 Y106.828 E.03899` at native 409 vs ares 408),
  i.e. the first divergence sits in the rotated object's own emission; the
  first object's corridor swap is the downstream cursor effect (layer-loop
  interleave makes object 1's next-layer chain start from object 2's last
  position).

## Root cause (source-cited, experimentally confirmed)

`Print.cpp:2193-2194` — the FIRST condition of `is_print_object_the_same`:

```cpp
if (object1->trafo().matrix() != object2->trafo().matrix())
    return false;
```

PrintObjects with different trafos never share layers, even when their model
volumes are identical (`PrintApply.cpp:138-172` creates one PrintObject per
distinct instance trafo modulo XY shift; the sharing dedup then refuses
cross-trafo pairs). The port's `ProjectObject::shares_print_toolpaths`
(`crates/ares-core/src/project/domain/sharing.rs`) and its equivalence table
(`2026-10-18-crosshatch-infill-ordering-divergence.md`, GREEN boundary
addendum) omitted this row: two instances of one mesh that differ by a
rotation compare equal in Ares, so the reversal ledger wrongly carries
instance 1's greedy reversals into the rotated instance 2 — Native slices it
independently with pristine orientations.

Confirmation (diagnostic-only env hack, reverted, never committed): with the
sharing lookup disabled, Ares reproduces the witness's Native output
byte-identically (0 non-header diffs, 0 M73 diffs); with sharing enabled the
pure-translation target keeps needing the carryover (it diverges when
disabled), isolating the over-merge as the sole cause.

## Fix boundary (GREEN proposal)

- Upstream owner: `Print.cpp:2193-2194` (trafo equality condition) read
  together with `PrintApply.cpp:1520-1534` (PrintObject trafo = instance
  transform with the XY translation reset).
- Ares destination: the sharing predicate gains the print-object trafo
  condition — compare each print object's instance transform modulo XY
  translation (upstream `trafo().matrix()` keeps z; the comparison covers the
  full 3×4 matrix). The transform is already reachable per print object via
  `PlannedPrintObject::transform_index` → `resolved.print_objects[transform_index]`
  (the same path `gcode_emit/motion/auto_lift.rs` uses for centering);
  `EmitState::shared_toolpath_object` takes the transform index alongside
  `source_object_index`, or the representative lookup keys on
  `(source object, trafo)`.
- Spec update: add the missing row to the equivalence table and note that
  `shares_print_toolpaths` moves from a pure `ProjectObject` predicate to a
  per-print-object comparison (volume fields from the ProjectObject, trafo
  from the print-object resolution).

## RED proposal

Freeze `target-rotinst.3mf` as `tests/parity/crosshatch-ordering/
target-rotinst.3mf` with a leg mirroring the existing target test: pin the
native corridor order (X98.585 lane before the X110.33 corridor on the
first diverging layer) plus the whole-output strict compare (DIVERGENT until
GREEN). Control leg: the existing translated-instance target must stay PASS
(carryover preserved).

## Included / deferred

- Included: the trafo condition port, the equivalence-table row, the
  rotated-instance witness leg, regressions (crosshatch suite both modes,
  brim/infill/crosshatch official, clippy, fmt, wasm).
- Deferred: none for this slice; the already-accepted seam/MMU/fuzzy facet
  and object-level extruder gaps are unchanged.
