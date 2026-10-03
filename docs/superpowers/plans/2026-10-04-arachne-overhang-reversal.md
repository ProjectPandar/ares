# Arachne steep-overhang detection and loop reversal plan

Upstream/destination/included/deferred boundary: ARD-0099 and matching spec.

1. Reproduce the unchanged saved Prusa failure, then add a fresh stock tracer
   and sensitive public zero-threshold false/true complete-output control.
2. Port ordinary clipped-path width-group detection against raw lower slices;
   consume surface-local flags with the existing common reversal function.
   Remove the old gate and obsolete rejection assertion only with this path.
3. Verify internal-only, contour/hole, nonzero threshold and raft boundaries
   in small public-input slices without widening the comparator.
4. Freeze each verified slice; run official/seeded/default controls, original
   saved replay, workspace, fmt/Clippy and WASM; independent six-axis review,
   fix/re-review, then code+docs commit/push.

Original stock report has37 overhang_reverse ARES_ERROR rows. The immutable
new stock scan19358 remains live on clean0c60ab28; preserve its loaded binary,
revision and artifacts while this source slice changes. Do not relabel that
scan as current-WIP acceptance.

Saved first failure is Prusa MINIIS0.25, `case-iTetLW`: wall_generator=arachne,
overhang_reverse=1,threshold=50%,internal_only=0,detect_overhang_wall=1.
Unchanged input/reference copied to `/tmp/ares-overhang-reverse-replay.DmoY6T`.
Current0c60ab28 replay exits100,0/1 PASS,0.247s,
`/tmp/ares-overhang-reverse-stock-red.log`, paired `case-DGqe9O` records the
same unsupported overhang_reverse failure. No production edits precede RED.

Ranked predictions: (1) the obsolete config gate blocks all Arachne reversal
inputs; removing it without consuming flags would leave sensitive native
movement mismatches; (2) existing traversal flags are never consumed by the
materialization caller; (3) clipped traversal lacks width-group threshold
detection and raw lower-slice input, so wiring flags alone cannot implement
the enabled detection branch. Source readback confirms all three gaps.
Native uses open ordinary polylines; do not use the closed-polygon diff helper,
which would incorrectly add a closing chord to fragmented clipped paths.

Fresh public RED exits100: two tests fail on the obsolete rejection,
`/tmp/ares-overhang-reverse-public-red.log`; the zero-threshold native
movement sensitivity assertions pass before the Ares failure. Initial
production GREEN finishes normally: two tests PASS in4.823s,
`/tmp/ares-overhang-reverse-first-green.log`, with complete paired artifacts
for stock Prusa (`case-tjsRVc`) and false/true zero controls
(`case-G67NeE`, `case-UaC6wv`). This is focused WIP evidence, not final
matrix acceptance or a committed revision receipt.

Public boundary controls now exercise hole geometry with internal-only both
values, nonzero absolute/percentage thresholds on the overhang fixture and
the detection-disabled branch. Their seeded complete-output run is pending
in `/tmp/ares-overhang-reverse-boundaries-seeded.log`. Independent read-only
six-axis review requested; full gates and commit/push remain pending.

Boundary run54442 terminates100:0/3PASS in4.162s. The hole cases differ
already at first-layer travel byte3086/line143 (before reversal is active);
the actual overhang fixture returns the pre-existing detect_overhang_wall
clipped-width interpolation guard. Keep these full-output assertions, and
add an explicit reversal-disabled hole control to separate baseline geometry
from this slice; no comparator waiver or unsupported-success classification.

Independent source review identifies two mandatory corrections: upstream
PerimeterGenerator.cpp:80 uses bbox filtering with get_entire_polygon=true,
and :502–514 additionally marks the contour/hole flag when final chained
paths contain OverhangPerimeter. Both are implemented for renewed focused
testing: expose the existing bbox mode while existing callers remain false;
scan actual output roles independently of ordinary-path threshold detection.
The traversal crossed400LOC during correction, so split its clipping/start
selection into traverse/overhang.rs without macros. New tests and review
corrections run in `/tmp/ares-overhang-reverse-review-fixes-seeded.log`;
none of these pending results constitutes final acceptance or permission
to discard the three failing boundary controls.

