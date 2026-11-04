# Proposal: per-object skirt x flush_into_objects — object-wipe tool ordering residual

Status: Proposed (probe-only; guard retained)

## Probe (Native first, frozen input)

`pf-obj2` = R1 SEMM base (prime tower, 2 filaments) with
`skirt_type=perobject`, `flush_into_objects=1`:

- Native: rc=0, accepted.
- Ares (guard term locally lifted, probe only): rc=0, normalized
  divergence = 21851 lines / 310 diff hunks.

## Concrete divergence (beyond the guard)

The first divergence is the layer tool plan itself, not skirt geometry:

- Header line 6: `; filament: 1` (Native) vs `; filament: 2,1` (Ares) —
  Ares schedules the layer's extruders in the opposite order.
- Layer 1 order: Ares emits M600 + `;_FORCE_RESUME_FAN_SPEED`, the
  second object, and the prime tower first (1500+ lines before Native's
  first object); Native starts with tool 1's object directly. Ares
  emits 52 M600 tool changes where Native emits 1.
- Skirt blocks themselves are 2 vs 2 — the skirt geometry/count is
  already at parity; everything else in the layer sequence reorders
  around it (101 `;TYPE:` lines displaced).

## Upstream owner

`ToolOrdering.cpp` `WipingExtrusions::mark_wiping_extrusions`
(:1613-1712) + `is_overriddable` (:1586-1596): with
`flush_into_objects`, `is_overriddable` returns true for ALL roles, and
`mark_wiping_extrusions` SORTS the layer's objects so dedicated
(dedicated to wiping) objects come FIRST (:1627-1636), iterates them
twice (perimeters then infills, `perimeters_done`, :1641-1655), and
marks perimeters/infill entities as overridden (:1672-1700); the
remaining volume goes to the tower. The resulting per-layer extruder
sequence feeds `GCode.cpp:5326-5360`'s two-loop emission (already at
parity). Ares' object-wipe marking does not reproduce the dedicated
object sort / two-phase (perimeters-then-infill) marking, so the layer
tool order and override set diverge from the very first header line.

## Scope sketch (SPEC to follow if accepted)

- Port `mark_wiping_extrusions` object-wipe semantics: dedicated-object
  sort (:1627-1636), two-phase perimeter/infill marking
  (:1641-1700), `is_overriddable`'s flush_into_objects==true→all roles
  (:1589-1590).
- Rust destination: the tool-ordering/wiping-extrusions marking module
  feeding `gcode_emit/layers/tool_buckets.rs`.
- GREEN retires the last guard term (`flush_into_objects`); guard
  comment goes away with the disjunct.
- Blast radius: the whole wiping-extrusions marking path — gate must
  include the full tower/wipe/toolchange/skirt families both modes.

## Deferred

- Nothing else in the per-object skirt guard: this is its last term.
