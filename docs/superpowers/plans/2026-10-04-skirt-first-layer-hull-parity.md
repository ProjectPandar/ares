# Skirt first-layer occupied hull implementation plan

Source boundary and included/deferred scope: ARD-0098 and
[spec](../specs/2026-10-04-skirt-first-layer-hull-parity.md).

1. Add the unchanged-stock public Kobra 3 Max whole-output test; confirm RED
   at the same bounds as saved `case-QaBPWD` before production edits.
2. Distinguish native export drift, offset-hull versus simplified loop bounds,
   and numeric conversion order using the tight native loop.
3. Port the separate final occupied-hull offset into `SkirtPlan`, preserving
   emitted paths and the upstream scaled float operation order.
4. Verify saved failure replay, all three native reference domains, existing
   controls, workspace Nextest, fmt/Clippy and WASM compilation.
5. Independent read-only six-axis review, fix/re-review, commit/push together.

Original immutable stock scan ran on its pre-review WIP executable;
do not relabel it as repaired-discovery or current-source acceptance.
Its paired failure is `/tmp/ares-stock-matrix-eb65bca3-wip-seeded/case-QaBPWD`:
first difference byte758/line26, first-layer min/max/size only beyond the
allowed generator identity difference. Preserve this original input/reference.

## RED and diagnostic predictions

Public stock test `/tmp/ares-skirt-hull-red.log` exits100/7.720s. All three
seed0/CPU0 reference attempts have the same byte758/line26 difference:
`/tmp/ares-skirt-hull-red/case-{YD3IRy,OmJwp7,NOyRrs}`. Expected min is
203.951,200.951; actual is 203.929,200.929. Max and size also differ; the
body does not. This unchanged stock cube isolates the footprint without
changing the original geometry or export settings.

Ranked predictions: (1) porting the separate occupied-hull offset fixes the
bounds without changing emitted skirt paths; (2) scaled-float ordering causes
the difference, requiring upstream float conversion order rather than f64
padding; (3) export/reference drift, weakened by identical three attempts and
the saved earlier scan. Upstream `Print.cpp:2773–2775` offsets the original
hull by final distance plus half scaled spacing without centerline simplify.
Ares padded the simplified centerline bounds instead. The patch retains the
separate offset result by ownership, using the existing scaled f32 values;
the bounds consumer uses its points. No path output or comparison exclusions
change. The first experiment failed unchanged, exit100/7.784s at the identical
bounds, `/tmp/ares-skirt-hull-green-seeded.log`. It had not reached that path:
saved native settings have skirt_loops=0. Auto brim is configured, but no brim
paths are emitted. `Print::has_skirt` (583–586) is height-only, so native still
executes the zero-loop final occupied offset. The Ares early return had bypassed
it and footprint code had substituted scalar bbox padding. The corrected slice
retains occupied geometry for zero loops, gives empty extrusion collections
zero emission layers, prevents minimum-length logic creating loops from zero,
and removes the old scalar padding fallback. The abandoned trial's gates are
not final-snapshot evidence. Re-run every gate on the corrected snapshot.

Independent static review found two required boundary repairs: footprint
incorrectly collected every layer for enabled shield with zero loops, and the
new zero-loop path activated the old multi-object per-object rejection. Public
two-object controls (one 4mm cube, one 10mm widening mesh, both native-sliced)
confirmed both reject paths RED: `/tmp/ares-skirt-hull-review-red.log`,
exit100/10.996s, 0/2 PASS, `skirt_type per-object` ARES_ERROR. They preserve
strict full comparison and verify all requested native config values, positive
min_skirt_length with zero loops, object positions/heights and widening mesh.

Repairs: first-layer islands are first-layer-only (`Print::first_layer_islands`),
while the skirt collector alone owns the finite-height/global-z domain. Respect
no-skirt height before the positive-loop per-object guard. Port zero-loop
object hull grouping/round envelope intersections to a small `skirt/groups.rs`
source-boundary module; keep separate object hulls until collision merging
stabilizes. The finite-shield control places the wider object at X109 beside
the X95 short cube: occupied skirt envelopes overlap, while model solids do
not during the short object's height. No-skirt control keeps X125 separation.
Positive multi-object per-object emitted skirts remain an existing gap, not
a new rejection or claimed acceptance in this zero-loop slice.

The first boundary repair run did not validate equivalent imported geometry:
the Ares widening mesh was correct, but native imported the other object's
cube. UUID and source-file changes did not fix this (identity-check exits100,
0/2 PASS). `Format/bbs_3mf.cpp::_generate_volumes_new:4929–4963` defaults
`shared_mesh_id` to the integer child object ID and consults a global mesh map,
even for different child paths. Both child meshes used ID1. The new zero-loop
controls now give the widening mesh ID4 consistently in its resource, root
component and part metadata; the existing positive-loop fixture remains
unchanged. Preserve taper geometry and complete-output assertions. This is
input identity repair, not a geometry waiver; fresh native validation follows.

