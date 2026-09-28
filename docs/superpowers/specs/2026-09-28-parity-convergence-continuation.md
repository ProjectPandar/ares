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