Renewed focused run26404 terminates100,3/4PASS in12.516s. Stock Prusa and
zero-threshold movement tracer still pass; all five supported cube boundary
cases match complete native output. Reversal-disabled hole control fails
with the identical first difference3086/143 (case-xIKjEv), proving the hole
difference is independent of enabling reversal in the current WIP. This
does not yet prove its baseline-commit provenance. Retain the failing case
and diagnose baseline geometry separately before claiming parity. Independent
static re-review closes the two source corrections, with no new mandatory
static issue; public actual-overhang/hole/raft coverage and full gates remain
open. Clippy76236 is running, not yet an acceptance receipt.

Baseline provenance now verified without checkout or restarting stock scan:
copy the immutable loaded test ELF from `/proc/2432631/exe` to
`/tmp/ares-hole-baseline-replay.Suegp1/baseline-tests` (SHA256
f39b9e368b511014ceca4396829d2b79366ed4ab1a61d674a94af6beebd60e13),
which the stock process loaded from clean0c60ab28. Replay the exact saved
reversal-disabled hole input/native reference. Terminal result101,0/1PASS,
0.34s, `/tmp/ares-hole-baseline-replay-0c60ab28.log`, case-U4gYd1 reports
the identical3086/143 discrepancy and lengths111441/111581. Therefore this
hole geometry mismatch predates this slice; it remains a real parity gap,
not a waived assertion. Clippy found one new test-helper argument-count
warning; grouped the case option values to remove it without lint suppression.
Fresh lint/test verification of that test-only adjustment remains required.

Next vertical slice: the existing public actual-overhang RED exercises
clip_extrusion at real crossings. Ranked predictions: (1) width sentinel
production, not reversal itself, causes detect_overhang_wall rejection;
(2) native cast timing (truncate delta before adding bottom width) matters
at variable widths; (3) clip-owned zero endpoints need native nearest-subject
projection repair, independently of the crossing callback. Port the callback
only for width execution, preserve existing tagged/marked contracts, and
rerun the unchanged public assertion before any broader geometry changes.
Remove the two new traversal nesting warnings using flag accumulation;
do not add lint allow attributes.

Initial WIP Clippy76236 has now terminated0 (2m13s): corelib255/coretest271,
CLI32 warnings. The two additional core nesting warnings and one CLI helper
warning have source corrections pending fresh lint readback; do not call
this a no-new-warning acceptance gate. Width implementation's first focused
run9807 is compiling, `/tmp/ares-overhang-width-first-green-seeded.log`.
Independent source readback confirms callback/repair arithmetic and mode
cleanup; requests subject-before-clip AddPath order and stale sentinel
comment correction before final acceptance. Keep all failed public assertions.

Width run9807 terminates100,3/4PASS in13.312s. Original stock/zero/five cube
controls stay complete GREEN; actual overhang passes the old width guard
but now panics at seam_placement/placement.rs:60's obsolete MultiPath
unreachable scaffold. Preserve that RED. Source GCode.cpp5773,6087–6100
proves only extrude_loop places a seam; leave open MultiPaths to existing
normal emission. Extend the included boundary in ARD before correcting
dispatch, and restore source subject-before-clip addition order. This is
not a skip of extrusion or a comparator waiver.

Dispatch run12308 terminates100,0/1PASS in2.698s. The same real-overhang
tracer now reaches gcode_emit/motion/perimeter.rs:59's second obsolete
MultiPath unreachable. Source GCode.cpp6038–6085 supplies ordered `_extrude`
dispatch and the complete joined/reversed wipe path. Extend the cited
destination before replacing the emitter scaffold, including MultiPath
segments in the existing touching-lines geometry view. Adaptive PA's weighted
flow lifecycle remains explicitly deferred; no claim about adaptive presets.

Emission run19555 terminates100,3/4PASS in9.138s. Actual overhang now emits
complete bytes, DIVERGENT at8799/377: native starts the open path at
114.875/114.817, Ares at114.896/114.854; subsequent points show opposite
traversal of the same path. Existing select_open_start explicitly uses
first-insertion order rather than source unordered PointHash traversal,
and its overhang fallback retains the first instead of the source last.
Replace this scaffold with the already implemented selected-reference map
order, sharing its geometry owner with the existing Tree caller. Do not
simply reverse all paths or hardcode the observed endpoint. Retain full
output comparison; later speed/fan differences remain to verify.

