# Orca Lattice Parity Milestone (4096/mm SCALING_FACTOR)

Status: PROPOSED — planned, not yet scheduled. This spec records the
investigation results that motivate the milestone and the staged plan.

## Motivation (evidence, 2026-09-14/15)

Three divergent buckets trace to coordinate-lattice differences between
ares and OrcaSlicer 2.4.2:

1. `top_surface_pattern=octagramspiral` fill ORDER: the emission reorder
   (validated as an exact port of upstream's v1 greedy) flips tie
   decisions because ares's geometry ints (1e6/mm, object-centered)
   differ from orca's (4096/mm, world-absolute) by ±1-2 units. The
   greedy compares squared distances on exact ints; sub-µm input noise
   flips equal/near-equal comparisons (probe: 4096-rounded inputs match
   orca's first six fragment picks, then flip at position 9).
2. Loop geometry ±1 scaled-int cases (smoke sweep "other" bucket).
3. Elephant-foot ±1 µm corner cases (E3NG family).

## Root findings

- `libslic3r.h:94-95`: `#define scale_(val) ((val) / SCALING_FACTOR)` —
  a pure division MACRO returning double. `SCALED_EPSILON =
  scale_(1e-4) = 0.4096` stays FRACTIONAL; ClipperLib offset deltas are
  doubles. Orca coordinates are ints (Point ctor rounds), but OFFSET
  DELTAS and epsilon amounts are unrounded doubles in scaled units.
- Ares quantizes every mm→scaled conversion to lattice ints
  (`CoordinateScale::checked_scale`), so `closing_ex(ε)` gets delta=100
  at 1e6/mm (same geometry) but delta=0 at 4096/mm → assertion failure
  (verified by the lattice-probe branch, since deleted).
- Ares `CoordinateScale`: Normal=1e-6/mm, LargeBed=1e-5/mm (dynamic).
  Orca: fixed `SCALING_FACTOR = 0.000244140625` (1/4096).
- Independent µm quantizations exist outside CoordinateScale:
  `contours.rs:238`, `segments.rs:173`, `extrusions.rs:255`
  (`(value * 1_000_000.0).round()`), `fuzzy_skin_noise.rs`,
  `shell_layers.rs`, `support_ironing.rs`, `mesh_slicer.rs`. Orca's
  mesh slicing runs in f64 with no µm quantization.

## Audit inventory

- ~126 `checked_scale` sites + ~30 unscale sites + 3
  `scaled_offset_coordinate` sites: classify each as COORDINATE (round,
  Point-ctor semantics) or DELTA/EPSILON (unrounded `mm / factor` as
  f32/f64, orca's double-delta semantics).
- The hardcoded ×1e6 sites above: remove the quantization or route
  through the scale.
- Tests with hardcoded 1e6-scale ints (classic_clip, seam placement,
  timelapse, etc.) need regeneration on the new lattice.

## Staged plan (each stage gated by golden + targeted parity cases)

1. **Spec + inventory table**: classify all conversion sites
   (coordinate vs delta) in a checked-in audit table.
2. **Delta semantics fix** (independent of the lattice switch!):
   convert DELTA/EPSILON sites to unrounded scaled doubles. At 1e6/mm
   this is nearly a no-op (100 vs 100.0); it unblocks the lattice
   switch and matches orca exactly.
3. **Mesh-slice quantization removal**: keep f64 where orca keeps f64.
4. **Lattice switch**: NORMAL_SCALE = LARGE_BED_SCALE = 1/4096 (the
   dynamic distinction becomes vacuous with i64 coords — consider
   collapsing the enum). Fix int-overflow guards and test fixtures.
5. **Full sweep + golden validation**; expected to resolve the three
   buckets above and possibly part of the M73 knife-edge family
   (sub-ms timing shifts from ±1 geometry).

## Risks

- The 866 PASS baseline moves: outputs recompute on the new lattice.
  Cases where ares-1e6 happened to round to orca's printed mm but
  ares-4096 differs would newly fail — mitigated by the fact that
  ares-4096 should equal orca-4096 exactly wherever the algorithm ports
  are faithful.
- WASM/Windows i64 arithmetic unchanged (coords stay i64).
- Effort: the inventory classification is mechanical; the fixture
  regeneration is the bulk.

## Probe results (2026-09-15, continuation #81)

- Fourth lattice family confirmed: the Flashforge Adventurer X4.464
  group (6+ cases) — ares emits a duplicate travel waypoint whose two
  positions differ by sub-4096 amounts at the 1e6 lattice (both print
  as X4.464 Y4.464), while orca's 4096 lattice collapses them to one
  integer and the router's consecutive-duplicate skip
  (`router.rs` `to_polyline` emulation) drops the hop.
- Second lattice probe (lattice-m2, deleted): after the stage-2 delta
  fix, the 4096 switch no longer panics on the Flashforge case, but the
  output diverges WORSE (5485 vs ~988 lines) starting at the priming
  anchor (`G1 X-9.007` → `X-9.009`, a 0.002 mm shift = 8 lattice
  units). At least one upstream-faithful stage still computes
  differently on the new lattice — the switch requires the full audit
  table (stage 1) plus per-stage anchoring before flipping.
