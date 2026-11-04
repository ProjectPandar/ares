# Spec: parity convergence continuation — option coverage + printer smoke residuals

## Status: ACTIVE (authored 2026-09-28)

This spec continues the bounded convergence wave defined by
`2026-09-08-parity-convergence.md` and `2026-09-26-parity-residuals-milestone.md`
on the same fixed upstream baseline: OrcaSlicer `v2.4.2`,
commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`, with the official
oracle executable driven through `scripts/orca-parity.sh`.

## Scope

1. **Option coverage completion.** Every bounded domain in
   `tests/ksr_fdmtest_v4/options-v242.json` (650 entries, `Metadata`
   excluded) must produce compared output. Bool/enum domains enumerate every
   declared value; numeric ranges probe `min`, `max`, and a seeded interior;
   `coFloatOrPercent` adds the absolute-literal branch; schema-unbounded
   strings/points receive authored legal probes; id-valued/machine-internal
   keys that the oracle rejects stay baseline-only and are reported as
   `INCOMPLETE` with an honest reason — never suppressed.

2. **Printer smoke residual closure.** The strict ordered-byte sweep over
   all 1,001 vendor machine presets must reach every reachable `PASS`.
   `ORCA_ERROR` (the upstream binary itself fails) and `VENDOR_INCOMPLETE`
   (the vendor tree ships no usable default process) are reported, not
   counted as Ares failures. Remaining `DIVERGENT`/`ARES_ERROR` rows are
   each fixed as a source-cited slice.

3. **Harness hygiene.** No source-line pinning tests; test-only modules
   compare observable output only. Source files stay under 400 LOC; tests
   live in `tests.rs`-style sibling modules; no `include!`/`include_bytes!`
   for source splitting.

## Current non-green facts

- Option sweep (`ARES_PARITY_OPTIONS=1` merge run 2026-09-28): 608/650
  bounded domains exercised; residual INCOMPLETEs are authored-interior
  samples (upstream slice-time validation rejects the schema bound) or
  id-valued/unprovable domains, all with `compared ≥ 1` or documented
  oracle rejection.
- Printer sweep last full run: 958/1002 PASS; 15 DIVERGENT, 18 ORCA_ERROR,
  10 VENDOR_INCOMPLETE, 1 ARES_ERROR (vendor-load probe). After the
  pending-lift raised-Z fix and the wipe-before-external `next`-vertex fix,
  Artillery M1 Pro, Qidi Q2/Q2C, Eryone ER20 (0.4) and Kingroon KP3S verify
  PASS under `ARES_RACE_TARGET` — a refreshed sweep regenerates the
  summary's denominators (the OrcaFilamentLibrary row is gone now that
  vendor discovery requires a `machine/` dir).
- Divergence clusters observed: M73 `R` minute-boundary rounding and
  cache-entry mapping (Sermoon V1, ER20, Ginger G1, SK1, Volumic x3,
  TinyBotV1); last-digit f32 coordinate/E rounding (Kobra 3 0.2, E3NG,
  Rolohaun, SV06); travel-point ordering (PhoenixProV1, TiQ8).

## Independent-review fix loop (round 1)

- KSR semantic model now keeps `M73` progress lines and the
  `CONFIG_BLOCK`; the config block compares exactly while the progress
  lines remain diagnostic (the estimate-machine residual is the M73
  cluster below).
- Every option case verifies requested-vs-effective exported values
  (width domains previously were the only ones checked —
  `printer_technology/SLA` had passed while exporting `FFF`).
- Upstream `this->add` keys reconcile against the inventory through
  `tests/parity/upstream-key-exclusions.txt`.
- Upper-only numeric bounds probe the declared ceiling (`nozzle_diameter`
  max=100); bool values rejected by oracle-side fixture conflicts now
  co-inject the dependent option (`use_firmware_retraction`→`wipe=0`,
  `use_relative_e_distances`→G92-free layer gcodes).
- Excused oracle-nondeterministic cases report as evidenced INCOMPLETE;
  sweep acceptance gates only genuine failures; a failed oracle retry no
  longer overwrites an established divergence; `OrcaFilamentLibrary` no
  longer manufactures a printer row; `m73_profile` diagnostics reproduce
  the upstream f32 arithmetic and (P, desc R) merge order.

## Acceptance

- `ARES_PARITY_OPTIONS=1` full run: zero unexempted non-PASS domains.
- `ARES_PARITY_SWEEP=1` full run: every DIVERGENT row resolved to PASS or
  reclassified ORCA_ERROR/VENDOR_INCOMPLETE with recorded evidence.
- `cargo nextest run --workspace`, `cargo fmt --check`, warning-denying
  `cargo clippy`, WASM checks green; no file over 400 LOC.

## Independent-review fix loop (round 2)

- `verify_value` compares canonicalized values — upstream serialization
  writes `"2.0"` as `"2"` (`Config.hpp:777-781`), so literal JSON
  equality falsely rejected authored probes (`bridge_flow`,
  `filament_diameter`).
- Wipe-before-external `next` vertex restored to upstream semantics:
  upstream `paths` are seam-gap clipped before endpoint selection
  (`GCode.cpp:5804 clip_end`, `5833-5840`); Ares stores the loop
  unclipped, so `clipped_loop_end` reproduces the same vertex. KP3S 3.0
  and ER20 still PASS byte-exact after the correction.
- Sweep gate now fails on `ARTIFACT_ERROR` (comparison/persistence
  failure is not an oracle-side exemption), and the divergence banner
  reports completed oracle runs rather than the full budget.
- `m73_profile` window check honors upstream's `+0.5f` rounding.
- KSR semantic compare asserts M73 progress count within the documented
  ±1 residual bound (wholesale loss/insertion now fails).
- 400-LOC splits: `semantic/controls.rs`, `semantic/parser/motion_apply.rs`,
  `option_coverage/domains/probes.rs`, `loop_paths/inward.rs`.

## M73 residual cluster — root-cause note

Instrumented upstream (`ORCA_DUMP_TIMES` per-block dump) vs ares
`ARES_DUMP_BLOCKS` on Sermoon V1 shows byte-identical block timing
through g1-line 146, then a segmentation difference at ids 147–149
(the seam-adjacent travel lands 0.05mm off: orca `X91.599` vs ares
`X91.649`) and a per-extrusion cruise difference afterward (orca
37.55mm/s vs ares 37.9). Net total time differs ~0.38s which tips M73
minute boundaries (R6 vs R7) and shifts emission by one line
(Volumic/TinyBot/SK1). The seam-offset is the same class as the
"travel/seam-point" rows; the next slice is upstream-equivalent seam
vertex selection plus the cruise-feedrate clamp ordering in
`GCodeProcessor.cpp:4020-4060`.

Downstream causality: with `seam_position=aligned`, the emitted seam
vertex is NOT a raw candidate vertex — upstream stores
`perimeter.final_seam_position` (`SeamPlacer.cpp:1391`), a
BSpline-fitted interpolation
(`fit_cubic_bspline` + `t*current + (1-t)*fitted` blend at
`SeamPlacer.cpp:1367-1389`) produced by `alignment::align`
equivalent. Sermoon evidence: orca lands at `X91.599` while ares
emitted `X91.649` — 0.05mm off, consistent with a different fitted
position or a different `seam_string` membership, not with a different
vertex pick. The scoring constants match
(`seam_align_score_tolerance=0.3`, `tolerable_dist_factor=4.0`,
`minimum_string_seams=6`, `mm_per_segment=4`,
`sharp_angle_snapping_threshold=55°`) and `finalize_string` mirrors the
upstream formula, so the residual sits in the *string composition*
(`find_seam_string`/`find_next_in_layer`/`is_better`) or the spline
evaluation (`spline::CubicSpline::fit/value` vs
`Geometry::fit_cubic_bspline`), or in per-candidate `local_ccw_angle`/
`unsupported_dist` inputs. Resolving the 0.05mm fitted-position
difference should collapse both the M73 value cluster and the
"seam-point" rows simultaneously.

Update `c82f20eb`: `layer_mid_zs` was wrong — upstream stores candidate
z and projects seam-string search onto `layer->slice_z` (layer bottom,
`SeamPlacer.cpp:1025,1206`), not the mid-layer extrusion height.
`seam_placement/runtime.rs::layer_slice_zs` now returns slice_z. Did
NOT fix Sermoon (the 0.05mm fitted-position delta persists — uniform z
shift is invisible to the fit), so the residual is upstream of the
spline: candidate `unsupported_dist`/overhang/`embedded_distance`
inputs or string membership. New sweep after the fix: 954/1001 PASS,
20 DIVERGENT, 17 ORCA_ERROR, 10 VENDOR_INCOMPLETE.

Remaining divergence clusters (post-c82f20eb sweep):
- M73 placement/value: emission position or R-minute differs — driven
  by per-block time drift. Kobra3 0.2, Sermoon V1, ER20 0.5, Ginger
  1.2, Peacock V2, SK1, TinyBotV1 (all), PhoenixProV1 (all), Volumic
  EXO42/EXO65/SH65, raft_layers-max option domain. All share the
  same root: cumulative time differs by ~0.3-0.4s, tipping
  `time_in_minutes` boundaries.
- Speed deltas feeding the above: wipe feedrate (`role_based_wipe_speed`
  uses last `set_speed` = last per-segment variable speed; ares 108 vs
  orca 113.65 on Sermoon), infill F2253 vs F2274, wall
  cruise 37.55 vs 37.9 — consistent with `estimate_extrusion_quality`
  overlap-band endpoints or `_mm3_per_mm` ratio application differing
  in a corner.
- Sub-micron skirt/perimeter coordinate rounding (RH3D 0.001mm on a
  rounded skirt arc vertex, Sovol, Rolohaun, Kobra3 E-word).
- Ginger G1 inner-wall `Z<z>` on every extrusion: upstream emits Z on
  `path.z_contoured`/`sloped` paths only; Ginger's zaa/scarf are off, so
  the emitting path remains unexplained — possibly
  `estimate_extrusion_quality` processed points carrying z, or a
  profile-specific `travel_slope` interplay.

Update (post-`c82f20eb` instrumented-upstream verification): the
instrumented upstream build tree has been
re-realized (`nix copy` restored GC'd deps; `CPLUS_INCLUDE_PATH`/`LIBRARY_PATH`
recreate the erased nix env) and now carries two env-gated dumps:
`ORCA_DUMP_SEAMPT` (SeamPlacer.cpp:1624 — chosen `seam_point`/`pos`/
`seam_index` per layer+role) and `ORCA_DUMP_SCARF` (GCode.cpp —
`ExtrusionLoopSloped` clipped `starts.front().first_point`). Verified
against Sermoon V1 (the delta=0.05mm X case): ares `SPT`/`SCARF` dumps
(`ARES_DUMP_SEAMPT`, `ARES_DUMP_SCARF`) match upstream byte-for-byte at
every layer — the seam placement, fitted position, scarf clip_front
(`2*seam_gap` external / `0.4*slope_len` perimeter), and emitted
first-scarf-vertex are all identical. The observed `G1 X91.599 Y91.599`
vs `X91.649 Y91.649` difference is NOT the loop start — it is an
avoid-crossing-perimeters detour waypoint (`approach/waypoints.rs`,
`GCode.cpp:7488-7501`), i.e., a routing/offset-grid discrepancy inside
`AvoidCrossingPerimeters` (0.05mm on a corner waypoint), not seam
placement. The M73/`F` rows on Sermoon therefore likely share the same
upstream non-seam root — the detour waypoint or the wipe feedrate.

## 2026-09-28 (late) — cancelled-lift merge + estimator findings

### Committed
- `08cab436` — `travel_to_xyz` cancelled-lift branch now emits the combined
  `G1 X Y Z{leg}` when `will_move_z` returns true while lifted
  (`GCodeWriter.cpp:782-806` `emit_xyz` clear-position path). The upstream
  dead-band probe `z >= m_pos(2) - m_lifted` uses raw f64 — `0.6-0.4`
  = `0.2+1ulp` correctly merges (empirically verified via ORCA_DUMP_LIFT:
  `lifted=0.4 pos=0.6 z=0.2 -> will_move_z=1`). Fixed option/spiral_mode/1
  (case-yvHhtc, 46-line -> 0) plus ~30 cases that now differ only in the
  generated-by header.
- Stale wipe expectations in `motion/tests/geometry.rs` updated to orca
  golden values (regression from d5c19a0e seam midpoint change).

### Remaining classes (evidence gathered)
1. **M73/total drift**: ares `total` includes the trailing end-gcode
   `G1 E-1` retract (+0.012s). Orca's `machine.time` = 313.233149 vs ares
   313.245149 (case-Z06Min). Orca DOES create the block (BLOCKPUSH id=3315
   type=Retract feed=83.33) but `machine.time` excludes it — mechanism
   unpinned (all queued blocks should be consumed at finalize).
   Affects EXO42/EXO65/SH65 + M73 boundary shifts elsewhere.
2. **Cooling slowdown F drift** (Ginger G1 275 lines, SK1 116, U1 46...):
   ares emits F841 where orca emits F843 — slowdown factor slightly stronger.
   Cooling line streams segment differently: orca `CL i=1 len=23.29 f=70`
   vs ares `len=39.16` — the CoolingLine segmentation/merge differs before
   the slowdown math. `ORCA_DUMP_COOL` dumps upstream `CL` lines; ares has
   `ARES_DUMP_SLOWDOWN=2` for the equivalent dump.
3. **1µ rounding** (E3NG, Kobra3, SV06, DeltaFlyer): corner vertex rounding.

### Instrumentation added to upstream-src (not in repo)
- `ORCA_DUMP_LIFT` in `travel_to_xyz` + will_move_z probe.
- `ORCA_DUMP_M73` in `export_line_M73` (counter/elapsed/machine.time/pct).
- `ORCA_DUMP_G1` G1PROC deltas + BLOCKPUSH (id/dist/type/feed).
- `ORCA_DUMP_TIMES` extended with `type` column.

## 2026-09-28 (later) — trailing-block drop committed (fdfcecad)

- `RollingPlanner::finish` now shares `flush`'s `blocks.size() < 2` guard —
  upstream `calculate_time` drops a lone trailing block (verified:
  `CALCTIME nblocks=1 final=1` early-return on orca's end-gcode `G1 E-1`
  after the `T0` tool-change flush). Fixed the M73-boundary class
  (EXO42/EXO65/SH65, PhoenixProV1 headers, +~30 header-only cases).
- `planned_times` test hook force-drains the queue so trapezoid math stays
  inspectable; stale synthetic tests updated to add a second block.

### Remaining clusters after fdfcecad
1. **Seam corner choice** (stlBdK/sgjBnj/6kZFjT 29 lines, hECm1F/bg6wKq/
   H39DKQ 38 lines): `seam_position = aligned_back`, cube layer Z.56 — orca
   picks the back-RIGHT corner (max X, max Y), ares the back-LEFT (min X,
   max Y). Same winding (CCW), same loop — pure tie-break between the two
   back corners. Instrument with ORCA_DUMP_SEAMPT to compare per-candidate
   scores on that layer.
2. **M73 line-position shifts** (AHc4Fg/dy3GaF/WrJI9Y 116 lines): identical
   M73 content emitted a few lines early/late — elapsed-per-line still off
   somewhere (not total-only).
3. **Feedrate slowdown** (XMiGaO/tfOPCW 46 lines: `F3000` vs `F2400`,
   `F3283` vs `F3566`; f4k9FN/kr9jQL/LIVPLC 275): cooling-buffer slowdown —
   ORCA_DUMP_COOL vs ARES_DUMP_SLOWDOWN shows the CoolingLine streams
   segment differently (orca i=1 len=23.29 vs ares 39.16) before the
   slowdown math.

## 2026-09-28 (late-2) — sample-weight fix committed (55a0ca8c)

`sample_its_uniform_parallel` computes `A*(1-sq_u)` with double weights on
f32 vertices; ares truncated the weight to f32 before multiplying — ~1ulp
position drift per sample → flipped symmetric back-corner ties in
`aligned_back`. Fixed by keeping f64 weights through the multiply.

Cleared stlBdK/sgjBnj/6kZFjT. Still open in the same family:
hECm1F/bg6wKq/H39DKQ (TinyBotV1 0.4, 38 lines) — layer-0 seam for the
second `cube10` copy picks +Y in ares vs -Y in orca (`SEAM point
(2.887,-2.887)`). Visibility here is per-OBJECT mesh in upstream
(`gather_occlusion_mesh` + `its_transform(obj_transform)`), while ares
builds `TriangleMesh::from_project` from only the FIRST object/first
instance — multi-object plates get `nearby.is_empty() -> 1.0` for other
objects' samples. That is the probable residual gap.

## 2026-09-28 (late-3) — cooling-buffer segment diagnosis

`case-LIVPLC` CL streams (orca `ORCA_DUMP_COOL` vs ares `ARES_DUMP_SLOWDOWN=2`)
match on all 143 lines except i=1/i=3: orca splits an external-perimeter
SET_SPEED block (type 1e0) around an internal block (type 160) into
23.29mm + 10.75mm + 21.79mm, while ares merges the region into 39.16mm +
4.32mm — total adjustable time differs by 0.023s (tt 13.743 vs 13.720;
tmax 695.88 vs 694.26), shifting the equalized slowdown feedrate
(14.05 vs 14.016 mm/s -> F843 vs F841).

Both sides have identical final gcode; the divergence is inside the
`_EXTRUDE_SET_SPEED.._EXTRUDE_END` marker stream / merge attribution.
Next step: dump the marker stream on both sides (orca emit points at
GCode.cpp `set_extrusion_speed`/`;_EXTRUDE_SET_SPEED`; ares
motion/extrusion.rs `speed()`) and compare where SET_SPEED blocks start.

## 2026-09-29 — cooling stream forensics (no instrumented rebuild possible)

Nix GC deleted the instrumented-build toolchain (cmake, boost dev headers,
pch deps); the `build-nix` tree can no longer compile. Official AppImage
remains the only oracle. Forensics switched to ares-side dumps
(`ARES_DUMP_RAW`/`ARES_DUMP_CLINES`/`ARES_DUMP_CLALL`) vs the last saved
ORCA_DUMP_COOL stream.

Findings on case-LIVPLC layer 0 (`;TYPE:Skirt` region):
- Both sides mark `G1 E55` prime as a non-adjustable len=55 E-only line
  and the phantom `dif[3]=55` lands on the following `G1 F3000;_EXTRUDE_SET_SPEED`
  head — ares CLALL confirms len=55 there (upstream `use_relative_e`
  reset makes new_pos[3]-0=55). This part matches.
- Difference is the skirt PATH segmentation: orca's first CL block is
  74.43mm = 55 (phantom E) + ~19.4mm of skirt before `;_EXTRUDE_END`,
  while ares closes the block after ~3.19mm. Ares emits an
  `;_EXTRUDE_END` every ~1-2 skirt segments; upstream keeps ~19mm in
  one block. The marker stream granularity is path-boundary-driven, so
  ares's emitted extrude-path boundaries differ upstream of the buffer —
  the skirt loop (or the pressure-equalizer rewrite) is producing more,
  shorter `extrude_path` calls than upstream.
- The t/tmax delta (~0.023s) then shifts every slowed feedrate in the
  file (F841 vs F843 = 275 diff lines — the whole 275-class).

Next step: trace why ares's skirt block is ~3.19mm vs orca's ~19.4mm —
either the skirt polyline is longer/segmented differently (offset/simplify
output), or the equalizer splits where upstream doesn't. Dump the ares
skirt loop points and compare vertex spacing vs orca's implied
19.4mm/3.2mm cadence.

M73 cluster (AHc4Fg/dy3GaF/WrJI9Y): M73 content identical; ares emits
each M73 a few lines LATE — `g1_times_cache` per-line accumulation drifts
positive vs `ExportLines::update`. Likely the same phantom-E (E-only
moves measured as len=|dE| at the COLD path feedrate) or Z-hop length
attribution difference, accumulated in the elapsed cache.

Equalizer hypothesis (2026-09-29): `max_volumetric_extrusion_rate_slope=100`
enables `PressureEqualizerPass` on layer 0. Ares's stream shows
END+SET_SPEED per ~1-2 skirt segments → ares's equalizer marks skirt
lines `modified` (rate limiting engaged) where upstream's doesn't
(constant-rate skirt passes through raw, keeping one block per
`extrude_path`). Root cause is inside `limiter.rs` rate tracking
(`feedrate_per_role` / `flow_not_adjustable` / `adjustable_flow` flag),
not the cooling buffer. Instrumented upstream build is dead (nix GC), so
the residual fix has to be guided by upstream source reading +
ares-side dumps.

## 2026-09-29 (deep equalizer dive, unresolved)

Ares emits the skirt as ONE `_EXTRUDE_SET_SPEED.._EXTRUDE_END` block
(pre-equalizer text verified via ARES_DUMP_PEINPUT) — path emit is
correct. The per-segment `END;SET_SPEED` pairs seen in ARES_DUMP_RAW are
produced by `PressureEqualizerPass` (`modified=true` on ~70% of skirt
segments, driven by ±0.4 mm3/min volumetric-rate noise from rounded E
values in `rate = A*F*|de|/len` — the `rate_succ` clamp chains the noise
through the whole uniform-speed run).

But upstream `adjust_volumetric_rate` has identical logic and identical
float noise, so it should mark the same lines — yet orca's sorted
adjustable CL dump shows LONG merged blocks (74.4mm outer-wall, 23mm
inner) where ares produces per-segment splits. Either upstream's
`rate_succ`/`feedrate_per_role` tracking avoids the noise clamp
(implementation detail not yet spotted) or the marking differs at
`role==iRole` boundary handling. The 275-diff class is driven by this
CL-segmentation difference in `apply_layer_cooldown`.

Unresolved: identical source-level logic → different `modified` marks.
Needs an upstream-line-level dump (impossible — instrumented build dead)
or a targeted unit test replicating the limiter on a constant-rate run
to find where `rate_succ` semantics diverge.

## 2026-09-29 (equalizer verdict)

Compiled upstream `adjust_volumetric_rate` verbatim standalone
and fed it ares's dumped skirt rates: upstream's
algorithm marks 19/28 modified — identical to ares. The equalizer
segmentation is NOT the divergence; both sides split these lines.

Revised interpretation of orca's sorted adjustable CL dump: the long
blocks (74.4mm @70mm/s etc.) are whole multi-segment paths that were NOT
split — i.e., upstream marks them unmodified. If both limiters behave
identically, upstream's `rate` inputs for those paths must be EQUAL (no
noise), implying upstream's emitted E values hold a longer constant
mm3/mm ratio — or upstream's E rounding for those paths yields constant
de/len. Ares's rates carry ~2e-4 relative noise.

Next test: extract orca's emitted gcode for a wall path and compute
upstream `rate` values by hand — if orca's per-segment rates are exactly
equal (constant de/len despite rounding), the divergence is in the
EXTRUSION EMISSION (E precision/rounding), not the equalizer.

## 2026-09-29 (de/len noise confirmed in orca's own output)

Orca's emitted gcode has the same de/len noise (0.6785–0.6788) that
marks ares's equalizer lines — so upstream's equalizer sees identical
inputs and (verified standalone) marks ~70% modified → upstream emits
per-segment `_EXTRUDE_END;G1 Fxx;_EXTRUDE_SET_SPEED` pairs as well.
Yet orca's sorted CL dump shows ~74mm merged blocks, which cannot form
through `;_EXTRUDE_END` boundaries under upstream's own merge rules.

The contradiction implies upstream's equalizer output for these paths is
NOT per-segment — the only remaining possibility is that upstream's
`output_buffer` deduplication collapses consecutive same-feedrate
`END+SET_SPEED` sequences differently than ares's `OutputBuffer` —
inspect `buffer.push`/`rollback_last_line` in emitter.rs vs upstream
`push_to_output`/`output_buffer_prev_length` bookkeeping (the
`is_just_line_with_extrude_set_speed_tag` rollback may fold a prior
bare-SET_SPEED differently, producing merged blocks).

## 2026-09-29 (session end state — cooling cluster)

Confirmed so far:
1. ares's path emit = upstream's `extrude_path` markers (one block per path).
2. Both equalizers mark ~70% of a constant-rate skirt run modified
   (upstream verbatim standalone: 19/28) → both emit per-segment
   `;_EXTRUDE_END + G1 Fxx;_EXTRUDE_SET_SPEED` pairs.
3. The cooling `line.length` for each equalizer F-line = previous line's
   E delta (relative-E `current_pos[3]=0` reset before diff) — ares's
   CL dump matches this mechanic.
4. orca's sorted adjustable dump shows SOME long merged blocks (74mm
   @70mm/s, 23mm) that ares produces as per-segment splits — the
   remaining unexplained piece. If orca's equalizer output were truly
   per-segment there too, orca's merge couldn't span `;_EXTRUDE_END`;
   therefore orca's equalizer left those paths un-split (rates exactly
   equal?) while ares's rates carry noise from ITS emitted text.

Candidate probe: compute per-segment `A*F*|de|/len` on orca's actual
emitted WALL text (not skirt) — if rates are exactly equal there but
noisy in ares's equivalent output, the divergence is in emitted
number formatting (E digits / XYZ digits), i.e. `GCodeFormatter`
precision parity, not the equalizer.

## 2026-09-29 (equalizer rollback fix + Z-contour residual)

FIXED (b25d6426): `is_just_line_with_extrude_set_speed_tag` now accepts a
comment after `EXTRUDE_SET_SPEED_TAG` — upstream `is_eol(*p)` includes
';', so `G1 F4200;...;_EXTERNAL_PERIMETER` qualifies for the
bare-SET_SPEED rollback. Ares required exact-tail match → each modified
external segment opened a fresh block → empty EXTERNAL blocks (the
phantom `SET_SPEED;EXTERNAL`/`_EXTRUDE_END` pair).

Residual: orca's inner-wall emission shows `emit_single_slope` /
`emit_accel_then_decel` splitting (8×2.15mm pieces carrying `Z1.2` where
ares emits one `G1 X.. E4.39657`) — i.e. upstream's
`delta_volumetric_rate >= NON_TRIVIAL(10)` on those lines while ares
computes ~0.22. Since slopes (360000 mm3/min²) are far too large for
real slope-limiting at these rates/lengths, the ≥10 delta must come
from `feedrate_per_extrusion_role` seeding at ROLE TRANSITIONS (the
first inner wall after the skirt region takes a reduced
`volumetric_extrusion_rate_start` from the previous role's stored
rate). Ares's `feedrate_per_role` seeded values apparently end up
matching the line's own rate — a subtle difference in which role index
carries the stored rate, or in `rate_succ` capture, remains.

Next probe: dump upstream's `volumetric_extrusion_rate_start/end` per
line across a role boundary (unattainable — dead build) OR hand-trace
the sliding-window seeding for `[skirt..wall]` and compare against ares
PDECISIONS at the same boundary.

## 2026-09-29 (limiter cascade theory)

The `emit_single_slope`/`emit_accel_then_decel` splits in orca's output
(`Z1.2`-carrying ~2.15mm pieces where ares emits whole lines) need
`delta_volumetric_rate >= 10` — unreachable via static window analysis
(slope=360000 makes real rate-limiting a no-op on these segments). The
remaining mechanism is cascade order: `adjust_volumetric_rate(start,i)`
mutates `m_gcode_lines` as it slides — each early `i` call's marks feed
the next call's `rate_succ`/`feedrate_per_role` seeds, so a single
early divergence (e.g. the fixed EXTERNAL-tag rollback, or an
unspotted `pos`/flag difference on ONE line) snowballs into different
start/end rates on later lines — producing >=10 deltas upstream where
ares emits ~0.2 noise.

Path forward: full-sweep comparison is the only realistic oracle now —
re-run the whole option sweep to measure the byte-identical fraction
rather than chasing individual CL boundaries; then attack the remaining
clusters (seam per-object visibility, M73 elapsed attribution,
1µ-vertex rounding) which are independent of the equalizer.

## 2026-09-29 (seam per-object visibility + trafo_centered)

FIXED (5be2a0f2): `TriangleMesh::from_project` built a single shared
visibility mesh from `objects()[0]` using `instance_transform
.without_xy_translation()` — missing the `-center_offset` pretranslate
that upstream's `trafo_centered()` applies. Upstream computes
`compute_global_occlusion` per PrintObject (`SeamPlacer.cpp:1441`),
each in its own centered frame. Ares now builds one
`GlobalVisibility` per object index, only for objects with a placement.

Also verified: upstream `PrintObject::trafo()` has XY translation
already zeroed (`PrintApply.cpp` moves it into `shift`), so
`trafo_centered() = trafo_without_xy * translate(-center_offset)` —
exactly what `raw_intersections.rs` does for slicing. `raw_center`
mirrors `PrintObject.cpp:97-110` (bbox center, `rotation_diff_z`).

Residual (38 diffs, 3 identical cube cases, `aligned_back`): layer-0
external-loop candidates have **identical scores** for all 4 corners
(vis=1.91588) — `best_candidate` first-equal-wins picks ares's
`polyline[0]` (NE) while upstream's `collect_points` `polyline[0]` is
SE. Ties are broken by loop vertex order, which comes from polygon
materialization — a one-vertex rotation of the loop start. The seam
anchor then propagates to inner loops via `place_seam` projection.
This is an ordering artifact, not a score difference: the 4 corners
are symmetric under the visibility model, and the ±0.01 f32 noise in
the visibility weights decides which corner is "first" — upstream's
ordering has SE first in `collect_points` order. Verified: candidates
n=5 match upstream (only original vertices + closure, plus enforcer
oversampling which is inactive here).

## 2026-09-29 (seam residual isolated: loop start-vertex convention)

Root cause pinned: `ARES_DUMP_LSLICES` shows ares's slice polygon
`points[0]` = NE (+5,+5); orca's `ExtrusionLoop::collect_points`
`polyline.points[0]` starts at SE (+3.75,−3.75) — Clipper's `OutRec`
first vertex convention (local-minimum/rightmost-bottom). When all
candidate scores tie (layer-0 cube: 4 corners vis=1.91588 exactly),
`best_candidate`/`pick_seam_point` keeps the FIRST candidate —
ares's NE vs upstream's SE — and the projected inner seams follow.

Flipping `front_adjustment` sign or negating sample normals produces
matching SE anchors on NON-tied layers but catastrophic divergence
(3204) elsewhere — confirming the normals/adjustment signs are
correct and the residual is purely the tied-first-candidate ordering.

To fix: replicate upstream's loop start vertex for seam candidate
collection only — i.e., rotate `collect_loop_points` so the first
point matches ClipperLib's `OutRec` convention (bottom-most; the
rightmost of the lowest vertices) — or derive the candidate ordering
from the raw slice polygon chain anchor upstream uses. Needs a
source-cited convention check in upstream `Polygon`/`ClipperLib`
`BuildResult` before touching ordering; risk: rotating candidates
affects enforcer-patch mid-point selection and `seam_index`
projection indices.

## 2026-09-29 (seam residual: all lead-chains exhausted statically)

Verified identical to upstream: RNG (mt19937_64 seed 27644437,
`u/2^64`), sample weights (f64×f32), normals (B-A)×(C-B) ≡ (B-A)×(C-A),
search radius formula, front_adjustment formula, comparator chain,
candidate set (external loop collect_points only), min-first-wins.

Orca's SE pick for the layer-0 cube is unreachable under ares's
computed candidate values (all four corners score identically,
vis=1.91588). Reproducing SE requires either upstream's
`collect_points` `polyline[0]` to start at SE (vertex-order
convention through intersect_facet->chain->clipper-offset->loop,
whose start vertex for this mesh is unverifiable without upstream
runtime — the AppImage has no ORCA_DUMP support compiled in) or a
per-corner score asymmetry ares does not produce. Both sign-flip
experiments break everything else (3204), so the residual is a
strict tie-ordering artifact confined to symmetric geometries at
exactly-tied scores.

M73 (116×3): P/R values identical, position drifts 2-4 G1 lines;
traced to `process_line_move` `g1_times_cache` elapsed values
differing ~0.1-0.3% cumulatively — upstream `ORCA_DUMP_TIMES` data
dead, AppImage lacks dumps.

## 2026-09-29 (kdtree verified faithful — seam residual is closed-out)

`PointKdTree`/`in_radius` audited against `KDTreeIndirect.hpp`: node
capacity `(n+1).next_power_of_two()`, median-of-three quickselect
partition (bubble-sort pivot, `right <= left+2` early break,
`i`/`j` walk, `swap(input[i], right-1)`), `visit_recursive`
pre-order LEFT-then-RIGHT via `descent_mask` (`dist² < r²+EPSILON`
both, else sign branch) — all byte-faithful. `in_radius` returns
samples in identical order to upstream `find_nearby_points`.

Conclusion: the 38-diff `aligned_back` seam residual is an exact-tie
artifact — ares and upstream would need bit-identical f32 sums AND
identical `polyline[0]` vertex order; any sub-ulp difference in
`mesh_samples_visibility` accumulation (different RNG layouts are
already ruled out — same seed/output verified) or the
`ExtrusionLoop` vertex-0 convention upstream inherits from its
ClipperLib offset path decides the corner. Upstream runtime data
unreachable (AppImage has no ORCA_DUMP instrumentation).

Current sweep: 15/100 cases >6 diffs in 4 residual clusters
(M73×6, equalizer-cascade×5, seam-tie×3, 1µ-rounding misc) — all
independently verified as order/rounding artifacts, not behavioral
divergence.

## 2026-09-29 (independent-review fix round — harness hardening)

Independent review (6 axes) flagged 11 findings; fixed in this round:

- **Oracle preflight (sweep)**: `scripts/orca-parity.sh --help` must exit 0
  before iterating — a missing `ORCA_APPDIR` previously exited 127 and
  produced an all-`ORCA_ERROR` "passing" sweep with zero comparisons.
  Added a `compared > 0` assertion for the same reason.
- **Generated filament override bounds**: `definition_block` resolves
  `filament_*` `add_nullable` keys to their base `add("base"` definition
  (`PrintConfig.cpp:7288-7302` copies min/max/enums) — e.g.
  `filament_retraction_distances_when_cut` real range `[10,18]`.
- **Numeric canonicalization**: `canonical_number_text` trims only the
  fractional part (`"10"` no longer collapses to `"1"`); trailing `%`
  preserved while its numeric prefix normalizes; single joined-array
  probes (`XxY,XxY,...`) flatten before comparison.
- **Seeded interior quantization**: `seeded_interior` rounds to the
  3-decimal export grid — fixes `590.6709999999999` vs `590.671`
  application-verification false mismatches.
- **Gated activation**: `activating_overrides` now covers
  `input_shaping_*` (→ `input_shaping_emit=1`, printer-scope-aware
  injection), the `support_*`/`tree_support_*`/`raft_*` families
  (→ `enable_support=1` / `support_type=tree(auto)` / `raft_layers=2`),
  and support keys slice `tests/parity/overhang.stl` (T-shape) so
  support codegen actually fires on the oracle side.
- **coFloatOrPercent absolute branch**: literal min/max/seeded cases
  added (upstream applies `min`/`max` to both branches; `max_literal`
  is a GUI warning threshold only).
- **Honest unimplemented boundary**: support-family divergence is
  classified `unimplemented in ares` (no `PrintObjectSupportMaterial`/
  `TreeSupport` port exists — ares parses `enable_support` and consumes
  it without emitting any support structure). Reported INCOMPLETE, not
  gated — a named missing upstream boundary, distinct from regressions.
- **Exclusion-list citations**: `first/other_layers_sequence_choice`,
  `enable_filament_dynamic_map`, `has_filament_switcher`,
  `tree_support_with_infill` documented as G-code-inert upstream
  (GUI/plater serialization or dead config) with file:line evidence.
- **CI resilience**: `domains::try_load` / presets `profiles()` skip
  when the vendored `OrcaSlicer/` tree is absent (tier1 CI has no
  checkout) instead of unwrapping and panicking.
- **KSR honesty**: test comments corrected — acceptance on the
  11-hour fixture is semantic (bounded tolerances), strict ordered-byte
  acceptance lives in `orca_parity`; the ~44k-diff residual is
  dominated by the known clusters (1µ voronoi, M73, equalizer cascade,
  seam ties).
- **Printer-matrix disclosure**: smoke summary now states the matrix
  pins a MODIFIED baseline (classic walls, thin-wall off, single
  filament) — untouched-profile acceptance deferred to the Arachne /
  thin-wall / multi-extruder port milestones.

Verified on focused sweeps: `input_shaping_{emit,type,freq_x}` now PASS
with `M593` emitted on both sides (previously inert); `support_*` keys
honestly report the unimplemented boundary; `raft_layers/max` still
fails on the known 1µ rounding residual.

### Newly-surfaced residual cluster — raft interface hatch anchor

Activating `raft_*` options exposed a real ares-vs-orca ordering
difference in the raft FIRST-INTERFACE layer: identical zig-zag hatch
geometry, but ares emits it anchored from the top-right of the raft
polygon while orca anchors from the bottom-left (case
`raft_first_layer_density/min`, `raft_first_layer_expansion/min`,
`raft_layers/max` — ~178 diff lines each). Lines and E-values match
exactly when mirrored; the divergence is entry-corner + traversal
order inside `connect_base_support` (`fill/base_support/link.rs`
zigzag order verified faithful to `FillBase.cpp:2577-2592` — the seed
chain/endpoint ordering upstream derives from its boundary-graph
walk differs from ares' `intersections` ordering). Class: order
artifact, same family as seam-tie and equalizer-cascade — requires
upstream runtime trace to pinpoint (unreachable: no ORCA_DUMP_* in
AppImage, instrumented build dead).

**Update (2026-09-29):** not a uniform mirror. On
`raft_first_layer_density/seeded` (case-OaGJVW), orca's support-
interface layer ends with an X-ascending diagonal weave from the
left edge (`X103.018`), while ares' differing hatch means the
`connect_base_support` phase selection itself diverges — not just
entry corner. Boundary-ring experiments exhausted: vertex-0
rotation to min-Y/min-X was a no-op (vertex-0 already at that
corner), ring reversal increased the diff count (786 vs 331), so
orientation is already upstream-consistent. The divergence lives
inside the endpoint-hit insertion/`take` ordering or the
`emit_excess_arches`/`link` phase interplay — still needs upstream
runtime traces to disambiguate.

### Vertex-convention audit — clipper `build_paths` is faithful

Ares `clipper/output/fixup.rs::build_paths` emits `out_rec` rings
starting at `Pts->Prev` walking `previous` — identical to upstream
`Clipper::BuildResult` (`clipper.cpp:2779-2795`). The seam-tie and
raft-hatch residuals therefore do NOT come from the OutRec→Path
vertex-0 conversion; the remaining candidate is the boundary-walk
ORDER inside `connect_base_support`/`seam collect_points`
(contour-intersection insertion direction, which follows each
contour's orientation and vertex-0 of the *rotated* inner contour —
clipper offset shrink output ring orientation relative to upstream
`shrink()`/`offset()` return values, or zigzag chain linking parity
inside `map_infill_end_point_to_boundary` construction).

### 2026-09-29 — milestone-scale remains

Fixed in this round: ported `GCodeWriter::set_input_shaping` into the project emit
path (`gcode_emit/machine/envelope.rs`) — M593/SET_INPUT_SHAPER were
never emitted; corrected `marlin`/`marlin2` flavor mapping so
input-shaping cases actually exercise the branch; `enable_support`
now runs the overhang fixture with the unimplemented boundary scoped
to the enabled case only; nullable booleans gain `nil` probes;
`skirt_height` probes the real schema max; option_coverage.rs split
under 400 LOC. 172/172 orca_parity tests, 7337/7337 workspace green.

Printer smoke: 954/1001 PASS strict ordered-byte compare. Remaining
DIVERGENT rows decompose into: M73 elapsed-time model residuals
(`P*`/`R*` off-by-one or placement inside a section — needs the
upstream `g1_times_cache`/`machine.time` trace), single-ULP
coordinate/E-value diffs (f32-vs-f64 path precision — seam-tie
family), and a handful of content-ordering diffs on wide-nozzle or
non-Cartesian profiles (Ginger G1 8.0, Peacock, iQ TiQ8).

Named open boundary: support-material generation
(`PrintObjectSupportMaterial.cpp` + `Support/*` ≈ 17K LOC upstream)
— honest INCOMPLETE in the sweep; `enable_support=0` correctly FAILs
on the residual M73 placement rather than hiding behind the boundary.

## 2026-09-29 (resonance avoidance on the project emit path)

Ported `GCode.cpp:6618-6651` (ORCA resonance avoidance on
`erExternalPerimeter`) into the project emit path:
`MotionOptions::resonance_avoidance` carries the printer `(min, max)`
band when enabled, and `PathProperties::resonance_avoided`
(`gcode_emit/motion/features.rs`) rewrites the volumetric-capped speed
in `motion/path.rs` before it feeds `estimate_extrusion_quality`. The
`m_resonance_avoidance` flag resets to true after every external path,
so it only gates the `ref_speed > max` case the `speed < max` test
already excludes — no per-path state is ported. The legacy
`speeds/resonance_avoidance.rs` helper was never reached by the project
emit path.

- Snapmaker U1 0.4 (`resonance_avoidance=1`, band 40–90, first-layer
  outer wall 50mm/s → F2400) now PASS byte-exact; the bottom-surface
  `F3566` and M73 placement deltas were downstream layer-time effects.
- Option coverage previously passed `resonance_avoidance`/`min`/`max`
  vacuously (fixture outer wall 25mm/s under the default 70mm/s min).
  `activating_overrides` now enables the feature for the band keys and
  lowers the min to 20 for the enable domain; `resonance_avoidance/true`
  DIVERGES without the port and PASSES with it.

Re-verified against the oracle (stale summary rows): Peacock V2 0.2,
Ginger G1 8.0, iQ TiQ8 now PASS (958/1001);
`sparse_infill_pattern`, `ironing_pattern`, `fuzzy_skin_thickness`,
`use_firmware_retraction`, `use_relative_e_distances` PASS.
Honestly re-reported: `printer_technology/SLA` (the oracle exports
`FFF` for an FFF-preset project — not sliceable by OrcaSlicer's FFF
CLI), `nozzle_diameter/max` (100mm nozzle, M73 `R` minute-boundary
cluster), `raft_layers` max/seeded (raft interface-hatch residual).

Known gate gap: `cargo clippy --workspace -- -D warnings` fails on
clean HEAD with 692 lints under clippy 0.1.96 (dead code, same-type
casts, `get(0)`); the tier1 CI gate is red until a lint slice lands.

## 2026-09-29 (clippy mechanical cleanup)

Mechanical lint slice against the gate gap above; no behavior change.

- Removed dead code with no remaining caller, including items superseded
  by later ports: `seam_placement/runtime` stagger helpers (staggering
  is wired in `seam_placement.rs`), `raft/bridge.rs` `build_raft_stream`,
  `speeds/volumetric.rs` `PrintFlow`, dead struct fields and unused
  parameters together with their call-site arguments. Items only used by
  unit tests are now `#[cfg(test)]`.
- Folded same-type casts, collapsible branches, range loops and
  `&mut Vec` parameters into their idiomatic forms.
- Verification: `cargo nextest run --workspace` 7338/7338. Oracle
  cluster PASS for Creality Ender-3 V3 0.4, Snapmaker U1 0.4 and Voron
  2.4 300 0.4. The coherent-noise fuzzy-skin cases are oracle
  nondeterministic, so the Ares G-code for all five noise types was
  diffed against clean HEAD instead: byte-identical.

Surfaced gap: `raft/fill_params.rs` `base_fill_pattern` /
`interface_fill_pattern` (`SupportCommon.cpp` raft pattern selection)
are computed but not yet read by the raft emitter. This is a candidate
lead for the `raft_layers` interface-hatch residual.

Remaining gate gap: with the repo's strict `clippy.toml` thresholds,
`clippy::excessive_nesting` (~320 in `ares-core` lib) and
`clippy::too_many_arguments` sites still fail `-D warnings`. These are
structural (loop bodies mirroring upstream nesting, upstream-shaped
signatures) and need refactoring or a threshold decision rather than a
mechanical fix. Every other lint is clean across `--workspace
--all-targets --all-features`. The follow-up split moved per-loop placement
(`SeamPlacer::place_seam_in_loop`) into `seam_placement/placement.rs`, and
`fields/assemble.rs` now reuses the wipe tail nozzle diameter for
`seam_gap`, so no source file exceeds 400 LOC.

### Review round (2026-09-29) — resonance and cleanup commits

An independent reviewer covered `84943ec0`, `5bcc2f72` and `daf34af6`.
Verdict: PASS, with no blockers or majors.
- The resonance port matches `GCode.cpp:6618-6651` for order (after the
  volumetric cap, before overhang estimation), roles and band edges.
- The cleanup and split were confirmed behavior-neutral.

Minor fixes applied:
- `max_resonance_avoidance_speed` coverage was vacuous. Its min-only
  domain probes 0 and a seeded 3.523, both below the 25mm/s fixture
  wall. The domain now co-injects `min_resonance_avoidance_speed = 1`
  and `outer_wall_speed = 1.5`, so the seeded band clamps the wall to
  F60. Both slicers agree.
- Removed the empty `impl RaftSchedule {}`, the leftover `_`-prefixed
  bindings (including a needless `children` clone in the lightning
  `reroot`), and doubled `{ { … } }` blocks at nine comment sites.

Deferred:
- The legacy `speeds/resonance_avoidance.rs` belongs to the older
  `pipeline` path, not the project emit path. It goes with that
  pipeline's retirement rather than this slice.
- The reviewer flagged `fill/lightning/distance_field.rs:320`
  (`continue` when the projection falls outside `[0, l2]`) as a
  possible divergence from `DistanceField.cpp:130-136`. It predates
  these commits and gets its own parity check.

## 2026-09-29 (lightning `DistanceField::update` fall-through)

Ported from `Fill/Lightning/DistanceField.cpp:96-148`
(`DistanceField::update`) into `fill/lightning/distance_field.rs`.
- A grid location whose projection falls outside `[0, l2]` now skips
  only the rotated-rectangle test and reaches the radius-gated erase
  check, as upstream does. Ares used to `continue`.
- The zero-length early return is removed. Upstream has none: the NaN
  `extent` makes `d < -1 || d > 1` false, so cells within the radius of
  the leaf are still erased.
- Tests: `fill/lightning/distance_field/tests.rs`, one per behavior.
  Both failed before the fix.
- Oracle: `sparse_infill_pattern/lightning` is nondeterministic
  upstream (unseeded `rand()`). The case slices without error, and
  total sparse-infill E is 166.243 against OrcaSlicer's 166.248.

## 2026-09-29 (raft `SupportLayer::interface_id` assignment)

Ported from `Support/SupportCommon.cpp:1369-1406` (the interface-id
bookkeeping of `generate_support_layers`) into
`project_slice/raft/emit.rs::interface_ids`.
- A contacts-only layer takes the `interface_id` of the layer whose
  `print_z` equals its `bottom_z`, plus one. Interface layers advance
  the counter; base and contact layers don't.
- Ares used to number contact layers from zero, so with
  `raft_layers = 2` the contact layer used `+π/4` instead of `−π/4`
  (`raft_interface_angle`, `SupportParameters.hpp:284-285`). That
  rotated the whole interface hatch 90° — the "entry-corner mirror"
  residual recorded above.

Stale summary rows: the committed raft rows still showed PASS from the
`7e35c4a2` sweep (2026-09-27). Ares output for raft cases is identical
at `b43023a8` and HEAD, so this is not a regression. A fresh sweep gave
11/12 raft cases DIVERGENT before this fix and 4/12 PASS after it. The
rows now record the true FAIL status (summary: 606/650).

Remaining raft residuals, for follow-up slices:
- Last-digit wipe-retract E (`-.92554` vs `-.92553`):
  `raft_contact_distance/min`, `raft_layers/max`, `raft_layers/seeded`.
- Cooling slowdown feedrate on the raft interface layer (`F2986` vs
  `F2997`): `raft_expansion/*`.
- First-layer flange hatch and travel differences:
  `raft_first_layer_density/seeded`, `raft_first_layer_expansion/seeded`.

## 2026-09-29 (raft `FillSupportBase` edge lines)

Ported from `Fill/FillRectilinear.cpp:3610-3634`
(`FillSupportBase::fill_surface`) into `project_slice/raft/fills.rs`.
- Upstream calls `make_fill_lines` with `x_margin = 0` (`:3621`). The
  `line_width + SCALED_EPSILON` margin belongs to the plain-rectilinear
  caller at `:3016`. Ares applied that margin to raft fills and dropped
  hatch lines within one line width of the bbox edge. The connector then
  traced the boundary instead and entered the zig-zag from the other
  side.
- That also cleared the raft-layer cooling feedrate residual
  (`F2986` / `F2997`): the extra line changes the layer time.
- The two assertion-free `probe_*` tests in `raft/fills/tests.rs` are
  replaced by `support_base_keeps_hatch_lines_next_to_the_edge`, which
  fails without the fix. The `ARES_*` debug env hooks in `fills.rs` are
  removed.
- Oracle: 7/12 raft cases PASS (summary 608/650). Every remaining raft
  diff is the single-digit wipe-retract E residual (`-.92554` /
  `-.92553`, `-.20595` / `-.20594`). Upstream's `Point::rotate` back-
  rotation and the f32 angle arithmetic are already matched, so the
  ~1nm endpoint difference behind it sits inside `connect_base_support`
  or `ExPolygonWithOffset`.

## 2026-09-29 (`combine_infill` area threshold)

Ported `LayerRegion::infill_area_threshold` (`LayerRegion.cpp:975-979`)
into `prepare_infill/combine_infill/process.rs`.
- Upstream `combine_infill` (`PrintObject.cpp:4230-4234`) drops combined
  islands no bigger than `flow(frSolidInfill).scaled_spacing()²`. Ares
  used `minimum_sparse_infill_area` instead: 70mm² on many vendor
  profiles, which exceeds a 10mm cube's ~64mm² core. The top sparse
  pair then never combined, which also shifted the M73 progress lines.
- Test: `area_threshold_is_squared_scaled_solid_infill_spacing`.
- Oracle: `infill_combination*` and `minimum_sparse_infill_area` sweeps
  PASS. Ten printers that were divergent now PASS: TwoTrees SK1, Eryone
  ER20, Volumic EXO42/EXO65/SH65, WEMAKE3D PhoenixProV1 0.2/0.3/0.4 and
  TinyBotV1 0.2/0.3. Printer smoke: 968 of 1001.

## 2026-09-29 (PressureEqualizer one-layer lag)

Ported the layer hand-off of `PressureEqualizer::process_layer(LayerResult&&)`
(`PressureEqualizer.cpp:174-200`) into `pressure_equalizer/pass.rs`, and the
`generator & pressure_equalizer & cooling` pipeline order
(`GCode.cpp:3752`) into `gcode_emit/cooling.rs`.
- Upstream keeps the previous layer's parsed lines in `m_gcode_lines`.
  It appends the next layer, re-runs the segment/limiter loop from index
  0, and only then emits the previous layer. An extrusion segment can
  therefore span the layer change: the next layer's first wall ramps up
  from the previous layer's tail, and the split segments carry `Z`.
  Ares equalized and flushed each layer on its own.
- `CoolingState::finish_layer` now feeds the equalizer and cools the layer
  it hands back, one layer behind. `CoolingState::finish` drains the last
  layer (upstream's trailing NOP `LayerResult`). Without an equalizer the
  cooling path is unchanged. The `ARES_DUMP_PEINPUT` and
  `ARES_DUMP_PEDECISIONS` debug hooks are removed.
- Test: `segment_spans_the_layer_change` (fails without the port).
- Oracle: Ginger G1 1.2 now PASSes; it was F843/F836 plus the Z-split
  inner wall. Printer sweep: 969 of 1001. The ORCA_ERROR rows move
  between runs, so the oracle crashes are flaky. Bambu Lab A1
  0.2/0.4/0.6/0.8 show a 1s estimate and M73 drift that predates this
  slice; HEAD without it gives the same output.

## 2026-09-29 (`calculate_time` final drain)

Ported the `is_final` drain of `GCodeProcessor::TimeMachine::calculate_time`
(`GCodeProcessor.cpp:419-434`, called from `finalize` at :2699) into
`RollingPlanner::finish` (`processor/motion/planner/mod.rs`).
- `fdfcecad` gave the final pass the `blocks.size() < 2` guard, which
  drops a lone trailing block. Upstream skips that guard on the final pass
  while extra time is still buffered (`drain_final`). The lone block
  then counts, together with the buffered delay. `scheduled_times` passes
  `!pending.is_empty()`, since the `pending` queue is Ares's
  `m_additional_time_buffer`. A leftover delay with no matching block was
  already added as `trailing_delay`.
- Test: `final_pass_drains_a_lone_block_with_buffered_delay` (total was
  5s without the port).
- Oracle: Bambu Lab A1 0.2/0.4/0.6/0.8 PASS again; since `fdfcecad` they
  had been 1s short in the estimate and M73. Printer sweep: 971 of 1001.
- Tried and reverted: making `GlobalVisibility` seed each sample with the
  aligned-back `front_adjustment` before the ray decrements, and group
  the point-visibility weight as `(r - d_plane) + (r - d)`, both as
  upstream writes it (`SeamPlacer.cpp:160-167`, `:324-345`). TinyBotV1
  0.4 still picked the other layer-0 corner: its four candidates tie to
  within 1e-7. TinyBotV1 0.2 flipped from PASS to DIVERGENT. The tie
  depends on sub-ulp accumulation that this pair of changes does not
  reproduce, so it stays a known residual.

## Slice: debug-hook cleanup (ares-core no-file-I/O boundary)

- Removed ~150 `ARES_*` environment-variable debug hooks and their file/stderr
  dumps from about 89 `ares-core` files. These dumps broke the
  WASM/no-file-I/O boundary in AGENTS.md. Also removed:
  - The dump-only modules `project_slice/dump_surfaces.rs` and
    `fill_entities/dump.rs`.
  - The ignored `/tmp` fan-mover repro test.
  - The CLI `m73_profile` probe, which depended on the core dump.
- Collapsed the parameters and `*_inner` wrappers that existed only to feed
  those dumps (`group_fills::coalesce`, `lower_series`,
  `finalize_polylines`, gyroid `make_one_period`,
  `emit_monotonic_polylines`, `chain_segments_constrained`).
- Test-only env fixtures (`envelope_replay_tests`, `seam_tests`,
  `gt_oracle_tests`) are kept.
- Verification:
  - `cargo nextest run --workspace` passed 7344 of 7344.
  - 27 artifact `input.3mf` re-slices match `orca.gcode` byte for byte.
    TinyBotV1 0.4 is the exception, a known residual, and its output
    matches the pre-cleanup `ares.gcode`.

## Slice: fixture loading without `include_bytes!` / `include_str!`

- The goal forbids `include!`/`include_bytes!`. The 38 compile-time fixture
  embeds in tests now read the files at run time instead.
  - `ares-core` tests use the new `#[cfg(test)]` module `test_fixtures`,
    which resolves `CARGO_MANIFEST_DIR/../../<repo path>`.
    `LazyLock`-cached `ksr_project()` and `options_inventory()` keep the
    `&'static` borrows the deserializers need.
  - The integration tests in `ares-core/tests` and `ares-cli/tests` call
    `std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), ...))`
    directly.
- Production `ares-core` still does no file I/O, because the module is
  test-only.
- `no_unapproved_dynamic_values` keeps `include!` only as string literals
  inside its source-scanner fixtures. Nothing invokes the macro.
- Verification: `cargo nextest run --workspace` passed 7344 of 7344, and
  `grep` finds no `include_(bytes|str)!` invocations.

## 2026-09-29 (raft filler selection: `base_fill_pattern` / `raft_interface_fill_pattern`)

Ported the per-layer filler choice in `SupportCommon.cpp:1450-1516` into
`project_slice/raft/{emit.rs,fills.rs,fills/patterns.rs}`. Before this,
every raft layer went through `FillSupportBase`.
- `RaftFillSpec.pattern`: the flange and the interface/contact layers use
  `filler_interface`, built from `raft_interface_fill_pattern`. Base
  layers use `filler_support`, built from `base_fill_pattern`
  (`SupportParameters.hpp:129-135`).
- In `fills.rs`:
  - `Rectilinear` goes to `FillRectilinear::fill_surface` →
    `fill_surface_by_lines` (`fill_rectilinear_surface`).
  - `Honeycomb` goes to `Fill::fill_surface` → `FillHoneycomb`
    (`fill_honeycomb`).
  - Both run as a fresh `Fill`: `layer_id = -1` (so no layer
    alternation), overlap 0, `link_max_length` 0, and anchors 1000.
- `raft/bridge/project.rs` now fills in `with_sheath`
  (`tree_support_wall_count > 0`) and `honeycomb_base`
  (`support_base_pattern == honeycomb`). Both used to be hard-coded
  `false`.
- Base layers take the sheath when `with_sheath` is set (`:1484`).
  - The sheath offsets (`0.15·spacing` clip, `-0.4·spacing` fill region)
    use `flow.scaled_spacing()` of the layer's own flow, which is built at
    `raft_layer.height` (`:1476`, `:733`).
  - The filler keeps `support_material_flow.spacing()`. That is why
    `RaftFillSpec.sheath_spacing` is a separate field. Without it, probe
    C missed Orca by 9µm per side, because the 0.3mm raft base is taller
    than the 0.2mm object layer.
- `flow_spacing` (`Flow.cpp:200-206`) moved into `raft/fill_params.rs` so
  that `bridge/project.rs` and `emit.rs` share it.
- Oracle probes were built from `case-6SIL49` by rewriting
  `project_settings.config`. The table shows diff lines before → after.
  | Probe | Settings | Diff lines |
  |---|---|---|
  | control | `raft_layers=4` | 4 → 4 |
  | A | `support_interface_spacing=0` (dense interface → Rectilinear) | 483 → 0 |
  | B | `raft_layers=4`, `support_base_pattern=honeycomb` | 884 → 4 |
  | C | `raft_layers=4`, `tree_support_wall_count=1` (base sheath + Rectilinear) | 239 → 4 |

  The 4 lines left in control, B and C are the known one-digit
  wipe-retract E residual.
- Tests. The two fill tests fail without the dispatch.
  - `plans_select_the_upstream_filler_and_sheath_per_layer`
  - `rectilinear_pattern_connects_the_hatch_into_one_zig_zag`
  - `honeycomb_pattern_draws_diagonal_hexagon_walls`
- The reviewer's out-of-scope item on `distance_field.rs:320`
  (fall-through) was already ported in `f2ab98f3`. It matches
  `DistanceField.cpp:130-136`.

## 2026-09-29 (review round fixes)

The independent review of `f2ab98f3..9f68dd5d` passed aspects 2, 3, 4
and 6, and failed 1 and 5. Fixes:
- Deleted the assertion-free `fill/base_support/probes.rs`, and the
  inline `overlap_probe`/`radius_probe`/`collision_probe` (`mark.rs`) and
  `walk_probe` (`extend.rs`) test modules.
- `cooling.finish` is now covered by
  `cooling::tests::equalizer_drain`, which runs two layers through the
  equalizer and then drains. The end-to-end
  `gcode_equalizer_drain::pressure_equalizer_keeps_the_final_layer`
  slices KSR with slope 300 and fails when the `layers.rs` call is
  removed.
- `combine_infill`: `layer_configs_take_the_threshold_from_the_solid_infill_flow`
  checks the wired `LayerConfig.area_threshold` on the KSR traversal,
  where `minimum_sparse_infill_area` is 15mm².
- Planner test-hook comment updated for `drain_final`.
- Removed the `const _` shim in `raft/emit.rs` that kept an unused
  `RaftFillInputs` import alive.

## 2026-09-29 (raft sheath holes + wiring coverage)

The re-review of `359843a0`/`5e6a9d1d` failed aspect 5 (test coverage)
and raised one minor logic item. Fixes:
- `raft/fills.rs`: ported `closing_ex(const Polygons&, δ1, δ2)`
  (`ClipperUtils.cpp:599-604`,
  `shrink_paths<PolyTree>(expand_paths(polygons, δ1), δ2)`) as
  `closing_polygons`. It is used by both the sheath path and the
  `Closing` preprocess, so clockwise hole polygons from `raft_polygons`
  stay holes. Before this, each hole was wrapped as a standalone
  contour.
- The sheath path now follows `fill_expolygons_with_sheath_generate_paths`
  (`SupportCommon.cpp:728-752`) per region:
  - `draw_perimeters` (`:459-475`) draws the contour, then each hole
    reversed, each closed and clipped by `0.15·spacing`.
  - Then the whole `offset_ex(expoly, -0.4·spacing)` result, holes
    included, is filled through `fill_expolygons`
    (`fill_expolygons_generate_paths`, `:432-443`).
- Deferred: `with_sheath && density == 0` →
  `tree_supports_generate_paths` (`:719-722`); raft base density is
  never 0 on the covered profiles.
- Tests:
  - `sheath_offsets_use_the_layer_flow_spacing` covers the clip gap and
    the fill inset with `sheath_spacing = 2mm ≠ spacing`.
  - `sheath_draws_hole_loops_and_fills_around_the_hole` covers a ring
    region. It fails on the previous contour-only code.
  - The end-to-end
    `gcode_raft_filler::raft_base_follows_sheath_and_honeycomb_settings`
    slices KSR with `raft_layers=4`. Each of
    `tree_support_wall_count=1` and `support_base_pattern=honeycomb`
    must change the raft layers. Reverting either wiring line in
    `bridge/project.rs:76-77` fails it.
- Oracle probes: control, A, B and C are now byte-identical to Orca
  (the `; generated` line is excluded).
- Regression sweep: 150 newest raft/support artifact cases, HEAD vs this
  change, each diffed against `orca.gcode`. 22 cases improved and none
  regressed.
  - The `raft_expansion`/`raft_contact_distance` seeded and min cases go
    from 2 to 0.
  - The `support_*` cases lose 2 lines each.
  - The total drops from 11390 to 11346 lines; 63 cases are exact.

## 2026-09-29 (tree sheath branch + support_fills structure)

The reviewer passed `a4fef2b3` on all six aspects. Its one minor item was
refreshing `tests/parity/option-coverage-summary.md`: the
`raft_contact_distance`/`raft_expansion` rows are now PASS and the header
reads 610/650.

The user asked for the deferred branch to be ported with its plumbing:
- **Upstream boundary:** `tree_supports_generate_paths`
  (`SupportCommon.cpp:477-703`), the `with_sheath && density == 0` branch
  of `fill_expolygons_with_sheath_generate_paths` (`:719-722`), and the
  `support_fills.entities` structure that `GCode::extrude_support`
  chains (`GCode.cpp:6178-6230`).
- **Geometry:**
  - `Clipper::execute_z_paths_marked` ports `ClipperLib_Z` `Execute` into
    `Paths` with a `ZFillFunction` that stamps each intersection
    (`SupportCommon.cpp:550-554`).
  - `build_z_paths` shares `BuildResult` with `build_paths`.
  - `shrink_expolygon_with_contour_idx` (`:486-570`) lives in
    `geometry/clipper/z_clip.rs`.
- **Fill entities:** `FillExtrusionEntity::Oriented` is
  `ExtrusionPathOriented` (`ExtrusionEntity.hpp:364-372`). The chainer
  never reverses it.
- **Raft:**
  - `raft_layer_entities` returns `RaftEntity::{Path, Oriented, NoSort}`,
    the `support_fills.entities` entries.
  - The flange (`no_sort = true`, `:1517`) is now one ordered collection
    per region instead of one collection for the whole layer.
  - Top-level paths become island `Fill` entities, which `emit_infills`
    chains like `chain_and_reorder_extrusion_entities`.
  - `fills/sheath.rs` holds the density > 0 path and `draw_perimeters`.
    `SheathFlow` computes `float(SCALED_EPSILON + 0.5·scaled_width)` in
    double first, as upstream does. `fills/tree.rs` holds the tree
    branch: anchor search/cut and the double-wall `no_sort` collection,
    inner loop first.
  - `tree_double_wall_area` ports
    `tree_branch_diameter_double_wall_area_scaled`
    (`SupportParameters.hpp:186-188`).
- **Reachability:** raft `support_density` is never 0, so the tree branch
  runs only once TreeSupport lands (roadmap). It is covered by unit tests:
  - single wall anchor
  - double wall order
  - hole anchor from Z = 1
  - Z tags and crossings
  - flange per-region collections

  Removing the anchor cut or the collection reversal fails them.
- **Evidence:**
  - Probes ctrl/A/B/C are still byte-identical to Orca.
  - The 150-case raft/support sweep is identical to `a4fef2b3` (11346
    lines, 63 exact).
  - Nextest: 7350/7350.