Original clean0c60 stock sweep19358 terminates/reaps100:1303.811s,1001
outcome rows/784PASS/217incomplete (39ARES_ERROR,101DIVERGENT,67ORCA_ERROR,
10VENDOR_INCOMPLETE), `/tmp/ares-stock-matrix-0c60ab28-seeded.log` and
unchanged stock-printer-summary.json. This fixed-discovery receipt is not
current WIP acceptance; all paired artifacts and producer revision remain
unchanged. Do not restart on timeout or relabel it as the new source.

Width WIP Clippy13823 terminates0 in1m22s: corelib253/coretest269/CLI31,
back to the unchanged baseline with no added warnings. This receipt predates
the shared map move and is not its final lint gate. First endpoint build
fails101 before tests due a mechanical duplicated closing block in the
moved map's into_ordered function; fix the parse error and keep that log.
Renewed build28049 uses the corrected source and a separate log
`/tmp/ares-overhang-endpoint-green-seeded-parse-fixed.log`; outcome pending.
The original map insertion/growth/iteration and Copy payload behavior stay
unchanged for Tree. No endpoint coordinate or wholesale path reversal is
hardcoded; the source's last-overhang fallback is now also consumed.

Endpoint run28049 terminates100,0/1PASS in1.972s. Native/Ares now agree
on the initial open path's start, moves, extrusion, fan and slowed speed;
full-file assertion remains RED at9249/399 (progress placement), with later
closed triangular-loop seam movement differences visible at Z1.4. These
later moves can affect whole-file estimated-time/progress placement, so do
not normalize M73 or fix a progress position independently of geometry.
Add a reversal-disabled actual-overhang control to isolate the remaining
seam behavior; run82290 in `/tmp/ares-overhang-reversal-disabled-seeded.log`.

The first three-style Tree run65505 was not real acceptance: its explicit
ARES_PARITY_TREE_SUPPORT_REFERENCE opt-in was unset and all cases returned
without comparison. Keep that log honest. Fresh enabled run59293 terminates0:
3/3 complete PASS in5.084s, `/tmp/ares-shared-point-map-tree-active-seeded.log`,
including retained paired Slim/Strong/Hybrid files (Hybrid case-ou9pGu).
Independent static map re-review finds no mandatory fix and confirms Tree
algorithm preserved; final lint/full-workspace/WASM remain pending. New
Clippy24846 includes the moved map and starts after the new test build.

Reversal-disabled overhang82290 terminates100,0/1PASS in1.951s with the
same9249/399 first difference. Thus the remaining body/seam discrepancy is
not fixed by toggling reversal; keep both public controls, do not declare
geometry parity from the repaired first open wall. Source follow-up:
GCode/SeamPlacer.cpp:485 owns region->flow(frExternalPerimeter).width(),
while alignment/preparation.rs derives flow_width from the first emitted
external path (variable-width Arachne). This is a source contract gap with
consequences for unsupported distance and inter-layer alignment (source
:1087,:1092,:1097,:1203); investigate the already available per-region nominal
Flow before modifying seam selection. No hardcoded corner or progress-line
normalization is authorized.

### Nominal region Flow seam dependency

The previous status-only goal turn verified the already integrated main but
made no implementation progress; resume the existing complete-output RED.
LayerRegion.cpp:21–28 resolves the actual layer's height/first-layer Flow;
PerimeterInputRecord.ext_perimeter_flow is that canonical Rust owner.
Pair island perimeter entities with perimeter_regions, then use the matching
RegionLayerIndex; retain source_order, collection association and scoring.
Independent static six-axis readback confirms region/raft indexing and
compatible-region ownership, with no must-fix; it does not prove the tiny
triangle root cause. Require fresh Classic/multi-region/raft controls before
final acceptance. No comparator or fixture changes.

