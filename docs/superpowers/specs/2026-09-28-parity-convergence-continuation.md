# Spec: parity convergence continuation — option coverage + printer smoke residuals

## Status: ACTIVE (authored 2026-09-28)

This spec continues the bounded convergence wave defined by
`2026-09-08-parity-convergence.md` and `2026-09-26-parity-residuals-milestone.md`
on the same fixed upstream baseline: OrcaSlicer `v2.4.2`,
commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`, oracle executable
`/tmp/orca-official-242/squashfs-root/bin/orca-slicer` driven through
`scripts/orca-parity.sh` (`ORCA_APPDIR=/tmp/orca-official-242/squashfs-root`).

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
upstream build tree at
`ares-parity/2026-09-07-coordinator/wave9-upstream-boundary` has been
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
(/tmp/equalizer_test.cc) and fed it ares's dumped skirt rates: upstream's
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