The zero-loop collision group port computes each round's envelopes once,
unions all intersecting pairs, then rebuilds hulls before the next round,
matching the upstream fixed-point sequence. Use only the first offset polygon
as upstream does. `ClipperUtils.hpp:331` declares polygon offset delta as
`float`, so the scaled integer grouping distance is converted to f32 at that
same boundary rather than silently changing upstream precision.

## Frozen-snapshot verification

The distinct-child-ID diagnostic exits0:2/2 complete controls PASS in3.932s,
`/tmp/ares-skirt-hull-distinct-id-seeded.log`. No production changes follow.
Final focused native runs each include stock Kobra, stock Ender, original
unequal-height positive skirts, two zero-loop boundary controls and the
existing skirt-height domain test (its legal variants compare whole output;
its negative control validates rejected input).

- Seed0/CPU0 process72473 exits0,6/6 PASS,20.633s;
  `/tmp/ares-skirt-hull-accepted-seeded.log`.
- Unmodified official CPU0 process48248 exits0,6/6 PASS,14.336s;
  `/tmp/ares-skirt-hull-accepted-official.log`.
- Unmodified official default threads process80862 exits0,6/6 PASS,4.265s;
  `/tmp/ares-skirt-hull-accepted-default.log`.
- fmt process91347 exits0, and diff-check exits0.

Workspace, Clippy, WASM compilation and saved-original replay are still
running or waiting for the shared build lock. Their final terminal receipts
and independent six-axis readback are required before checkpoint commit.
These focused passes do not close the218 old stock-scan residuals or all
Option values; the original Option scan was live at that earlier readback.
Both original broad scans have since terminated incomplete; their terminal
receipts are recorded in the stock-printer-sweep plan.

Independent static re-review withdrew the f32 concern after reading the
actual upstream offset signature, but required the source short-circuit for
objects already in the same collision group (`Print.cpp:2887–2893`). Move
root comparison before Clipper intersection; otherwise fully intersecting
N-object groups perform N(N−1)/2 intersections instead of N−1 in that round.
This preserves traversal/union order. Freeze this correction and re-run all
gates as `/tmp/ares-skirt-hull-reviewed-*.log`; preceding accepted-prefix
receipts are diagnostics for the pre-correction snapshot, not final evidence.

Clippy's accepted-prefix job exits0 but exposes three new excessive-nesting
warnings in the collision pass (core256/272 versus baseline253/269). Extract
that pass into a private `merge_collisions` function, preserving loop order,
same-root short-circuit and intersection/union operations. No suppression.
All reviewed-prefix jobs were still waiting for the shared build lock before
this extraction; their builds therefore consume the corrected source, not
the earlier nesting snapshot. Preserve those existing handles. Final source
group module SHA256 is
f554ad1d7d13a32a8fff951c00abc0381fbb198fd853bf5a83aeb76ef9b4ae5a.
Re-review the extraction and require the warning counts to return to baseline.

Current helper snapshot's final native processes all terminate/reap0:
seeded49308,6/6 PASS32.147s; official CPU0 37395,6/6 PASS23.681s;
official default74761,6/6 PASS10.725s. Logs and paired artifacts are under
`/tmp/ares-skirt-hull-reviewed-{seeded,official,default}`. Final fmt43237
exits0, `/tmp/ares-skirt-hull-reviewed-fmt-final.log`, staged diff-check0.
Independent extraction readback confirms no operation-order, allocation,
copying or error-propagation changes. Final workspace91365,Clippy83219,
WASM8749 and original saved replay68291 are still live; do not approve or
commit until they terminate and the reviewer reads back their receipts.

Final Clippy83219 terminates/reaps0:253 core library /269 library-test /31 CLI
parity warnings, baseline restored; no groups.rs warnings or suppressions.
`/tmp/ares-skirt-hull-reviewed-clippy.log`,11m13s including shared-lock wait.
Workspace, WASM and saved replay still require terminal evidence.

Final workspace91365 terminates/reaps0:7604/7604 PASS,36 slow,9 skipped,
263.224 seconds, `/tmp/ares-skirt-hull-reviewed-workspace.log`. KSR initialized
ordered-byte output and spiral-vase/avoid-crossing boundary regressions pass.
Final WASM8749 terminates/reaps0:12m26s including shared-lock wait,
`/tmp/ares-skirt-hull-reviewed-wasm.log`; compilation only, not browser or
Windows/macOS runtime parity. Final saved-original replay68291 exits0:
1/1 complete PASS,0.506s, `case-w3fYOA` and original input/reference retained
under `/tmp/ares-skirt-hull-reviewed-saved-replay`. No comparator changes.
All final native, workspace, fmt, Clippy, WASM and replay gates are terminal;
request independent final six-axis checkpoint approval before code+docs commit.
The previous accepted-prefix workspace67888 also exits0(7604/7604,264.223s),
but is not substituted for this current-helper final snapshot.

Independent final six-axis reviewer reads back the staged11paths and all
terminal gates, closes every mandatory finding and approves code+tests+docs
commit and feature-branch push. Approval excludes main merge, browser runtime
claims and full printer/Option completion. Source hash and comparison contract
remain unchanged; final staged diff/fmt checks are clean.