Pre-flow shared-map Clippy24846 terminates0 in1m32s,253lib/269test warnings,
unchanged baseline. Flow-source run17608 builds/runs five strict seeded/CPU0
tests including reversal-on/off overhang, disabled hole, zero threshold and
supported range boundaries; log /tmp/ares-seam-region-flow-seeded.log and
paired files /tmp/ares-seam-region-flow-seeded. New Clippy41636 is queued
behind this build, /tmp/ares-seam-region-flow-clippy.log. Keep results pending
until terminal readback; no commit or completion claimed.

Flow run17608 terminates100:5run/2PASS/3FAIL,10.209s. Seven cube complete
comparisons pass; hole remains3086/143, overhang on/off remains9249/399.
An unchanged first difference alone was insufficient diagnosis: comparing
the prior case-kYBGIl against new case-Z10UiO shows the Z1.4 triangle now
starts114.81/114.81, emits the native F1187 and exact wipe. Both new native
and Ares Z1.4 sections agree. Thus nominal Flow fixes that seam dependency,
not the entire file. Remaining genuine geometry appears at Z10.2: Ares
omits an Overhang wall emitted by native near124.866/114.743. Whole-file
filament696.56vs696.53 and time16m25svs16m23s remain RED. Do not normalize
progress/totals; investigate upstream Arachne generation/clipping owners.

Clippy41636 terminal0,2m17s,253corelib/269coretest/31CLI unchanged; no new
nesting warning from the Flow lookup. Reviewer confirms the Z1.4 movements,
not whole-layer byte equality (one M73 placement still differs).
Detection-off public control99960 terminal100 in1.945s,0/1, byte9371/404:
/tmp/ares-arachne-overhang-no-detection-seeded.log, paired case-Zr4CWk.
The same Z10.2 corner simplification/missing short open path persists without
clipping, and Z10.4 adds an extra tiny closed loop. This falsifies clipping
as the sole cause; follow the source WallToolPaths/SkeletalTrapezoidation
generation and simplification boundary next. The comparator remains strict.

Now remove the obsolete private crossing-Unsupported assertion, which pins
the old missing ZFill scaffold rather than correct movement. Keep all public
crossing complete-output controls RED until their geometry is repaired, and
keep covered-wall behavioral coverage. PerimeterGenerator.cpp:351–359 owns
the internal positive-width assertion; replace the obsolete production
Unsupported guard with debug_assert, not a fallback or silently skipped path.
Final gates must be rerun after this source change.

Independent readback confirms no-detection excludes clipping as the sole
cause, not that SkeletalTrapezoidation is proven guilty. Inspect target lines
after generation, after simplify and before materialize to find the first
missing stage. Source follow-up finds outline.rs currently uses one even-odd
union in place of WallToolPaths.cpp::fixSelfIntersections (264–309, called
twice in generate:497/501), and combined collinear/degenerate cleaning differs
from source functions. These are candidate source gaps, not yet established
causes; do not implement speculative bulk cleanup before stage evidence.
Reviewer also accepts obsolete guard/test cleanup as source-contract repair,
but declines checkpoint approval while actual geometry is RED.

Post-cleanup native workspace24838 is /tmp/ares-arachne-flow-workspace.log;
quality73458 is /tmp/ares-arachne-flow-quality.log; wasm32 core/adapter16996 is
/tmp/ares-arachne-flow-wasm.log, in separate target to avoid native lock.
Native no-reference workspace cannot establish Orca parity: explicit paired
seeded RED receipts above remain authoritative for output. Read terminal
handles before reporting any gate complete. No staging/commit/push yet.
WASM16996 now terminal/reaped0,32.86s: core and browser adapter compile
for wasm32-unknown-unknown. This is compile-only, not browser/runtime parity.

### Target-corner stage evidence and stale rejection cleanup

Quality73458 terminal/reaped0,1m21s,253corelib/269coretest/31CLI unchanged.
Native workspace24838 terminal100: cancelled on the obsolete private
post_perimeter_boundary_rejects_out_of_scope_arachne_dispatch_typed assertion.
It expects Unsupported detect_overhang_wall but the implemented clipping
consumer succeeds, so err().unwrap() panics. Remove only that old scaffold
assertion and its orphan import/CONFIG; retain allocation-forwarding tests
and every public complete-output assertion. This is not a motion regression
waiver. The cancelled workspace is not final acceptance.

