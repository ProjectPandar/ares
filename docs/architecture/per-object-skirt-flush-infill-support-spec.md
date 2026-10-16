# Spec: retire the flush_into_infill/support sub-disjuncts of the per-object skirt guard

Slice status: DRAFT (awaiting review)
Upstream boundary: `OrcaSlicer/src/libslic3r/GCode.cpp:5330-5333` (wiping-pass
extrusions), `Print.cpp:2784-2996` (per-object skirt grouping), `GCode.cpp:4465-4485`
(per-group skirt emission).

## Background

The per-object skirt GREEN (`w5-skirt-per-object` chain, now in stack-24)
kept a residual guard in `gcode_emit/skirt/generate.rs`:

```text
|| (tower_active && any(flush_into_objects || flush_into_infill || flush_into_support))
```

The guard anticipated a wiping-pass skirt divergence that was never
witnessed. The tool-change wipe timing slice (`4ef40ffed..d569edcbe`)
removed the timing residual that motivated it.

## Discovery (native first, CPU7,8, R1-derived SEMM fixtures)

With the full disjunct removed (offline probe, restored afterwards):

- `pf-tgt` (skirt_type=perobject, flush_into_infill=1, prime tower on):
  native rc=0, normalized diff = 0.
- `pf-sup` (skirt_type=perobject, flush_into_support=1): native rc=0,
  normalized diff = 0.
- `pf-obj` (skirt_type=perobject, flush_into_objects=1): native rc=0,
  ares diverges massively (`; filament: 2,1` vs `1`, M600 ordering) — a
  separate tool-change/print-order residual, NOT skirt-related. Out of
  scope here; stays guarded.

## Included

- Remove `flush_into_infill` and `flush_into_support` from the guard
  disjunct; `flush_into_objects` remains guarded.
- Public RED: two test legs (pf-tgt, pf-sup) asserting strict
  native/ares parity. Today ares REJECTS both inputs
  (`UnsupportedProjectFeature("skirt_type per-object")`), so both legs
  FAIL before the GREEN.

## Deferred

- `flush_into_objects` + per-object skirt: diverges in tool-change and
  print ordering (`; filament:` header, M600 placement), a separate
  upstream boundary (GCode.cpp tool-change sequencing); needs its own
  slice.
- No skirt geometry changes: the tower-obstacle grouping itself is
  already at parity (stack-24).

## Exit criteria

- Both RED legs FAIL at the RED hash, PASS at the GREEN hash
  (canonical official+seeded, targeted, CPU7,8).
- Existing skirt/wipe/tower/toolchange_wipe suites stay green.
- fmt/clippy/wasm clean; no file over 400 LOC.
