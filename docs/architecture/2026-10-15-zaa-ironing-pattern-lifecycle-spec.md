# Object ironing pattern replacement and regional ZAA lifecycle

## Status and source boundary

SPEC for review only; no Native or build run has been started. The candidate
is based on main `2fe214545c1a50e88bf321b8221f0eba73a83ad8` plus the approved
object-ironing ZAA chain `8c9a1bc94..32e111a06`. Worktree:
`${WORKTREE_ROOT}/ares-worker2-zaa-ironing-pattern-lifecycle-spec`.

The owning upstream boundary is `OrcaSlicer/src/libslic3r/Fill/Fill.cpp` in
`Layer::make_ironing`:

- `1605-1614` derives each `IroningParams` angle/fixed-angle/pattern and
  records its source layer region.
- `1514-1525` defines `IroningParams`; `1527-1536` sorts only by
  extruder, just_infill, line_spacing, height, speed, angle, fixed_angle and
  inset. `pattern` is intentionally absent from `operator<`; it is checked by
  `operator==` at `1539-1548`. Therefore a Concentric and Rectilinear span
  with otherwise equal values remain adjacent in sorted order, while their
  pattern difference forces the replacement branch. The fixture must not rely
  on that equal-key ordering: it sets Concentric `ironing_angle=0°` and the
  following Rectilinear ZAA=true region `ironing_angle=45°`, with all other
  compared fields equal. This strict angle key makes Concentric sort before
  Rectilinear deterministically under the non-stable `std::sort`.
- `1622-1627` creates the initial Rectilinear filler and initializes object
  bounding box, layer id, z and overlap.
- `1628-1631` starts each equivalent span and assigns the span region's
  `zaa_enabled && zaa_dont_alternate_fill_direction` to the CURRENT filler
  before any pattern replacement.
- `1633-1641` then replaces the filler when the span's pattern changes. The
  freshly constructed replacement has the default
  `dont_alternate_fill_direction = false`; upstream therefore DROPS the
  regional ZAA bit for that replacement span. The faithful Rust port must
  preserve this ordering and dropped bit; it must not reapply the region flag
  after replacement. Object/layer/z/overlap are reset on the replacement.
- `1643-1644` computes the equivalent `IroningParams` span that receives the
  same filler and timing parameters.
- `1705-1718` applies each span's spacing/angle/fixed-angle and invokes
  `Fill::fill_surface`; `FillBase.cpp:275-323` (especially `311-313`) applies
  the regional direction suppression.

PrintConfig.cpp:4298-4308 restricts `ironing_pattern` to Rectilinear and
Concentric. The approved Ares implementation already handles those two
patterns in `fill_entities/ironing.rs:55-125` and carries the regional ZAA bit
at `:127-149`, but its `Input` is consumed one region at a time and has no
upstream equivalent of the sorted span/replacement lifecycle.

## Rust destination and scope

The future rewrite destination is `crates/ares-core/src/project_slice/
fill_entities/ironing.rs`, with only the minimum grouping/state seam needed to
mirror the cited `Layer::make_ironing` lifecycle. Existing Concentric and
Monotonic generators remain the geometry carriers; no new Ares-owned ironing
pipeline is proposed. The approved `dont_alternate_fill_direction` assignment
and its FillBase direction kernel remain unchanged.

Included in this slice: a two-region, one-object fixture where adjacent
printing regions use Rectilinear and Concentric ironing settings. The port
must preserve the upstream sorted-span boundary, recreate/reset filler state
when the pattern changes, and apply each span's region ZAA conjunction. The
fixture will compare complete API/CLI output and role/XY/E streams at each
span boundary.

Deferred: more than two regions, multiple extruders and per-extruder grouping,
the equivalent-span merge at `Fill.cpp:1643-1644` (including non-adjacent spans
that become equivalent after sorting), `IroningParams::just_infill`
(`Fill.cpp:1649-1652` remains TODO upstream), support ironing,
raft/tower ironing, pattern schema expansion beyond PrintConfig.cpp:4298-4308,
bridge/anchoring inset behavior, and any independent ironing flow/speed or
CoolingBuffer difference. No change to the comparator or existing assertions.
Keep every changed Rust file <=400 LOC.

## Native-first plan after SPEC approval

Use one frozen input-only 3MF derived from the approved wide-slab provenance,
with two printing regions and the same mesh/profile members. Region A uses
Rectilinear ironing plus ZAA flag false; Region B uses Concentric ironing plus
ZAA flag true. Add a mirrored control with both regions' ZAA disabled and a
single-pattern control with both regions Rectilinear. Keep topmost ironing,
fixed ZIP timestamps, classic wall0 relative E and no modifiers/support.

Run official CPU13 and seeded CPU14, outer taskset including all subprocesses,
jobs2/threads2, canonical environment, two fresh Native references per case
and mode. Record effective options, sorted region/span role order, whole output
and per-span Ironing XY/E streams, repeat/cross-mode normalized hashes and
non-Ironing XYZ/E invariance. Native must accept all cases, remain stable, and
show the replacement ordering and the dropped-bit direction result on the
Rectilinear ZAA=true span, plus no unrelated region drift. If Native rejects,
crashes, lacks a replacement witness, or Ares already matches the complete
outputs, stop with NO-RED FOUND; do not widen the fixture.

Only after an active witness: submit a separate test-only RED using
`OrcaRunner::from_env` and the unchanged strict whole-output comparator.
Controls must PASS and the mixed-pattern case must fail at the localized
pattern/span boundary in both modes. Production GREEN requires only the
upstream lifecycle seam, existing ZAA/fill-direction/anchoring controls and
strict/fmt/wasm gates. Full/workspace/CI/merge remain coordinator-only.