Probe36901 terminal100,1.979s,0/1 public no-detection complete RED; paired
case-T62owH in /tmp/ares-arachne-stage-probe-seeded, log with same prefix.
The target is local x14.5..15.5/y4.1..5.5 (instance origin110/110).
Input corner points include14957080/4957080,14664482/4957080,
14801310/4911471,14776255/4757079; prepared replaces14664482/4957080
with14727466/4936086. Raw outer lines already contain only14775001/4575000
and14775001/4157077 at this corner: the native protrusion is absent before
stitch/simplify/materialize. The next input corner generates the extra tiny
six-junction loop before simplification. This narrows investigation to
outline preparation or raw generation, not downstream removal alone.
No claim of exact native prepared-outline equality is made. Independent
review confirms non-equivalent fixSelfIntersections/cleaning contracts; first
isolate the outline stages before choosing a production fix.

Temporary target-only eprintln probes were removed immediately after capture.
wall_toolpaths.rs diff is empty and the instrumentation audit has no matches.
No probe or comparator modification will be committed.

Final cancelled24838 receipt:6131/7612run,6128PASS/3FAIL/9skip,183.881s;
1481 not run. All three failures require old Unsupported detect_overhang_wall:
the predecessor assertion above, the Arachne row inside classic/preflight's
deferred-branch list, and its raft-plus-Arachne error assertion. Remove the
obsolete row and raft error test as well; preserve the counterbore rejection
and existing actual wall/raft behavior coverage. Dedicated Arachne+raft
complete native coverage remains a pending domain, not waived or counted PASS.
Do not use the huge unwrap_err byte dump as a log excerpt.

Repaired full no-fail-fast43230 (/tmp/ares-arachne-flow-repaired-workspace.log)
and quality91364 (-repaired-quality.log) began before the last two stale
assertion edits; their snapshots need terminal verification and cannot alone
be final acceptance of subsequent source changes. No probe remains.
Independent six-axis readback accepts all three stale-rejection removals,
confirms probe removal and the outline/raw scope limitation. It explicitly
does not approve a checkpoint or Arachne+raft parity. Counterbore is now a
single accurately named rejection test, without the obsolete one-row loop;
its assertion is unchanged. Keep the real output RED and await gate handles.

### User-approved finite checkpoint boundary

User requests checkpoint closure before pursuing outline/raw generation.
Freeze current source: clip_extrusion crossing-width/zero projection rewrite,
shared PointHashMap with unchanged Tree ordering, its Arachne open-endpoint
consumer, MultiPath dispatch/emission, steep flags/reorientation and nominal
layer-region seam Flow, plus public controls and obsolete rejection cleanup.
No outline.rs or raw-generation algorithm changes are included. Strict hole
and actual-overhang full comparisons stay RED as retained next-slice tracers;
finite cube/Prusa/Tree GREEN is not all-option or all-printer completion.
Obtain frozen final workspace/clippy/fmt/WASM and focused complete reference
controls, then independent six-axis checkpoint approval before commit+push.
Commit code/tests/docs together; explicitly state complete output still RED.
Next slice owns source-cited WallToolPaths outline/raw parity, not another
unbounded extension of this held checkpoint.

The proposed maximum-deviation diagnostic cannot use an epsilon<1 value:
PrintConfig.cpp:7253–7263 constrains it to0.005..0.05; do not introduce an
out-of-range option as an acceptance test. Legal boundary/interior controls
belong to the next source-cited outline slice.

### Frozen checkpoint terminal receipts

All gates below use the frozen staged Rust source; only this receipt is added
afterward. Workspace58813 terminal/reaped0:7610/7610PASS,38slow,9skip,
274.318s, /tmp/ares-arachne-checkpoint-frozen-workspace.log. This run has no
native-reference optins and is not itself complete-output parity evidence.
Clippy46308 terminal0:253corelib/269coretest/31CLI warnings, unchanged;
/tmp/ares-arachne-checkpoint-clippy.log. fmt --all -- --check and git diff
--check succeed; fmt log is empty. WASM30829 terminal0,37.18s,
/tmp/ares-arachne-checkpoint-wasm.log:core and adapter compile-only, not
browser execution or all-platform parity.

Seeded50085 terminal0:8/8PASS,31.473s,
/tmp/ares-arachne-checkpoint-seeded.log. Tree and support-modifier reference
optins were enabled, but the selected Classic higher-density function lacked
ARES_PARITY_REGION_PERIMETERS_REFERENCE and returned early: its initial
green is not parity evidence. Enabled rerun80455 terminal0:1/1PASS,10.153s,
/tmp/ares-arachne-checkpoint-seeded-classic.log and paired artifact root with
the same prefix. Seeded reference is the separately built seed-replay2.4.2,
seed0,CPU0; no comparator change.

Official17011 terminal100:1PASS/7FAIL,0.485s,
/tmp/ares-arachne-checkpoint-official.log. The direct binary could not load
libpangoft2-1.0.so.0 (exit127); no G-code comparison occurred, and the sole
PASS was the disabled Classic function. Do not classify this as a geometry
regression or successful official verification. Corrected run32840 uses
the existing /tmp/orca-wrapper-tree-singlecpu.sh runtime-library launcher,
ORCA_APPDIR=/tmp/orca-official-242/squashfs-root, all three explicit reference
optins, untouched official2.4.2 at CPU0:terminal0,8/8PASS,10.896s,
/tmp/ares-arachne-checkpoint-official-enabled.log and paired artifact root
/tmp/ares-arachne-checkpoint-official-enabled. Official executable SHA256 is
64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60.
CPU affinity does not seed the official binary; previous default-multithread
results remain preserved. The finite controls cover zero-threshold actual
move reversal, supported cube boundaries, stock PrusaMINIIS0.25, Slim/Strong/
Hybrid Tree, support modifiers and Classic higher-density regions. Several
functions compare multiple cases, not just one output per selected function.

These receipts authorize only the finite checkpoint after independent review.
Strict hole and actual-overhang comparisons recorded above remain RED; no
test fixture, comparator tolerance or full-output assertion was relaxed.
Outline preparation/raw generation, Arachne+raft and AdaptivePA remain deferred;
the all-printer/all-option goal is not complete. All staged Rust files remain
below400LOC; no temporary instrumentation or source-splitting include macro
is introduced. Code, tests, spec, plan and ARD are committed together.

Saved-original replay uses the unchanged input/reference in
/tmp/ares-overhang-reverse-replay.DmoY6T:terminal0,1/1 strict completePASS,
nextest0.506s (outer command0.753s),
/tmp/ares-arachne-checkpoint-saved-original.log, case-vp4eNI under the same
artifact prefix. Fresh-stock output is not substituted for this gate.

Official default-thread71310 terminal/reaped100:8run6PASS2FAIL,3.634s,
/tmp/ares-arachne-checkpoint-official-default.log. Retain failures:
Slim case-AghWdE byte1379/line61, expected384266/actual383343 normalized
bytes; Hybrid case-4GN3FF byte499/line19,451907/450835. These are not waived
by CPU0 GREEN. To isolate regression, run the immutable clean0c60ab28 loaded
test ELF (SHA256f39b9e368b511014ceca4396829d2b79366ed4ab1a61d674a94af6beebd60e13)
against those exact two saved3MF/native pairs; baseline terminal101,0/2PASS,
/tmp/ares-arachne-checkpoint-baseline-default.log and replay-summary.json.
Baseline cases77YuhP(Slim)/X5EckN(Hybrid) reproduce identical first differences.
Raw Ares output cmp is equal old/new for BOTH pairs, without normalization:
Slim SHA2567e4571a048dd9d3e51e1b56a4508516b039241ddff038b55c2927a32bd104be1;
Hybrid SHA2566f9a9ab3bf60d7987356befe5106df6c6a0453e9baa21a409ac4fac3939ebe1a.
Therefore the shared-map extraction and this checkpoint do not introduce
these default-reference divergences. This baseline control alone does not
prove the precise upstream race; prior multithread uncertainty evidence is
preserved separately. Default-thread execution was performed, not deferred
or reported GREEN. Its pre-existing divergences remain open evidence alongside
the approved independently reproducible CPU0 reference, with strict comparison
unchanged. The finite checkpoint is distinct from whole-slice acceptance.
