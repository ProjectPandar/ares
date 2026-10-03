# Region-role tool emission implementation plan

## Current slice: modifier ninth-layer waypoint

Previous turn made verified progress:63748b6b is pushed and current worktree
is clean. Exact prebuild exit0; fresh original seededCPU0 full comparison
41590 terminal/reaped100,3.368s, /tmp/ares-modifier-route188-original-red.log.
Baseline PASS/modifier26707/1293 persists. Case-4onXcp shows printingZ1.48,
liftZ1.88: matching G3 and first travel X166.33 Y104.27, then Ares omits
native X165.821 Y104.78 before matching X165.6 Y105 and later waypoints.
Approved public seam remains saved3MF to byte-output, not private geometry.
Add native-first complete ninth-layer approach tracer, run RED, then vary
only reduce_crossing_wall to distinguish routing from downstream emission.
Do not change original fixture/full comparison. Rank falsifiable hypotheses
only after this feedback loop and control verdict; no production fix yet.
Source boundary is GCode.cpp::travel_to and
GCode/AvoidCrossingPerimeters.cpp::travel_to, perimeter routing and its path
simplification consumers. Rust destination is gcode_emit/motion/path and
path/avoid_crossing. Include only output-evidenced waypoint contract; defer
unmeasured routing/numeric changes, inward arithmetic and later modifier
gaps. Existing adapters are temporary upstream compatibility shells.
Require source-cited spec/ARD, focused/full E2E gates, platform/browser gates,
six-axis independent review, probe cleanup and logical code+docs commit/push.

Tracer prebuild91777 terminal/reaped0,11.83s; runtime57050 terminal/reaped100,
2.646s, /tmp/ares-modifier-route188-tracer-red.log, case-1DPBuu.
Native literal precondition PASS; complete ninth-layer approach differs only
by the omitted waypoint. Independent preflight requires exact next-layer and
unretract delimiters, avoiding scans into later layers; apply this before the
one-variable routing-off control. No production code changed.

Control prebuild18266 terminal/reaped0,11.60s; runtime87413 terminal/reaped100,
5.296s, /tmp/ares-modifier-route188-control.log. Delimiter-tight tracer remains
RED; routing-off full approach PASS and native omits the disputed waypoint.
Only reduce_crossing_wall changes; its whole file retains earlier M73 RED.
Ranked predictions: initial routing lacks the point (inspect pre-simplification);
simplification removes it (inspect before/after retained points);
emission drops it (compare final route to raw output). Target-only temporary
route boundary probe distinguishes these, with no acceptance reading logs.
Source FirstIntersectionVisitor uses Geometry.hpp::segments_intersect, whereas
current crosses_boundary reuses coordinate-producing segment_intersection.
This is a candidate distinction, not yet demonstrated as the runtime cause.

Probe prebuild65084 exit0,1m16; runtime91275 terminal/reaped100,2.686s,
/tmp/ares-modifier-route188-probe.log. Intended endpoints captured:
(4790011,4730011) to(2539350,2539350), protected entry(4330063,4270063),
exit(2976446,2976446). Disputed native waypoint is absent BEFORE simplification;
final existing points match the emitted route. This falsifies the measured
simplification/drop-at-emission hypotheses. Do not port the adjacent boolean
intersection gap as this fix. Extend narrow probe to the selected boundary
contour: if absent there, inspect boundary generation; if present, inspect
entry/exit/vertex walk. Keep the candidate boolean gap deferred.

Contour prebuild63821 exit0,1m15; runtime43404 terminal/reaped100,2.663s,
/tmp/ares-modifier-route188-contour.log. Selected contour has nine vertices;
the missing native point lies along its unsplit diagonal segment3. Raw walk
is backward through indices3,2,1,0, consistent with these actual contour
coordinates; do not infer forward indices without the ordered contour.
No debugger is provisioned; native internals are not claimed observed.
Source candidate resample_polygon802–832 uses integer p2-p1, sqrt squared
norm, truncating offset, integer middle vector and truncating interpolation.
Rust rounds points and subtracts unquantized floating offsets. Independently
reviewed single-contract counterfactual will change only that resampling
function; keep cascade, closest-line and simplifier unchanged. Retain target
probe temporarily to compare selected contour/raw route before and after;
public tracer/full outputs decide whether this candidate actually fixes it.

Resampling counterfactual prebuild40911 exit0,1m17; runtime32496
terminal/reaped100,19.476s, /tmp/ares-modifier-route188-resample-focused.log.
Six public motion diagnostics PASS, including new ninth-layer on/off and
previous second-layer/outer-exit controls. Original complete baseline PASS;
modifier RED now128699/6286: nativeF5093 versus AresF5092. Preserve full
artifacts case-qdPxfY(baseline),case-PGDGQl(modifier) and remaining diagnostics.
Target modifier contour grows9→13 vertices; raw route includes
(3820518,4779608), which survives simplification and formats to the missing
X165.821Y104.78. Entry/exit geometry and contour segment indices consequently
change, not a fabricated output line. This is actual Ares before/after and
public native output evidence, not an observed native internal trace.
Source contract and public RED→GREEN support this correction. Static boolean
intersection gap remains deferred, unchanged. Remove temporary probes before
frozen gates; do not extend this checkpoint into the later feedrate diagnosis.

Temporary route probes removed; router diff is empty. Frozen production/test
diff SHA25636086260b629358791bfb093ed000dd01a5243d9446178624d0d467a1197acf4.
Prebuild44501 terminal/reaped0,1m17, /tmp/ares-modifier-route148-frozen-build.log.
fmt96438 terminal/reaped0; diff/probe searches empty. Clippy stage within
pipeline98463 finishes exit0,1m19 (workspace stage started via &&), unchanged
253corelib/269coretest/31CLIparity. Source unchanged thereafter. New Rust
resample247LOC and diagnostic194LOC remain below400. Independent static
readback has no new must-fix; final approval still requires all frozen gates.

Frozen seeded76722 terminal/reaped100:72run71PASS/1FAIL,236.839s;
/tmp/ares-modifier-route148-seeded-controls.log and matching artifacts preserve
only original complete modifier128699/6286. Quality pipeline98463
terminal/reaped0:clippy+workspace,7582PASS,37slow,9skipped,269.619s run;
/tmp/ares-modifier-route148-workspace.log. Existing vase ramps/current-spacing
behavioral controls PASS. Platform81583 terminal/reaped0:WASMbuild4m13 plus
bindings,WindowsGNUcheck37.68s,macOSDarwincheck37.82s; compile only, not native
Windows/macOS runtime. Chromium82732 terminal/reaped0:3/3PASS2.3m,
/tmp/ares-modifier-route148-browser.log; completeKSR6321698bytes at
/tmp/ares-wasm-gcode-ZyXPj0/browser.gcode, SHA256
91e2fce752e5dc6d7ed221b39209838000ba1cd1bf134e937a59a8b8d2d27ab4,
unchanged from previous checkpoint, and repeated complete bytes identical.
Official86779 terminal/reaped100:72run71PASS/1FAIL,258.732s,
/tmp/ares-modifier-route148-official-controls.log, same original complete
modifier128699/6286. Default67352 terminal/reaped100:72run54PASS/18FAIL,
15.149s, /tmp/ares-modifier-route148-default-controls.log, all artifacts
retained. Controlled71/1 does not replace the default-domain RED receipt or
prove all default failures share a reference mechanism.
Actual raw readback finds102current cases versus100previous,98identical raw
Ares outputs; changed cases are three routing-on modifier-travel diagnostics
and the target inherited-second-filament modifier. The two new diagnostic
runs share existing labels; routing-off new output matches its old control.
ZIP verification87660 terminal/reaped0 additionally confirms all98matched
pairs' member names/order and every decompressed byte:98identical,0misses.
Reference executable hashes unchanged (official64515d01...ba3b60,
seeded3d05d4cc...a3bfaa). Final fmt/diff7123 terminal/reaped0.
Source/test diff remains frozen, no probes. Request independent final six-axis
approval for five-file feature checkpoint only. Modifier feedrate RED,
boolean collision/inward arithmetic, default-domain parity and global printer/
option coverage remain open; no main merge or full-goal closure is claimed.

Independent final six-axis review approves the five-file finite checkpoint
commit/push on the feature branch, with no new must-fix. Final diff check and
production/diff probe searches are empty. Approval retains official/seeded
71/1,default54/18 and modifierF5093/F5092 RED; it excludes main, complete
modifier parity and full printer/options goal closure. Keep these docs with
the logical source/test commit.

## Current slice: modifier loop exit

Checkpoint5fb2e928 is pushed; worktree verified clean. Exact CLI prebuild exit0;
fresh seededCPU0 original complete acceptance65136 terminal/reaped100,
3.361s (/tmp/ares-modifier-waypoint-red.log): baseline PASS, modifier18606/905.
Raw files case-salSPe locate the symptom atZ=.84 after two tiny outer-wall
extrusions, not routed approach: native G1 X166.79 Y104.781 versus Ares
X166.794 Y104.783. All preceding commands through this loop match.

Approved public seam remains saved3MF to complete byte-output. Keep original
fixture/full assertions; add a native-first fourth-layer object-output tracer,
then the sole-variable reduce_crossing_wall=false control. Record complete
comparison artifacts even for local diagnostics; no private geometry test.
Owning source boundary is GCode.cpp::extrude_loop(5993–6033), with Point angle,
rotation and integer conversion helpers; Rust destination is
gcode_emit/motion/loop_paths/inward.rs and its loop-path consumer. Existing
loop geometry adapters remain temporary upstream compatibility shells.
Include only the measured loop-end move contract; adjacent seam/clipping,
routing precision, region-flow and later modifier gaps remain open. First
establish tracer RED/control verdicts, then rank falsifiable hypotheses; source
must explain the differing destination before a production edit. Remove any
temporary probes and run focused/full controls, fmt/clippy, platform/browser
gates and independent six-axis review before logical code+docs commit/push.

Native-first tracer prebuild82940 terminal/reaped0(12.14s), fmt77510 exit0;
public runtime51693 terminal/reaped100,2.664s, same18606/905 whole-file first
diff. Its complete fourth-layer first-object block differs only in the terminal
inward move; native precondition passes. Artifacts /tmp/ares-modifier-exit-tracer-red
case-vZYfYg preserve full input and both outputs. Add routing-off control next;
do not change production until the control and source diagnosis distinguish
this exit from the previous approach defect.

Routing-off prebuild82640 terminal0(11.77s); control2185 terminal/reaped100,
2.603s. Its fourth-layer block again differs only in the same inward endpoint,
so routing is not load-bearing. Its whole file has separate earlier M73
placement10599/552; preserve that output, not an all-file routing diagnosis.
Ranked predictions: wrong clipped/unclipped angle inputs change direction;
wrong point selection gives a distinct reference segment; shift/quantization
changes distance despite matching direction. Source5993–6033 consumes already
clipped paths(5804); current inward consumer receives original unclipped paths.
Point::ccw_angle(81–89) and end-3 index themselves match, suggesting the
container is wrong. No debugger installed; add one targetZ=.84/tiny-path probe
for original points, seam gap and emitted cursor, then remove before commit.

Probe build81000 terminal0(1m18); runtime57627 terminal100(2.653s) reproduces
the target, but no probe lines appear. Exact layer_z==.84 plus an assumed raw
point-count limit does not establish that the intended loop was inspected.
Retarget by a narrow Z interval and the measured pre-exit writer cursor;
do not infer its raw geometry from an unmatched probe.

Retarget build16440 terminal0; runtime11793 terminal/reaped100(2.647s) now
captures the intended loop: original A(4790011,4790011),B(4746960,4790011),
C(4790011,4746960),A; seam_gap=.06mm; actual clipped extrusion cursor
(4778026,4758944). Cutting60000scaled units removes the final C→A edge and
shortens B→C, so a four-point closed path becomes three points and source
end-3 moves fromB toA. Extend this target-only probe with actual orientation,
last role, clipper result and pre-rotation integer point per independent
preflight. Review also flags adjacent swapped-displacement-vector and missing
pre-rotation truncation contracts; determine whether they affect this target
before choosing scope. No acceptance reads diagnostic output.

Expanded metadata runtime68312 terminal/reaped100:5m48 build,2.655s test,
/tmp/ares-modifier-exit-metadata-probe.log. Confirmed ExternalPerimeter,
orientation(false,true), clipped three points A,B,(4778026,4758944),
pre-rotation integer(4781400,4790011). Compiler-only affinity was broadened;
test/reference parents remainedCPU0. No source changed during that run.
Temporary production probe removed. Include only clipped-tail ownership:
share allocation-free tail query with current mutator, use existing emitted
path_count/remaining_clip, retain original winding and scarf zero-clip paths.
Do not change arithmetic helper: swapped displacement is not exercised here;
pre-rotation fractional difference is below this formatted output's precision.
Both remain explicit deferred gaps, not claimed parity. Formatter must finish
before unrestricted prebuild, then CPU0 focused/full reference controls.

Fix prebuild91134 exit0,1m16. First focused invocation used the wrong artifact
environment name and failed before slicing; exclude it from behavior evidence.
Corrected runtime87412 terminal/reaped100: four public travel diagnostics PASS
(both fourth-layer exits and prior second-layer approaches), original full
baseline PASS/modifier RED now26707/1293, native intermediate X165.821 Y104.78
missing from Ares atZ1.88. Preserve complete outputs under
/tmp/ares-modifier-exit-fix-focused-valid, log with same root suffix.log.
Frozen clippy65441 exit0,1m20:253corelib/269coretest/31CLIparity unchanged;
fmt14270/diff checks exit0, temporary production probes absent. Changed Rust
files45/319/123LOC and diagnostic153LOC are below400. Independent source
readback finds no new must-fix, but checkpoint waits for all frozen gates.

Frozen source/test diff SHA256
190ddfd61bccd471ddc5ebf63b407fab72aa3944028cbc78ff0ea7fa10b2e757.
Seeded99331 terminal/reaped100:70run,69PASS/1FAIL,319.737s;
/tmp/ares-modifier-exit-seeded-controls.log and matching artifacts preserve
only original complete dual RED26707/1293. Workspace61352 terminal/reaped0:
7580PASS,38slow,9skipped,266.792s, /tmp/ares-modifier-exit-workspace.log.
Shared clip's three existing tests, spiral vase Z and current-spacing
behavioral coverage PASS. No new test hooks or comparator normalization.
Platform pipeline96075 terminal/reaped0:WASMbuild2m53 followed by bindings,
WindowsGNUcheck37.96s,macOSDarwincheck37.15s. Compile checks are not native
Windows/macOS runtime evidence. Chromium62312 terminal/reaped0:3/3PASS2.3m,
/tmp/ares-modifier-exit-browser.log; actual completeKSR6321698bytes at
/tmp/ares-wasm-gcode-yFnmD8/browser.gcode, SHA256
91e2fce752e5dc6d7ed221b39209838000ba1cd1bf134e937a59a8b8d2d27ab4,
same as previous checkpoint; repeated browser complete bytes identical.
Official76838 terminal/reaped100:70run69PASS/1FAIL,203.707s;
/tmp/ares-modifier-exit-official-controls.log and matching artifacts preserve
same original modifier26707/1293. Default73117 terminal/reaped100:
70run53PASS/17FAIL,15.132s, /tmp/ares-modifier-exit-default-controls.log,
all artifacts retained. Default domain is not GREEN; controlled69/1 does not
replace that receipt or prove all default failures share one cause.
Read-only raw readback:100current cases versus98previous checkpoint,95raw
Ares outputs identical; changed cases are four modifier-travel diagnostics
(including two newly added runs sharing that label) and the targeted inherited
second-filament modifier. Verification98627 terminal/reaped0 additionally
checks all95matched pairs' ZIP member names/order and decompressed bytes:
95identical,0misses. No unrelated raw Ares output changed in this finite suite.
Reference hashes remain official64515d01...ba3b60 and seeded3d05d4cc...a3bfaa.
Frozen source/test hash unchanged after all gates. Request independent final
six-axis approval for this seven-file checkpoint only; complete modifier RED,
adjacent inward arithmetic, default-domain parity and global coverage remain
open. No main merge or global completion is requested.

Independent final six-axis readback approves the seven-file finite checkpoint
for feature-branch commit/push. Final fmt74897 terminal/reaped0, diff check0;
production probe search and entire diff diagnostic-marker check are empty.
Approval does not close the open modifier, arithmetic, default-domain or
global coverage gaps. Document receipts remain with the code commit.

## Current slice: modifier approach routing

Footer checkpoint81504219 is pushed and the worktree was clean. Fresh original
complete acceptance, /tmp/ares-modifier-motion-red-valid.log, terminates100:
baseline PASS, modifier first difference9227/488. An earlier invocation lacked
ORCA_APPDIR and failed export127; it is not motion evidence. Original fixtures
and full assertions remain unchanged. Approved public seam is saved3MF to
complete G-code; add a separate native-first raw second-layer approach tracer
and then a one-variable reduce_crossing_wall=false control to isolate routing.

Ranked falsifiable hypotheses: multi-region boundary changes route (disabling
routing removes the approach difference); spiral direction diverges despite
equal route; wipe cursor differs before routing. Raw wipe coordinates match,
lowering the third hypothesis. No production fix yet: establish the tracer RED,
run the routing-off control, then select the smallest source-owned correction.

Upstream boundaries are AvoidCrossingPerimeters.cpp::get_perimeter_spacing,
get_perimeter_spacing_external,get_external_perimeter_width(499–550),
get_boundary/inner_offset(1014–1134),init_layer/travel_to, and GCode.cpp::travel_to
(7356–7505). Rust destinations: gcode_emit/layers geometry adapters,
perimeters/classic/traversal flow queries, and motion/path/avoid_crossing.
Include only the output-evidenced routing defect in this slice; defer unrelated
option/range expansion and other modifier differences without weakening the
original complete acceptance. Any temporary instrumentation must be removed.
Run focused/full controls, fmt/clippy and independent six-axis review before
a logical code+docs commit/push; unverified work is not a completed checkpoint.

Public approach tracer28170 terminates/reaps100,2.145s with identical first
diff9227/488; native approach preconditions pass. Its routing-off control27946
terminates/reaps0,2.106s (/tmp/ares-modifier-routing-off.log): the whole raw
approach matches when only reduce_crossing_wall changes. Selected prebuild
21080 is terminal0; the control waited on its build lock before running.
This localizes this approach defect to routing, not every complete-file gap.
Test the redundant-union hypothesis next: emit_helpers re-unions already
merged compensated Layer::lslices, while get_boundary consumes them directly.
Change only that union to an owned Rc slice copy, preserving point order.
Prebuild13730 remains live; do not claim the hypothesis confirmed yet.

Prebuild13730 terminates/reaps0,1m15s. Experiment7912 terminates/reaps100:
three tests run, routing-off PASS, tracer and original acceptance both remain
RED9227/488 (/tmp/ares-modifier-direct-slices.log). Redundant-union removal
does not explain this symptom and is restored with apply_patch. No production
change remains. Preserve the two public tests and request independent read-only
six-axis diagnosis review, not completion/commit approval. Routing-off is a
control, never a product workaround. Next correction needs discriminating
boundary/closest-line evidence rather than speculative combined changes.

Independent read-only review identifies resample_polygon(802–834): native
vertex offsets truncate, middle vector is formed from those integer offsets,
interpolated points truncate. Rust rounds both and computes middle with the
unquantized vector. Main verifies the source and applies only this numeric
contract (including source sqrt norm arithmetic) after public RED. Do not mix
the separately identified inner_offset acceptance-cascade gap(1066+) into this
experiment. Destination boundary/resample.rs, include resample_polygon/fill_middle;
defer cascade pending measured output. No new private/source-pinning test.

Truncation prebuild64578 terminal/reaped0,1m16s; runtime29420 terminal/reaped100,
6.861s, /tmp/ares-modifier-truncation.log. Original complete and public tracer
stay RED9227/488, routing-off control PASS. The numeric source gap does not
explain this target symptom; restore the experiment and retain it on the
review repair list for a separately red-capable public input. No production
changes remain after this experiment. Diagnostic clippy15340 terminates0
(31 CLI parity warnings) but predates the numeric experiment and is not a
final-source implementation gate. Next work must capture discriminating
region-flow/input-boundary and offset-stage evidence before further edits.

Next diagnostic build60638 terminal/reaped0,1m16s; original runtime80310
terminal/reaped100,2.618s, /tmp/ares-modifier-route-probe.log. Temporary
Tagged route logging records the wrong physical target when gated by
layer_index==1 (z=.36, not the first-difference z=.52). Do not infer target
geometry from those lines; retarget the probe by print_z=.52. Independent
read-only review additionally identifies unconditional closest expansion,
end-first shared-contour selection, collinear simplification intersections,
f32 boundary-distance accumulation, nudge conversion order and closest-hit
dedup/precision differences. These are static gaps, not established causes.

Target probe74126 build terminal0; runtime88509 terminal100 original acceptance
still9227/488, /tmp/ares-modifier-z052-probe.log. At z=.52 both inputs have
spacing=.4156637,width=.42,top0; baseline slices split a triangle/bulk while
modifier has one four-point square at±5000010. Modifier route's first point
is(164.875821,104.410214), consistent with its differing formatted approach.
This does not prove native input contours equal to either Rust contour.
Probe only the four-point modifier square's inner_offset round result count,
hole count and selected contour to test the premature-acceptance prediction.
Temporary route/offset diagnostic output must be removed before
commit, and git diff plus relevant sources checked for diagnostic markers.

Offset build20712 terminal/reaped0; runtime86247 terminal/reaped100,2.668s,
/tmp/ares-modifier-offset-probe.log. Immediately preceding target z=.52
approach (lines32–33), the modifier square has one hole after resampling;
first width311747.25 returns two zero-hole polygons. Rust prematurely accepts
the largest, while source1066–1091 retries before the last width. This is the
discriminating target-stage evidence, unlike the rejected rounding/union
experiments. Include only inner_offset's acceptance cascade: matching single
topology immediately; otherwise retry until final width; final single/max-area
or retained resampled original when empty. Largest-area ties retain first.
Remove both temporary diagnostic probes now; keep numeric/top-inset/closest
and region-flow gaps deferred for independently red-capable inputs. Rebuild
and run public tracer/off-control/original complete before claiming GREEN.

Cascade prebuild52519 terminal/reaped0; focused36184 terminal/reaped100,6.941s,
/tmp/ares-modifier-cascade.log: public approach tracer RED→GREEN, routing-off
control PASS, original complete baseline PASS; modifier moves to18606/905
(native G1 X166.79 Y104.781 vs Ares X166.794 Y104.783). Preserve full RED;
this slice closes the approach/cascade defect, not modifier/global parity.
Independent static review confirms cascade/first-tie/empty retention and no
bundled adjacent fixes, but awaits final runtime/quality results. Freeze source
and run workspace, clippy/fmt, WASM/platform/browser and approved full controls;
retain default/reference variability results before finite checkpoint review.

Initial quality clippy5304 terminates0 but adds one excessive-nesting warning
at the new maximum-selection loop (254lib/270lib-test versus253/269), so this
source is not approved for commit. Flatten early-width rejection and consume
the final candidates with a first-retaining strict maximum; same contract,
no helper/allow attribute. Workspace2634, WASM80942, Windows59963/macOS54533
were already launched: treat them as preliminary receipts, not final-source
validation. Re-run final gates after all these handles are terminal and the
flattened source is frozen; do not claim old or mixed build receipts as final.

The first flattened loop still adds excessive_nesting (frozen attempt73717).
Replace it with allocation-free area/candidate mapping, reversed max_by and
destructuring: max_by's last-equal selection now preserves the original first
tie. Integer polygon areas are finite; compute each candidate area only once
as upstream does, avoiding repeated scans of the current maximum. Independent
review caught and corrected this performance requirement before final freeze.
Intermediate2634/82146 workspace and73717/53322/71687/67968 quality handles
predate this final selection; record separately, not as final-source gates.

Final production resample.rs SHA256 is
f8c9096e4c3b0de354701e2b3139eb3e26493aef171dad6db1bc1f93789886d7.
Checkpoint clippy4966 terminal/reaped0 (5m22 including shared build-lock wait),
/tmp/ares-cascade-checkpoint-clippy.log:253lib/269lib-test/31CLI parity unchanged,
no new resample warning. fmt1156 terminal/reaped0; diff check0 and both full
diff and production gcode_emit diagnostic-marker searches have no matches.
No production edits after this snapshot; other gate handles remain live.

Intermediate workspace2634 and82146 terminal/reaped0:7578/7578 each (263.931s,
262.571s runtime respectively). Preserve these successes but rerun the explicit
checkpoint workspace command after the final snapshot, as promised.
Final WASM88047, Windows73325 and macOS41254 terminal/reaped0; browser bindings
10117 terminal/reaped0, generated from the checkpoint WASM artifact. These
platform build/check receipts are not execution on Windows/macOS nor browser
runtime approval. Exact public CLI prebuild5771 remains live; do not launch
controlled reference runs until prebuild and workspace handles are terminal.

Final workspace60911 terminal/reaped0, /tmp/ares-cascade-checkpoint-workspace.log:
7578run/7578PASS,35slow,9skipped,254.874s runtime. Spiral-vase Z ramp and
current-spacing/stopped-neighbor anchor tests PASS. Exact public CLI frozen
prebuild terminal0(0.249s); source SHA256 remains unchanged before reference
runs. Live-reference test flags are enabled separately below; unflagged
workspace success is not a claim that every external comparison executed.

Official CPU0 controls84209 retain an additional normal-auto/organic failure,
20367/851: native X113.388/Y113.135 vs Ares X113.244/Y113.297 at the next
Internal solid infill block. Current case-sX3yfB and prior official PASS
footer/case-WdZL6n and seeded PASS footer/case-Rb7tO0 raw Ares hashes all equal
6c2dd46ef4c68c3f514ae7d63d110af32b1979151777bda2664f5d40b776db38.
Current/prior official saved3MF member order and all9 decompressed members
match individually (including literal [Content_Types].xml). Thus Ares output
is unchanged for identical input content; retain the actual failure, not an
all-green claim or an unsupported explanation of reference internals. Current
official binary hash remains64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60;
seeded executable hash3d05d4ccab740ca5e13aa94d0d00867a75f867ba2159b9f5b024dc0fcaa3bfaa.

Official84209 terminal/reaped100,168.571s,68run66PASS/2FAIL,281skipped;
/tmp/ares-cascade-checkpoint-official-controls.log and artifacts retained.
Failures are the unchanged-Ares organic normalization case above and original
modifier later RED18606/905. Original complete baseline PASS and both new
public approach tests PASS. This finite suite mixes strict whole-output
acceptances and named limited diagnostics, not68whole-output successes.
Seeded controls start only after official handle terminal, with both Ares
and reference sharedCPU0 and seed0; no builder or fixture/comparator edits.

Seeded2011 terminal/reaped100,222.227s:68run67PASS/1original-modifierFAIL,
281skipped, /tmp/ares-cascade-checkpoint-seeded-controls.log. Complete baseline,
organic normalization and both public approach tests PASS; modifier retains
18606/905 full RED. Original organic normalization complete test repeats
terminal0 twice (2.878s/2.876s), organic-repeat-{one,two} logs/artifacts; no
fixture/assertion/build changes. These repeated passes do not replace the
official66/2 receipt or establish a reference internal cause. Default-domain
official batch uses no whole-process CPU pin and the unpinned official wrapper,
retaining its independent complete-output results; browser runtime follows.

Default official15697 terminal/reaped100,15.098s:68run48PASS/20FAIL,281skipped,
/tmp/ares-cascade-checkpoint-default-controls.log with all artifacts retained.
Both approach diagnostics PASS, but this execution domain does not pass its
complete-output suite; do not classify all20 failures as one mechanism or
substitute controlled67/1 for this receipt. Reference executable SHA256 values
before/after all batches are unchanged. Final Chromium3test run26729 is live;
bindings were produced from the frozen checkpoint WASM artifact, with no test
assertion, comparator or timeout changes.

Default artifact readback finds98current cases versus96previous footer cases:
95match a prior same-label raw Ares hash,2are the new diagnostic labels, and
the only changed old-label output is targeted inherited-second-filament
modifier case-9FvB9k. Read-only verification23134 terminal0 additionally checks
all95matched pairs' ZIP member names/order, every decompressed member, and
actual raw G-code bytes:95identical,0misses. This is finite no-output-change
evidence, not a claim all default failures have one reference cause.

Final Chromium26729 terminal/reaped0:3/3PASS(1.2m), including original complete
KSR ordered-byte comparison and repeated complete-byte identity, no timeout or
assertion changes. Output /tmp/ares-wasm-gcode-mgcmyZ/browser.gcode is6321698bytes,
SHA25691e2fce752e5dc6d7ed221b39209838000ba1cd1bf134e937a59a8b8d2d27ab4.
Request independent final six-axis approval for this6file code+docs checkpoint
and feature-branch commit/push only. Do not claim main merge, whole modifier
parity, default-domain or all-printer/all-option/global completion. Preserve
official66/2,seeded67/1,default48/20 and modifier18606/905 in the commit record.

Final independent six-axis review approves this6file finite code+docs
checkpoint for feature-branch commit+push, with no remaining source must-fix.
Approval explicitly excludes main/global/full-modifier/default-domain closure;
commit body must preserve all failure receipts and adjacent deferred contracts.
Final fmt14786 terminal/reaped0, diffcheck0 and diagnostic-marker diff search
has no matches.

## Current slice: per-filament footer accounting

Previous seam checkpoint442ff3f4 committed/pushed; main worktree clean.
Fresh official shared-CPU0 complete test34727 terminates/reaps100,2.600s,
/tmp/ares-footer-current-red.log. Baseline first difference151482/7181:
native368.40,149.19 versus Ares517.59,0.00. Modifier9227/488 remains a
separate motion gap. Original fixtures/comparator/assertions untouched.
Ranked hypotheses: scan ignores logical T; writer private counters do not
reach footer; tool0 properties reused. Current finish.rs explicitly uses
index0 for display and writer gates, while ExtruderRegistry already retains
all writer counters. Rewrite the source owners cited in spec/ARD, not an
Ares-designed proportional allocation.

1. Add native-first public saved3MF full-footer-block regression in separate
   test mod; prove RED before production changes.
2. Move existing scan into a functional accounting mod and extend it to
   logical-filament volume vectors, sharing timing's source T validator;
   preserve global cursors and return full processor vectors. Read existing
   registry statistics for per-writer gates. No new public slicer API.
3. Re-run tracer and original complete baseline/modifier acceptance, retaining
   remaining RED. Add the next property-vector public tracer only after this
   first RED→GREEN. Freeze quality/workspace/platform/browser and approved
   shared-CPU0 official/seeded controls; independent six-axis review/fix loop
   before code+docs commit/push. No main merge/global completion claim.

Included/deferred scope and destinations are in spec/ARD. Existing scanner
is a temporary compatibility shell; adjacent M1020/M600, arcs/units and tower
accounting remain output-evidenced follow-up boundaries, not silent guards.

Public footer tracer78875 terminal/reaped100,1.859s:
/tmp/ares-footer-tracer-red.log. Native preconditions pass; the entire raw
stats/totals/change-count block differs in the expected per-filament vectors.
Minimal implementation follows only after RED: move scalar scan to accounting
mod, retain global cursors, flush UsedFilaments total-volume cache on logical
T changes (1556–1567), share existing unsigned-prefix/count validator, and
preserve float volume directly. Writer registry statistics now gate global
line presence; all configured slots are displayed independently of those
gates, using source property precision/order. Empty registry emits no raw
filament vector lines. Attempt27346 builds/runs tracer and original complete
baseline/modifier; wait for terminal result before claiming GREEN.

Attempt27346 terminal/reaped101 before tests: nested accounting function
visibility was narrower than its gcode_emit re-export. Correct to the exact
owning boundary (not a public API), and use a let-chain for the trusted prior
logical ID instead of an unnecessary unwrap. Corrected60773 builds/runs the
same public tracer and untouched original complete test; log/artifacts suffix
footer-attempt-fixed. No successful-runtime claim from the compile failure.

Corrected60773 terminal/reaped100,3.313s: footer tracer PASS and the original
complete baseline now PASS; original modifier remains9227/488DIVERGENT.
No fixture/comparator/assertion changes. Preserve baselinecase-uiqg48 and
modifiercase-MhTDSb. After this first GREEN, add a public control using distinct
diameters1.75/2.85,densities1.11/1.29,costs0/32; compare the entire raw footer
block with native and independently require its zero first-cost/nonzero second
slot. This exercises property selection and line replacement, not all ranges.

Property90118 terminal/reaped0,2.539s: both full-footer public controls PASS
(/tmp/ares-footer-properties.log). Source frozen after the visibility/let-chain
fix; this property control required no further production edit. Launch final
clippy44803, workspace30153 and WASM34275; fmt/diff38815 pending terminal.
All quality/runtime results and approved-domain broad references remain due,
not inferred from focused GREEN. No zero-usage/SEMM/empty-registry runtime
claim from this distinct-properties/zero-cost control.

Frozen clippy44803 terminal/reaped0,1m20s:253core-lib/269core-test/31CLI
warnings unchanged (/tmp/ares-footer-clippy.log). fmt/diff38815 terminal/reaped0.
Workspace30153 and WASM34275 remain live; cross-compile Windows7770/macOS66192
also launched. Build locks are expected waiting, not completion or failure.
Actual browser/reference receipts still due after appropriate builds terminate.

WASM34275 terminal/reaped0,5m42s including build-lock wait; fmt52096
terminal/reaped0. Workspace30153 runs7576 tests with no failure observed yet;
Windows7770/macOS66192 remain live. Generate bindings only from this final
WASM artifact, then run actual Chromium after workspace is terminal so its
default-num-cpus runtime gates have no external slice load. Reference66-case
groups likewise remain due; no gate is inferred from a compilation-only check.

Workspace30153 terminal/reaped0,263.666s:7576PASS36slow9skip,
/tmp/ares-footer-workspace.log. Spiral-vase ramps and current-layer-spacing
behavioral controls PASS. Windows7770/macOS66192 terminal/reaped0,
6m00s/6m35s including locks, compilation only. Bindings70065 terminal/reaped0.
Prebuild selected CLI graph on unrestricted CPUs before whole-process CPU0
references (workspace graph may differ); keep that binary frozen for both
66-case batches. Actual browser and final runtime review still due.

Final selected CLI prebuild terminates0 (cached),
/tmp/ares-footer-final-cli-build.log. Launch official74066/seeded47994 all66
with all12 activation flags, --no-fail-fast, whole-process CPU0; retain seed0
and the explicitly approved replay executable. Logs/artifacts are
/tmp/ares-footer-{official,seeded}. Chromium33836 launches only after final
bindings/workspace terminal with the unchanged3 tests and finite timeouts,
/tmp/ares-footer-browser.log. Resolve host libraries only in that process's
LD_LIBRARY_PATH; do not change test assertions or host/product configuration.
Await all terminal receipts before final independent checkpoint approval.

Chromium33836 terminal/reaped0:3PASS1.2m, unchanged finite timeouts,
strict ordered-byte KSR comparison, binding privacy and repeated complete-byte
identity all PASS. Raw complete output is
/tmp/ares-wasm-gcode-0tcAuY/browser.gcode. This is actual WASM/browser evidence,
not inferred from native tests. Official74066/seeded47994 remain live;
no source/binary changes beneath those comparisons.

Broad official additionally FAILs support_blocker_volume at11886/508:
two Internal solid infill blocks appear in the opposite order. Do not dismiss
it without evidence. Current case-ZinXkl Ares raw SHA256
adb5f07104c782e8bbeb6607807ee97162613312f3ab12e9d7210aa0febfda5d equals
prior PASS /tmp/ares-seam-shared-official-full/case-W934FL and current seeded
PASS case-ddyw6o exactly. Old/new saved inputs have the same ZIP member order
and all10 decompressed members compare equal, including escaped content-types.
Native old/new differ in the dated header and that solid-infill block order.
This establishes unchanged Ares output and observed reference ordering change
for this case, not its internal cause or an unknown manifest producer identity.
Retain the official FAIL and await both broad terminals; then repeat the
original full support-blocker test against the same current official binary.

Official74066 terminal/reaped100,332.095s:66run63PASS3FAIL; seeded47994
terminal/reaped100,385.317s:66run65PASS1original-modifierFAIL. Preserve all
three official failures, not an all-green claim. The additional Hybrid first
diff6496/254 is M73 placement; later native diff exchanges the same small
solid-infill blocks. Independently verify current official case-UBEVS4,
prior PASS case-T2995Z and current seeded PASS case-F7bSyI Ares raw SHA256
6f9a9ab3bf60d7987356befe5106df6c6a0453e9baa21a409ac4fac3939ebe1a equals
across all three. Old/new Hybrid inputs retain member order and all9
decompressed members compare equal. This is evidence of unchanged Ares
output, not a general declaration of no regression or reference internal cause.
Current official binary SHA256 is
64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60.
Repeat both original complete tests twice on this same approved CPU0 domain,
without rebuilding, dropping M73, relaxing assertions or changing fixtures.

Original complete repeats15775/98106 terminal/reaped0: both2PASS,
8.164s/7.810s, logs/artifacts footer-official-repeat-{one,two}. The official
binary hash before/after is identical. Preserve initial official63/3 receipt
and both failure cases; these repeated passes do not rewrite the earlier
result or identify the native ordering mechanism. Seeded65/1 remains the
finite reproducible control receipt. Run a separate fresh default-multiworker
official66-case group, keeping both Ares and reference in the default domain,
footer-default-official. Record any failures independently; do not substitute
its domain for the approved shared-CPU0 comparison or weaken any assertion.

Default official18953 terminal/reaped100,15.064s:all66ran49PASS17FAIL,
/tmp/ares-footer-default-official.log and all case artifacts retained. This
domain does not pass; do not classify every failure as one mechanism or use
it to replace the controlled-domain receipt. Final fmt48176 terminal/reaped0,
diff check0 after receipts. This11file checkpoint closes only the measured
per-filament footer contract in the approved CPU0 domain, with original
baseline complete PASS and original modifier motion RED9227/488. All-printer,
all-option and default-domain/global parity remain open. Request final
independent six-axis approval for commit+push to the feature branch only,
not main merge or completion of the active goal.

Independent final six-axis review approves this11file finite checkpoint
for commit and feature-branch push, no new must-fix. Approval excludes main
merge/global completion, preserves modifier9227/488 and default17FAIL, and
does not mislabel zero-cost coverage as zero-usage or SEMM runtime acceptance.

## Current diagnosis: existing-vertex seam versus overhang projection

Previous turn is progress: closed-boundary checkpoint8d698005 committed and
pushed; remote HEAD matches and worktree was clean. Fresh original complete
test14811 terminal/reaped100,2.117s reproduces baseline4866/line298 and
modifier9227/line488 (/tmp/ares-seam-current-red.log). Initial direct-binary
attempt exited127 in Export due missing runtime library; it is not slicing
evidence. Re-run uses the existing official runtime wrapper unchanged.
Artifacts: baselinecase-OKWIuk, modifiercase-Iwr7qA under that log root.

Read-only body diff excluding M73/header for diagnosis only finds the first
baseline physical-motion gap at Z1.64: native starts Overhang wall at
X165.939 Y103.939; Ares starts Inner wall at X165.8 Y103.939. Complete
comparator and original fixtures/assertions are untouched. Rank: differing
seam selection (different target), split/projection (same target, different
first path), or emission ordering (matching split, diverging emitted order).

Source `GCode/SeamPlacer.cpp::place_seam`1624–1627 first tries
`ExtrusionEntity.cpp::ExtrusionLoop::split_at_vertex`182–225 and only calls
`split_at`266–314 with non-overhang preference when no vertex is found.
The Ares `seam_placement/split.rs::split_at` compatibility adapter snaps a
matching vertex but still projects with non-overhang preference. This is a
confirmed static contract gap, not yet a confirmed cause of this output.
Destination is that adapter and its seam-placement callers. Include source
first matching path, nearest vertex within that path (`MultiPoint3::find_point`
609–623), strict snap tolerance, path rotation and fitting
preservation; keep projection-only behavior distinct from vertex-first seam
placement. Include the direct projection caller's source false preference
without importing the SeamPlacer search. Defer adjacent projection endpoint
precision, modifier routing and per-tool footer accounting unless the
selected output evidence requires them. No wider rewrite bundled by inference.

1. Probe the original public baseline at the seam/split boundary to distinguish
   these hypotheses; remove any tagged temporary instrumentation.
2. Add native-first saved3MF raw role/motion regression in a separate test mod;
   observe RED before implementing the source vertex-first return contract.
3. Re-run original complete baseline/modifier output and retained controls,
   preserving all remaining RED. Run frozen quality/platform/runtime gates
   and independent six-axis review before the next code+docs commit/push.

Independent read-only preflight confirms the source seam boundary but does
not approve implementation/runtime/checkpoint. Required contracts: nearest
vertex within first matching path, strict double squared-distance and first
tie, fitting-preserving vertex rotation, single-path invalid-side no-op;
SeamPlacer first search1.5um differs from both methods' default1um
(`ExtrusionEntity.hpp:472–473`), and projection ends in its own default1um
vertex re-search. Audit all callers before changing this shared adapter.

Read-only Rust GDB probes97603/93617/91836 terminal/reaped0; debugger quit
intentionally stops its own test inferior, not an acceptance result. First
probe confirms types; targeted probe with standard Rust pretty-printers
captures /tmp/ares-seam-gdb-paths.log: seam3939340/3939340 exactly matches
the middle vertex of path1 OverhangPerimeter. Path0 Perimeter endpoints are
3800000/3939340 and3939340/3800000,139340 scaled units away. Thus existing
vertex-first splitting must keep the overhang start; the current non-overhang
projection selects the inner-wall endpoint, matching the public wrong start
165.8/103.939. Source-specific hypothesis2 is supported; no production edits
or temporary Rust instrumentation were needed. Complete baseline remains RED;
next step is the public raw role/motion regression before the fix.

Public raw role/motion regression84737 terminal/reaped100,1.595s,
/tmp/ares-seam-tracer-red.log, exposes the same wrong approach/role order
and clipped ending on the approved saved3MF/public output seam. Original
complete assertions unchanged. Implement only after this confirmed RED:
separate vertex helper, source first-path nearest/strict first-tie search,
fitting-aware rotation and single-path invalid-side no-op. Projection uses
default1um snapping and installs prefix/suffix in place before a separate
vertex re-search. All placement callers retain vertex-first1.5um; direct
spiral/non-perimeter projection now explicitly passes false without that
search (`GCode.cpp:5770–5775`). Attempt45713 builds/runs original complete
and new regression; no result claim before its terminal receipt.

Attempt45713 terminal/reaped100,2.132s: new public regression PASS,
original complete dual RED (/tmp/ares-seam-attempt.log). Baseline now first
diff151482/line7181 in filament statistics (native368.40,149.19 versus
Ares517.59,0.00); its entire preceding body including M73 is identical under
the unchanged complete comparator. Modifier remains9227/line488 spiral lift
direction. Preserve both artifacts case-FHRMli and case-kbHdU1. This is not
full dual GREEN. Independent six-axis static readback finds no new must-fix,
but runtime/checkpoint approval remains pending.

Frozen clippy97554 terminal/reaped0,1m52:253core-lib/269core-test/31CLI
warnings unchanged (/tmp/ares-seam-clippy.log). Fmt17694 terminal0 and diff
check0. No source edits after attempt build. Workspace79152, WASM8827,
Windows43128, macOS35443, official33982 and corrected seeded56514 are live
per direct handle polls; build locks are not completion or failure evidence.
Initial seeded57994 omitted required replay environment; preserve its setup
result separately, not slicing evidence. Corrected command explicitly uses
the documented jqfsg42f6xzq3krnf5scm6w4g1z4kz4c seed-replay binary, seed0,
CPU0. Actual Chromium validation and final independent readback remain due
after final WASM bindings and all terminal receipts.

Continuation confirms all seven previous handles live without restarting.
WASM8827 then terminal/reaped0,5m13 including lock wait; bindings20458
terminal/reaped0. Windows43128 and macOS35443 terminal/reaped0 (compilation
only). First Chromium32775 terminal/reaped1 before slicing, missing libnspr4
and other host runtime libraries; /tmp/ares-seam-browser.log is setup evidence,
not a product regression or acceptance. Its launch overlapped binding
generation, so it is not frozen browser evidence either. Resolve only current
process library paths; ldd reports no missing libraries after the correction.
Corrected Chromium99040 launches after bindings terminal0, with unchanged
tests, timeouts and complete-byte assertions; /tmp/ares-seam-browser-fixed.log.
Workspace79152, official33982 and seeded56514 remain live per handle polls;
initial missing-environment seeded57994 still must be reaped separately.
No Rust edits; final browser/workspace/oracle results and runtime review due.

Workspace79152 terminal/reaped0,278.739s:7574PASS39slow9skip,
/tmp/ares-seam-workspace.log; spiral-vase ramps and current-layer-spacing
controls PASS. Corrected Chromium99040 terminal/reaped0,3PASS1.7m,
/tmp/ares-seam-browser-fixed.log, complete artifact
/tmp/ares-wasm-gcode-fsANrv/browser.gcode. Initial seeded57994 terminal100
before reference Export (required environment missing),0.066s,2FAIL;
corrected56514 terminal/reaped100,4.980s,1PASS/1original complete FAIL.
Baseline151482/line7181 footer and modifier9227/line488 unchanged.
Official33982 terminal/reaped100,11.760s,17PASS/1original complete FAIL;
one of those passes (infill_first) is an inactive perimeter-reference guard,
not native-output evidence. Do not claim all18 were activated comparisons.

Coverage readback requires the existing Normal/Tree/region finite group
before this shared seam adapter's checkpoint. Launch broad official3469 and
seeded41392 with every owning flag explicitly enabled: filament/perimeter,
clipped-loop, normal/style/flow/angle, support annotations/modifier,
multi-object skirt, tree support/containment. Keep original complete dual
assertions and seeded CPU0/seed0 reference. These tests are the existing
acceptance group plus current diagnostics, not all-printer/all-option proof.
Logs/artifacts /tmp/ares-seam-broad-{official,seeded}; wait for terminal
receipts and six-axis runtime readback before code+docs commit/push.

Broad group has64 tests. Default-multiworker official3469 terminal/reaped100,
16.738s:49PASS/15FAIL;
retain all /tmp/ares-seam-broad-official artifacts and its full summary.
Two containment tests fail their native-reference literal precondition,
before Ares comparison; other complete-output differences require controlled
reference replay, not automatic classification as product regression or
automatic dismissal as nondeterminism. Launch unmodified official CPU0
2411 using the previously approved single-CPU wrapper and the same activated
group (/tmp/ares-seam-broad-official-cpu0). Seeded41392 remains live on CPU0.
Both reference jobs must finish before final approval. Default multiworker
failures remain recorded and no comparator/assertion/input is weakened.

Controlled broad official2411 terminal/reaped100,192.226s:58PASS/6FAIL;
seeded41392 terminal/reaped100,233.442s:58PASS/6FAIL. Both share failures:
original complete dual, Organic/default auto Tree, and manual default,
organic/grid/snug normalized Organic styles. These five extra failures are
must-investigate before checkpoint, not known reference variation excuses.
The auto/default case's first complete diff897/line33 is M73, while diagnostic
body diff reveals already-different first-layer skirt geometry. Compare
current seededcase-LIGiQ3 with previously passing ExportLinescase-soH7Ov:
project_settings.config, model_settings.config and3dmodel.model are identical;
current Ares skirt differs from prior Ares as well as current native.
No complete assertion edits. Independent final approval remains withheld.

After every preceding validation handle terminated/reaped, create detached
baseline worktree /tmp/ares-seam-baseline.UAna9g at8d698005, link unchanged
upstream source and launch the existing auto/default complete test against
CPU0/seed0 reference. No stash or checkout of the dirty implementation
worktree. Reuse the build target only after all previous jobs stopped, so
artifact executable hashes cannot change beneath running comparisons.
Baseline log/artifacts /tmp/ares-seam-baseline-head; await its terminal
result to distinguish this seam slice from earlier checkpoint regressions.

Baseline HEAD38001 terminal/reaped100,2.974s: auto/default complete test
fails at the same897/line33; case-hkNUcq raw Ares SHA256
5a33864576fa17f5748f75e7f70dcd16e57ebf0406a706200c98fd50057c73dc exactly
matches current case-LIGiQ3. This rules out this seam slice for that failure,
but does not waive the five required Organic controls. All nine ZIP members
and their order compare identically with prior ExportLinescase-soH7Ov,
including escaped [Content_Types].xml and the actual Objects mesh member.
Container timestamps/hashes differ, not decompressed input content.
Production support/tree, skirt, slicer_context and project_slice pipeline
have no diff betweenbe5d592f and8d698005. Ranked predictions: earlier
checkpoint gap, current seam change, or input/concurrency dependence; HEAD
standalone failure with identical bytes rejects the current seam change for
this case, but does not exclude execution-worker dependence. The Orca wrapper
alone pins CPU0; the parent Ares process has affinity0–63. Organic generation
reads available_parallelism(), and upstream TreeSupport3D.cpp:2385–2391 uses
the worker count to select two- versus four-leaf merge buckets. Observe the
actual test thread and replay the whole process in the approved shared CPU0
domain before attributing these five failures to a source regression.
Rebuild the previously tested be5d592f commit in the same clean detached
diagnostic worktree to establish an immutable source baseline rather than
assuming the older artifact's unknown revision proves its producing commit.
Log/artifacts /tmp/ares-seam-baseline-export-lines. Main dirty worktree stays
untouched except this evidence record; no test/assertion modifications.

The be5d592f replay9235 terminated/reaped100,2.933s. Its case-YpRXcf
differs from HEAD case-hkNUcq only in the automatic header date line; all
remaining bytes are identical. The old passing case-soH7Ov manifest has an
unknown source revision, so its output alone cannot identify a producing
commit or establish that both executions used the same worker domain.

Read-only debugger49423/11724 terminated/reaped0; quitting intentionally
stops their diagnostic inferior, not an acceptance run. Actual test thread
pathing workers=64 and /proc thread affinity0–63 are observed in
/tmp/ares-seam-worker-gdb-count.log. The merge probe observes input_size59;
workers/bucket locals there are optimized out. Source calculation gives
(59+2)/4=15, so workers64 selects bucket2 and workers1 selects bucket4.
The separate be5d592f whole-process CPU0 replay10400 terminates/reaps0,
1PASS2.877s (/tmp/ares-seam-baseline-shared-cpu0.log), with the original
complete-output assertion unchanged. This closes that execution-domain
diagnosis only, not the four other Organic controls or global parity.
Restore current main-worktree binary before both64-case shared-CPU0 batches.
Retain earlier mismatched-domain results and official default-multiworker
results; no production worker logic, fixture or comparator is changed.

Main restore66686 terminated/reaped101 before tests: Cargo reused a detached
baseline core artifact without GenerationMetadata::default despite the main
source's explicit Default implementation. Clean only ares-core's rebuildable
artifacts in /tmp/ares-tool-ordering-target (277files/4.8GiB); no source or
comparison output removed. Rebuild43762 must terminate before any reference
batch; this setup failure is retained separately, not a slicing regression.

Main rebuild43762 terminal/reaped0,1m15s, restores binary5fbb5659a3fb1a65
(/tmp/ares-seam-main-rebuild-clean.log). Current-source read-only GDB46226
terminal/reaped0 observes actual test-thread workers1 and affinity0 in
/tmp/ares-seam-current-cpu0-gdb.log; diagnostic quit is not acceptance.
Whole-process CPU0 official88022/seeded9586 launch all64 selected tests with
all12 reference activation flags explicit1. Logs/artifacts are
/tmp/ares-seam-shared-{official,seeded}. No concurrent rebuild or source edit
may change the executable beneath these comparisons. Wait for both terminal
receipts and independent six-axis readback before checkpoint commit/push.

Shared-domain88022/9586 terminated/reaped100,52.186s/63.280s: each
7PASS/1original-dualFAIL, but nextest's default fail-fast skipped the remaining
56selected tests. These are incomplete coverage receipts, not64-case gates.
Corrected86298/95049 rerun the identical frozen binaries, selection and12flags
with explicit --no-fail-fast; log/artifact suffix is shared-{official,seeded}-full.
Do not change assertions or drop the failing dual test to reach later controls.

Corrected official86298 terminated/reaped100,322.154s: all64 ran,
63PASS/1original-dualFAIL,281skipped. All five earlier mismatched-domain
Organic controls now pass the unchanged complete comparator, as do new seam
and activated infill-first controls. Seeded95049 still runs; do not substitute
official completion for its terminal receipt. Retain every earlier failure.

Corrected seeded95049 terminated/reaped100,374.046s: all64 ran,
63PASS/1original-dualFAIL,281skipped. Its five Organic controls, new seam
and infill-first controls also pass. Both complete dual failures preserve
baseline151482/line7181 per-tool footer and modifier9227/line488 motion gap.
These finite replays exclude those remaining gaps and all-printer/all-option
closure; they do not establish cross-worker-count reproducibility. Source is
unchanged since the frozen7574 workspace/quality/platform/browser receipts.
Final fmt14078 terminal/reaped0 and git diff --check0 after evidence edits.
Request independent final six-axis approval for this8file checkpoint and
feature-branch push only, not main merge or global goal completion.

Independent final six-axis readback approves this8file finite checkpoint
for commit and feature-branch push, with no new must-fix. Approval explicitly
excludes main merge, full dual output and cross-worker/global reproducibility.
Preserve baseline footer151482/7181 and modifier motion9227/488 RED in the
commit body, together with the retained mismatched-domain and fail-fast history.

## Current slice: closed thin-corner boundary parameters

Previous turn17506330 committed/pushed Auto Lift. Clean worktree; fresh
complete RED61557 terminal100,2.591s, /tmp/ares-detour-red.log reproduces
baseline17243/line844 and modifier9227/line488. Baseline has no modifier;
keep it as the existing selected public reproducer, not a rewritten fixture.
Ranked hypotheses: region-flow boundary, route selection/simplification,
wipe/lift replan. Ares targeted diagnostic67698 terminal100 captures exact
travel points and contours. Unmodified official GDB86996 terminal0 captures
identical start2792993/4790001 and end4811212/4811212; main contour is
identical, tiny corner contour differs. First two native debugger attempts
missed due optimized ABI; unconditioned attempt decoded invalid memory and
was stopped, so none are acceptance evidence. Corrected disassembly-based
probe /tmp/ares-detour-native-corrected.log uses XMM points/32-byte Polygon.
Probe panic removed from Rust after capture. No official source changes.

Source849–863 precomputes f32 cumulative distances plus closing segment;
the Ares own_parameters adapter uses f64 and omits the closing sentinel.
Port that parameter contract first, preserving shared elephant-foot queries.
Adjacent offset cascade/closest-line gaps are not bundled without evidence.

1. Public saved3MF native-first Z0.84 raw travel sequence RED, existing
   agreed output seam; original complete fixture/assertions unchanged.
2. Source closed-contour parameter port, then original/tracer/control output.
3. Preserve remaining RED; frozen full tests/fmt/clippy/WASM/browser and
   independent six-axis review before code+docs commit/push.

Public tracer79465 terminal100,1.810s /tmp/ares-detour-tracer-red.log:
native3 raw G1 approach lines versus Ares4, same166.884 versus166.669
first difference. Independent preflight confirms closure sentinel safe for
shared query partitioning; preserve real len parameters plus source_index=len
closure only in the parameter vector. Require integer difference before f64,
sqrt sum-squares and f32 per-segment accumulation, not existing hypot helper.
Constrain tracer to the single layer before RED; no original assertion edits.
Apply this contract after confirmed RED and rerun outputs before claiming fix.

Frozen build92205 terminal0,1m19; official attempt78761 terminal100,4.444s,
/tmp/ares-detour-attempt.log:public thin-corner tracer PASS, original full
dual RED. Baseline first difference4866/line298 is M73 before wipe motion;
modifier still9227/line488 spiral direction. Parameter repair changes actual
travel geometry and total timing; do not normalize remaining timing away.
Static re-review finds no new must-fix. Full gates still pending; no checkpoint
approval/commit or complete dual claim yet.

Continuation validates all seven launched handles live without restarting.
Clippy48807 terminal/reaped0,1m21:253core-lib/269core-test/31CLI warnings,
zero increment (/tmp/ares-detour-clippy.log). Fmt1013 terminal0/diff check0.
Workspace21286 is running; WASM74950, Windows40804, macOS21972 and
official57679/seeded15551 remain live. Do not infer terminal results from
logs or lock waits. Source remains frozen. A read-only body diff, ignoring
M73 for diagnosis only, places next baseline motion gap at Z1.64 overhang
wall seam/traversal and also exposes per-tool filament statistics; comparator
and complete assertions are unchanged. Browser and final review still pending.

WASM74950 terminal/reaped0 and bindings91747 terminal0; Windows40804
and macOS21972 terminal0 (compilation only). Seeded15551 terminal100,
24.927s:thin-corner tracer PASS, original complete dual RED with unchanged
baseline4866/line298 and modifier9227/line488. Official controls57679,
workspace21286 and actual Chromium17187 still live. Chromium full KSR
comparison produced /tmp/ares-wasm-gcode-Ur5EoT/browser.gcode; wait for
repeat-byte and whole browser-suite terminal result before approval.

Final workspace21286 terminal/reaped0,263.603s:7573PASS37slow9skip,
/tmp/ares-detour-workspace.log. Official57679 terminal/reaped100,82.631s:
16 controls PASS, original complete dual RED with the same locations as
seeded. Chromium17187 terminal/reaped0,3PASS1.9m,
/tmp/ares-detour-browser.log:complete KSR bytes, binding privacy and repeated
complete default bytes. All launched handles terminal/reaped; no source edits
after frozen build. Request independent final six-axis runtime readback before
code+docs commit/push. Main merge/global closure remain unproven.

Final fmt3464 terminal/reaped0 and diff check0; no DEBUG-detour residue.
Independent final six-axis readback approves these six files as a finite
closed-boundary-parameter checkpoint, with no remaining slice must-fix.
Commit code, public regression and documents together, then push the existing
feature branch per user authority. Preserve official16PASS/1FAIL and
seeded1PASS/1FAIL: complete dual output remains RED. This approval does not
close all travel behavior, printer/option coverage, main merge or the global
rewrite objective.

## Current slice: Auto lift overhang consumer

Previous turn made verified progress:33ebbdcd raw vendor fan repair committed.
Fresh original complete RED99970 exited100,2.570s,
/tmp/ares-m73-current-red.log: same M73 baseline4593/line284 and
modifier4185/line266. Worktree initially clean. Ranked hypotheses shown:
actual motion/fan differences, cache row mapping, tool timing synchronization.
Read-only diff excluding M73 (diagnosis only, not comparator) finds XY
routing differences at Z0.84 and missing Auto spiral lifts at Z10.28, followed
by cooling F differences. Therefore an M73-only index change is not closure.
Source GCode::needs_retraction tests recent object overhangs; Ares currently
uses fixed Auto→Slope except layer change. Source boundaries and included/
deferred contracts are in spec/ARD0094 before production implementation.

1. Public saved3MF native-first spiral-lift tracer RED using the agreed
   outputs seam. Do not alter original fixture/comparator/assertions.
2. Prepare source lift overhang cache from compensated all-region slices;
   wire the selected Auto lift into new travel deferrals, preserving existing
   pending lifts and actual object/support print_z.
3. Rerun tracer/original full output and controls. Keep remaining XY/seam,
   fan/timing differences visible. Frozen fmt/clippy/workspace/WASM/browser
   and independent six-axis review before local code+docs checkpoint.

Public tracerRED59866 exited100,1.834s:6 native spiral lifts versus2 Ares
at Z10.28. New test verifies savedAuto setting; original full assertions stay
intact. First build9577 exited101 due formatting a non-Display ClipperError;
use Debug error details. Intermediate compile53389 overlaps a later coordinate
correction and is not frozen acceptance. Fresh84371 exited0 before attempt
63093 exited100,4.444s: Auto tracerPASS, original full testRED. Baseline first
difference moves to XY boundary path17243/line844; modifier9227/line488 is
lift direction/preceding XY. Initial M73 location difference is gone, not
normalized away. Complete output still differs; no full dual claim.

Independent static review identifies one mandatory repair: actual raft entry
print_z must update active layer Z just like object/support, instead of relying
on merged chunk Z. Add that assignment. Preserve source's ineffective discarded
travel_bbox.inflated(1), effective object bboxEPS offset, double×float overlap/
opening order and integer clipping before translation. All other static axes
have no new confirmed errors. Re-freeze and rerun gates after raft repair.

Raft-fixed official23710 exited100,44.076s:15 controls PASS and original
complete dual RED. Clippy44763 exited0 but adds four warnings (257lib versus
253 baseline): nesting and argument count in the new cache. Extract the
source detector to motion/auto_lift/detect.rs, borrow live EmitState for query
inputs and give Layer its clipping predicate; retain arithmetic/order and
avoid lint suppression. Independent static re-review confirms raft Z and
all three changes with no new must-fix. Final frozen runtime remains pending.
This finite source slice is a code+docs commit/push checkpoint after runtime
review, not complete dual parity or all-printer/all-option closure.

Frozen cleanup receipts: clippy22558 terminal0,253core-lib/269core-test/
31CLI warnings (zero increment), /tmp/ares-auto-lift-clean-clippy.log.
CLI build72951 terminal0; fmt53388 and diff check0. WASM16154 terminal0,
bindings84238 terminal0. Windows36698 and macOS3677 check terminal0;
these are compilation evidence, not native runtime parity. Official11675
terminal100,84.318s:15 controls PASS, original complete dual RED. Seeded10515
terminal100,25.687s:Auto tracer PASS, original complete dual RED. Both retain
baseline17243/line844 XY166.884 versus166.669 and modifier9227/line488
spiral direction differences. Full byte assertions and original inputs unchanged.
Logs/artifacts: /tmp/ares-auto-lift-clean-{official,seeded}. Workspace1746 and
actual Chromium38413 remain running; do not infer their final results yet.

Final workspace1746 terminal/reaped0,272.762s:7572PASS37slow9skip,
/tmp/ares-auto-lift-clean-workspace.log. Spiral-vase ramp regression PASS.
Actual Chromium38413 terminal/reaped0,3PASS1.5m,
/tmp/ares-auto-lift-clean-browser.log:complete KSR bytes, binding privacy
and repeated complete default bytes. Artifact:
/tmp/ares-wasm-gcode-56Y6sU/browser.gcode. Final fmt25381/diff checks0.
All launched handles terminal/reaped; no source edits after the frozen build.
Request final independent six-axis runtime readback. Full dual remains RED;
remaining XY routing/seam/timing and global option/printer matrices stay open.

Independent final six-axis readback approves all15 files as a finite
Auto Lift code+test+docs checkpoint with no remaining slice must-fix.
Commit/push on the existing feature branch per user objective; do not merge
main or claim full dual parity/universal reproducibility/global completion.
Next diagnosis starts with the unchanged public full-output XY routing RED.

## Current slice: raw vendor fan-command identity

Cooling checkpoint55136c5c is committed, worktree clean. Fresh complete
RED2968 exited100: baseline3346/line234 and modifier3582/line241 both
miss ` M107 P0 ;fan off T0` (/tmp/ares-fan-raw-red.log).
Ranked probes: template generation, cooling filtering, fan state handling.
Saved3MF contains the indented command; CoolingBuffer preserves raw text.
FanMover instead trims before classifying, unlike upstream raw-line prefix
recognition (`FanMover.cpp:90–114,310`). Its duplicate-off suppression then
deletes this ordinary vendor line. Use the previously agreed public saved3MF
and slice_project seam; native-first raw vendor-line diagnostic supplements
the unchanged complete baseline/modifier test. No fixture/comparator changes.

1. Add public native-first indented-command tracer and observe RED.
2. Port source raw-prefix recognition and rerun tracer plus original full test.
3. Preserve any remaining full-output differences; run focused regressions,
   fmt/clippy/WASM and independent six-axis review before code+docs checkpoint.

Public tracer RED14786 exited100,1.825s: native31 indented vendor fan
lines versus Ares30, missing first off command; no original fixture changed.
Raw-prefix repair official43215 exited100,40.133s:13 diagnostic controls
PASS, original full test RED. First differences now baseline4593/line284
and modifier4185/line266, M73 appears one motion earlier. Native-first vendor
sequence and both cooling controls PASS. This is not complete dual parity.
Clippy77255 exited0: existing253lib/269lib-test/31CLI warnings unchanged;
fmt/diff checks pass. Seeded66416 exited100,12.969s: vendor tracer PASS,
unchanged complete test RED at the same M73 locations. Frozen workspace96911
exited0,7571PASS38slow9skip266.246s; WASM35629 and bindings70702 exited0.
Actual Chromium browser65984 exited0,3PASS2.3m, including complete KSR bytes
and repeated default-context bytes (/tmp/ares-fan-raw-browser.log).
Independent six-axis final runtime readback approves the finite local
code+test+docs checkpoint without mandatory changes. Full dual output, push,
mainline integration and global all-printer/all-option acceptance remain open.


## Current slice: per-tool cooling speed ownership

Header follow-up53dfd357 is committed; working tree was clean. Fresh original
complete RED60779 terminal/reaped100,2.582s,
/tmp/ares-dual-next-red.log: same baseline1913/line143 M73 ordering and
modifier703/line25 R16/R17. Preserve original fixtures/complete assertions.
Baseline has no modifier and is already the minimal selected public reproducer;
do not rewrite it while diagnosing the separate modifier acceptance case.

Ranked falsifiable hypotheses: emitted speed/Z/fan differences affect total;
planner synchronization differs on identical commands; M73-only cache alignment.
Read-only command diff confirms first-layer F1569 versus1570 and bottom
F1540 versus1570. Both baseline and modifier reproduce these actual commands;
an M73-only index repair cannot close that output defect. Cooling currently
uses one configuration/bucket, unlike source per-logical-tool joint slowdown.
Source boundary and included/deferred scope are in spec/ARD0094.

1. Public saved3MF native-first first-layer feedrate tracer RED, using the
   already approved outputs seam. Original complete comparison stays RED.
2. Replace single bucket with registered-ID parser and actual writer prefix;
   port joint range slowdown with per-tool settings and source f32 folds.
3. Rerun tracer and original complete comparison. Preserve remaining fan/Z,
   timing and complete differences as open, not fabricated GREEN.
4. Focused single-tool/cooling controls, workspace nextest, fmt/clippy/WASM,
   actual browser and independent six-axis review before a logical checkpoint.
   Code/documents together; no universal/all-printer/full-dual claim.

Public five-raw-G1-F tracer RED51846 terminal/reaped100,1.809s,
/tmp/ares-cooling-buckets-red.log: native1569/1569/1569/3000/1540 versus
Ares1570/1570/1570/3000/1570. Joint bucket attempt: original91322
terminal/reaped100,2.582s (/tmp/ares-cooling-buckets-green.log), first full
differences move to baseline2697/line201 and modifier2933/line208, both F.
Fail-fast cancelled the tracer, so it was separately rerun1872, terminal100,
1.821s (/tmp/ares-cooling-buckets-green-tracer.log): last F1539, still RED.
These log names are attempted GREEN stages, not passing results.

Independent static review returned three fixes: wire actual initial logical
tool (`GCode.cpp:2811`), retain denominator<0 return and total>target source
comparison. All implemented. Further actual command diff identifies duplicate
Z after printing tool changes: forced Z-sync in restate did not clear lift,
so unlift emitted another Z. Its zero-XYZ line inherits relative E in the
CoolingBuffer cursor and counts extra time. Include this necessary source
state write (`GCode.cpp:6381–6386`, `GCodeWriter.cpp:812–828`) in the slice.
The original first-layer tracer remains unchanged. Add a separate public
nonzero-initial control with distinct filament settings; fixture helper accepts
process/filament values and verifies their exported saved3MF configuration.

Forced-sync repair75634 terminal/reaped100,5.549s,
/tmp/ares-cooling-lift-green.log: original five-F tracer PASS1.865s;
unchanged complete comparison remains RED at baseline3346/line234 and
modifier3582/line241, missing vendor ` M107 P0 ;fan off T0`. No comparator or
original fixture changed. The initial-tool candidate with all solid roles2 and
sparse1 is blocked by existing `bridge_over_infill_anchor_extruder_order`,
not cooling; retain this unmet adjacent scope. The isolated initial-tool
control instead uses walls/bottom2 and internal/top/sparse1, density20%.
Both tools must still actually extrude, startup ends withT1, and no first-layer
T can mask initialization. Native's firstT0 block is internal solid, not sparse;
correct the new witness to require some subsequentT0 Sparse block, without
changing any original acceptance assertion. Both native and Ares emit the
same five first-layer F commands for this corrected configuration in raw
artifacts `/tmp/ares-cooling-initial-control/case-wwKqk6`; the corrected
test must still execute to prove its final assertions.

Initial-tool fault injection33081 terminal/reaped100,2.064s,
/tmp/ares-cooling-initial-red.log: with only current_tool temporarily forced0,
all native activation witnesses pass, Ares fiveF=2519/2519/2519/3000/2519
versus native300/300/300/3000/300. Restore the actual initial parameter;
the injection and its unused-argument warning are not retained. Independent
review confirms forced-Z source ownership and fixture migration; requests
final frozen runtime evidence. First clippy7976 terminal0 has9new warnings
(262lib/278test versus253/269); flatten adjustable loops preserving f32 order,
name mutation methods slow_to_*, pass existing Adjustment to parser aggregation,
and isolate the source zero-minimum proportional range branch. No lint allow,
numeric changes or independent algorithm; rerun clippy and final gates.

Final source controls: official15509 terminal/reaped100,13PASS/1original
completeRED52.889s (/tmp/ares-cooling-final-official.log); seeded66484
terminal/reaped100,3PASS/1original completeRED23.107s
(/tmp/ares-cooling-final-seeded.log). Both cold-start and nonzero-initial raw
feedrate controls pass; single-tool infill-first complete comparison passes.
Remaining original first differences are missing vendor M107, not hidden.
No full-dual/all-option/all-printer claim. Workspace35298 terminal/reaped0,
7570PASS39slow9skip274.971s; WASM34032 terminal/reaped0, optimized build,
bindings13010 terminal/reaped0. Production source is unchanged since these
gates. Final CLI test spelling uses rfind after clippy's filter_next warning;
fresh workspace/clippy and actual browser are still pending. Browser launched
while those build locks were pending; any queue delay is not a slicer result
and does not authorize changing assertions/timeouts.

Final clippy6039 terminal/reaped0, /tmp/ares-cooling-frozen-clippy.log:
253lib/269test/31CLI warnings, identical to the existing accepted baseline;
new increment is zero, no lint suppression. fmt/diff checks pass after rfind
formatting. Independent18file static six-axis readback found no remaining
code fix; update ARD wording to describe the former shell as replaced, not
still pending. Latest workspace/browser terminal receipts remain required
before the finite local checkpoint.

Actual Chromium79083 terminal/reaped0,3PASS5.5m,
/tmp/ares-cooling-final-browser.log: complete initialized-Orca ordered output,
no private hooks, and identical complete bytes on repeated default slicing.
Full browser artifact `/tmp/ares-wasm-gcode-ZrYXVf/browser.gcode` is retained.
Build-lock wait is included in wall time; no assertion or timeout changed.

Final frozen workspace15856 terminal/reaped0,
/tmp/ares-cooling-frozen-workspace.log:7570PASS35slow9skip259.406s.
No failed/regressed tests; source gates are now terminal. Keep original full
dual baseline/modifier RED and adjacent fan/geometry/initial template gaps
open. The passed finite cooling window is ready for final independent readback
and a local code+docs checkpoint, not push/main/all-printer closure.

Independent final six-axis readback approves the18file finite local
code/tests/docs checkpoint with no remaining slice fix. Approval explicitly
excludes complete dual, fan/sequential/adjacent behavior, push and main/global
closure. Commit body must retain the original complete RED and verified scope.

## Current slice: processor filament-change timing

Source and included/deferred boundary are recorded in the spec and ARD-0094.
Reuse the approved public saved3MF → import/effective settings → slice_project
→ complete output seam. Existing private timing tests only adapt their config
argument; no private acceptance replacement or assertion relaxation.

1. RED: unchanged original dual baseline/modifier complete comparison.
2. Port per-physical filament state and scalar delay branches into a dedicated
   processor module; connect Estimate using borrowed existing ProcessorConfig.
3. Replay the unchanged full comparison; inspect full totals and first diffs.
   Preserve known remaining fan/F/Z gaps as open, not fake GREEN.
4. Run existing processor/public single-tool controls, fmt/clippy/WASM and
   workspace tests; independent six-axis review before a logical checkpoint.

Fresh RED /tmp/ares-dual-timing-red.log, session82249 terminal/reaped100,
2.594s: baseline byte1726 line124 missing M73 P6 R14; modifier byte718 line25
R16 versus R17. Baseline native15m46s versus Ares15m41s. All three configured
filament load/unload/toolchange delays are zero, disproving positive-delay
omission as this fixture's explanation. Source still synchronizes on every
actual logical transition; Ares currently only synchronizes on initial T.
That zero-duration planner boundary is the next falsifiable probe.

Probe prebuild86761 terminal/reaped0 (1m16s); original full test97255
terminal/reaped100,2.598s, /tmp/ares-filament-timing-probe.log. Baseline's
missing line124 M73 is repaired; first difference now byte1928 line143
(M73 appears one motion early), total15m42s versus native15m46s (old15m41s).
Modifier remains R16 versus R17, total16m52s versus native17m0s. This is
measured progress, not complete output closure. No assertions were changed.
Independent static review found an M702 C-word boundary defect: substring C
wrongly treats MACRO as an unload axis. Share the source word-prefix predicate
between unloaded state and synchronization. Supplement original acceptance
with public native-first nonzero delay deltas for maps[1,2] and[1,1]; load31,
unload17 and change23 are verified in exported configuration. Exact native
total-time delta is the independent expected value, not recomputed branch math.
These timing diagnostics do not replace complete output comparison or prove
all scalar ranges, parsers, configurations or platform runtime behavior.

Nonzero delta controls10684 terminal/reaped0,2PASS6.915s. Exact C-word
boundaries follow GCodeReader's space/tab separators and comment/CR/LF/NUL
terminators, not all ASCII whitespace. Extended public shared-hotend control
covers empty end template, M702 MACRO, M702 C and M702 VT C. Controls7946
terminal/reaped0,2PASS16.908s (/tmp/ares-filament-timing-cword.log), after
prebuild91125 terminal0 (1m17s). Further source inspection found the planner
overwrites its preceding real motion with the zero ToolChange block, unlike
process_G1's machine.prev ownership. Independent reviewer confirms required
conditional previous update plus safe initial junction handling: a queued
initial ToolChange is not proof that a previous real motion exists. Final
snapshot preserves that state and will rerun original and scalar controls.

Queued user follow-up: after this timing checkpoint, remove date/time from
the generated-by header, retain epoch0/seed0 defaults, inspect GCode.cpp and
PlaceholderParser::update_timestamp first, and record template-variable
policy in ARD-0095. No parity comparator relaxation is authorized.

Preserved-motion prebuild1722 terminal/reaped0 (1m17s). Original/delta replay
38788 terminal/reaped100:2PASS/1originalRED in19.200s. Baseline firstdiff
byte1928 line143, modifier byte718 line25. Clippy41281 terminal/reaped0
(1m21s) exposed two new nesting warnings and one clone-to-slice warning.
Replace the repeated unload conditional with the source unload-time getter
and borrow the mapping slice; no assertion or expected output changed.
Frozen final quality receipts: fmt/diff checks0; clippy32903 terminal/reaped0,
5m33 including lock wait, /tmp/ares-filament-timing-clippy-final.log,
253lib/269test/CLI31 unchanged. Workspace70426 terminal/reaped0,
/tmp/ares-filament-timing-workspace.log:7567PASS,36slow,9skip,267.370s runtime.
WASM39554 terminal/reaped0,5m16 including lock wait,
/tmp/ares-filament-timing-wasm.log; bindings76598 terminal0. Windows58899 and
macOS68634 core checks terminal/reaped0 (5m38/5m58 including lock wait),
/tmp/ares-filament-timing-windows.log and /tmp/ares-filament-timing-macos.log;
these are compilation, not native runtime. Chromium79777 terminal/reaped0,
3PASS2.5m, /tmp/ares-filament-timing-browser.log; retained complete KSR
comparison artifact /tmp/ares-wasm-gcode-LEPeqQ/browser.gcode and repeated
complete byte/epoch checks pass. Official30620/seeded51349 58-control runs
remain live; do not infer their terminal status from intermediate PASS lines.

Official30620 terminal/reaped100,302.960s; seeded51349 terminal/reaped100,
356.289s. Both 58 activated finite controls finish57PASS/1originalfulldualRED,
/tmp/ares-filament-timing-official.log and /tmp/ares-filament-timing-seeded.log.
The added nonzero delta controls (including all four shared-hotend end
variants) pass both references on the final snapshot. No additional failure
is observed. These PASS counts include diagnostic lifecycle tests, not58
complete-output successes. Original complete baseline/modifier remains RED;
no full dual-tool, all-printer/option, native-platform runtime or global
completion is claimed. All launched handles are terminal/reaped. Request
independent final six-axis readback for a local code+docs timing checkpoint.

Independent final six-axis readback approves the14file finite local timing
checkpoint with no remaining must-fix. Keep full dual RED and current M73/time
differences in the commit body. Approval excludes push/main, all change-entry
points/statistics, complete parity and global goal closure. Header follow-up
must remain a separate source-cited code+docs slice after this checkpoint.

Status: in progress as a separate milestone after the single-tool Normal/Tree
merge. Isolated `94d33b59` reproduces the baseline body gap. The modifier input
was previously rejected, not previously supported.
Spec: [contract](../specs/2026-10-03-region-role-tool-emission.md).
Architecture: [ARD-0094](../../architecture/ard-0094-region-role-tool-ordering-and-writer-state.md).

## Confirmed RED and diagnosis

The unchanged inherited-second-filament public baseline/modifier comparison
fails against both official and seeded Orca. Expanded final runs each executed
7 controls (6 PASS, 1 FAIL), then canceled 40 under fail-fast. These canceled
controls were subsequently rerun separately: official 46 PASS/225.131s,
seeded 46 PASS/266.085s, both exit 0. This is not a passing 47-test suite.

Logs: `/tmp/ares-normal-tree-merge-{official,seeded}.log` and
`/tmp/ares-normal-tree-merge-{official,seeded}-finite.log`.
Official artifact root: `/tmp/ares-normal-tree-merge-official-artifacts/`;
`case-Na2CR7` baseline and `case-NQMkDs` modifier preserve inputs, complete
outputs and manifests.

First normalized difference is byte 186/line 7: `filament: 1,2` versus
`filament: 2`. Baseline sizes are 182130/173811 bytes; modifier sizes
270578/226756. Native baseline printing switches occur at lines 232 (`T1`),
413 (`T0`), 535 (`T1`); Ares has no corresponding body transitions.
Native `total_toolchanges` is 16; Ares reports 0.

Effective-config inheritance is present. The missing chain is
`GCode.cpp:4971–5075` region/role bucketing, then per-tool traversal at 5241
and `set_extruder` at 5308. Rust object entries only pass region density;
motion does not consume role-filament assignments or maintain a schedule.
`gcode_emit/header.rs` also writes the used-filament count as an ID; that
exact header defect is present in base commit `94d33b59`. This source proof
does not substitute for a baseline runtime comparison of the body defect.

## Finite TDD slices

### Mainline regression attribution

Isolated worktree `/tmp/ares-main-tool-regression.AeeJvE` is detached at
`94d33b596809096928d0fe163e6e065183e2e7f2`. Its production files are unchanged;
only an artifact-replay test was added to the existing parity test crate.
The public CLI independently slices the identical baseline successfully,
but still emits `filament: 2`, `total_toolchanges: 0`, startup-only tool
instructions at lines 37/96/140/181, and no printing-body tool transitions.
This establishes the body gap on the old mainline, not merely the header bug.
Baseline CLI output SHA256:
`c8e7de5b29b950bbf1526cfa1032b560d4784bfb23b8d7f82cffecb5ecc668b0`.

The identical modifier input is rejected by that mainline with
`UnsupportedProjectFeature("sparse_infill_density")`. No old modifier G-code
exists, so do not claim an old modifier body comparison. This batch newly
accepts that regional configuration; its dual-tool consumer gap remains
explicitly unimplemented, not reclassified as parity.

Input SHA256, baseline:
`78eca1c5df43561a9bb9da3526040b1a15d12a488d00c5ddf86d5bfbda00629f`;
modifier:
`30a532e027f991b2674066d9ff853201b329425106e9f09c141a4bb140cdd0d3`.
CLI logs: `/tmp/ares-main-tool-regression-cli-{baseline,modifier}.log`.
Original-comparator replay finished in1.306s, exit100, with both inputs
evaluated before the unchanged PASS/PASS assertion:
baseline `DIVERGENT` (byte186/line7), modifier `ARES_ERROR` (unsupported density).
Log: `/tmp/ares-main-tool-regression-replay.log`; output artifacts are
`/tmp/ares-main-tool-regression-artifacts/case-L7GWAx` for baseline and
`case-8BVJ3T` for modifier.
These outcomes plus actual baseline body inspection establish the existing
gap; they do not claim the old modifier generated a comparable body.

### Implementation order

First finite public tracer: a single-tool modifier sets only `is_infill_first`
while the parent is wall-first. Source `GCode.cpp::extrude_perimeters` applies
each nonempty region configuration and selects its before/after-infill phase;
the first layer always prints walls first. Existing Rust order preparation
reads only the first model-part region flag and drops perimeter region IDs
despite `LayerFillEntities::perimeter_ranges` already retaining them. Public
test `orca_parity_modifier_infill_first_preserves_region_wall_phases` compares
both baseline and modifier complete files with unchanged comparison rules.
Provisional official RED exit100/2.342s: baseline PASS, modifier DIVERGENT, first difference
byte1296/line46 (`M73 P3 R16` versus `R17`), followed by body differences.
Native role-sequence witness differs; this is not just a config-header change.
Log `/tmp/ares-region-infill-first-red.log`; artifacts case-ueaqdU and
case-Om1XYs in `/tmp/ares-region-infill-first-red-artifacts/`.
This is a source-owned traversal prerequisite, not a multi-tool completion.

The old held worktree was temporarily stashed and checked out to94d33b59 by
the user's review process, then restored. Do not accept the provisional RED
from that overlap as authoritative; rebuild and rerun on
`codex/region-role-tool-emission` in `/tmp/ares-tool-ordering.jdZMyT`, with a
separate build target, before implementation. Remote main and the accepted
b86b73bd commit remain intact. Preserve the restored original-worktree edits.

Authoritative isolated rerun completed before production edits:
`/tmp/ares-region-infill-first-isolated-red.log` exit100/2.354s, baseline PASS,
modifier DIVERGENT at the same byte1296/line46. Both reference role-sequence
and unchanged complete-output assertions executed. Separate target
`/tmp/ares-tool-ordering-target` prevents reuse of overlap builds.
Implementation carries `perimeter_ranges` region IDs through island
assignment and ordered collections; motion emits the source four phases using
each region's flag. No geometry copies, header-only fix, tool guards or
unused ToolOrdering scaffolding. First-layer semantics are explicitly derived
from actual object/raft id and bottom_z, not the emitter's simplified id check.
Authoritative official GREEN `/tmp/ares-region-infill-first-isolated-green.log`
exit0:1 PASS/2.351s, both complete baseline and modifier comparisons executed.
The final build took6m49s because the process-wide replay affinity also reached
compilation; compiler-only affinity was expanded while actual test and Orca
replay stayed CPU0. This does not change production worker behavior.
Final official/seeded controls, workspace, clippy and WASM remain in progress;
do not commit the slice as closed before their actual terminal results.

Final frozen-source Clippy `/tmp/ares-region-infill-first-final-clippy.log`
exit0, core257 lib/273 lib-test diagnostics unchanged from the accepted
batch, no never-read/never-used/dead_code match. No added warning suppression.
This is not a `-D warnings` claim. Fmt/diff checks pass; modified functional
Rust files remain below400 LOC. Remaining validation is still live.

WASM check `/tmp/ares-region-infill-first-wasm.log` exit0 for ares-core and
ares-wasm, elapsed9m47s including build-lock queue. This is compilation, not
a fresh browser/native Windows/macOS runtime certification. Both activated
complete-output control runs select47 tests, exactly excluding the preserved
dual-tool RED; 282 other tests are outside these filters. Workspace selects7546
tests. Do not call any still-running control or workspace result PASS yet.

Workspace terminal `/tmp/ares-region-infill-first-workspace.log` exit0:
7546 PASS/36 slow/9 skipped in265.487s. The default env-gated parity returns
are not substituted for activated native controls: both47-control runs are
still live, and the new modifier tracer has already passed in seeded output
(14.335s). Original fixtures, complete comparators and the dual-tool RED are
unchanged. No current source edits remain after this frozen validation.

Seeded47 terminal exit0/349.876s. First official47 terminal exit100/305.864s,
46 PASS/1 FAIL: existing later-higher-density control failed its parent
baseline before reaching the modifier, at byte12097/line524, equal normalized
length138530. Native expected travel X113.388Y113.135; Ares X113.244Y113.297,
followed by swapped variable-width solid-fill paths. Keep this result; do not
report the initial suite all-green.

Diagnosis used the identical input SHA256
15811835a4ebaf38e3e391accc6a686d4255229021d476b48315677d9e3e41c2.
Saved failing case-BJ4QiK is in the official-controls artifact root. Both the
saved input and original producer path in orca-runner-NxZYui have that hash.
Four official CPU0 replays (two saved path, two original path) print the Ares
travel at line524, unlike the original reference. Replay1/2 are raw-identical,
SHA4e29ad6ba7b3bd108925120fc881a875406c6340a41c8fef8f568118646f438a;
replay3/4 are raw-identical too. Outputs/logs retained under
`/tmp/ares-region-infill-first-official-replay-{1,2,3,4}`.
This establishes observed producer variation, not its internal mechanism.

Untouched isolated b86b73bd production in
`/tmp/ares-region-phase-mainline.9Vk9pc` successfully slices that same input.
Its complete Ares output is byte-identical to the candidate after removing
only each Ares generator-time line (diagnostic old/new comparison, not a
relaxed native acceptance assertion). CLI output/log retained at
`/tmp/ares-region-phase-mainline-baseline.{gcode,log}`; source status is clean.
Thus this baseline difference is not introduced by the region-phase slice.
Unchanged full public test focused rerun exit0/2.338s:
`/tmp/ares-region-phase-official-focused.log`. No source, fixture or comparator
was modified during diagnosis. Fresh complete official47 terminal exit0:
47 PASS/282 outside-filter skips in114.018s, recorded at
`/tmp/ares-region-infill-first-official-final.log`. All validation processes
have terminated and their handles have been collected. This closes the
region-phase prerequisite validation, not the deferred dual-tool milestone.
The initial official failure and its retained diagnosis remain part of this
record; the complete-output assertions were never relaxed.

1. Preserve canonical region ownership through ordered extrusion collections.
   Source: `GCode::process_layer` region/island buckets and split wall roles;
   destination: `island_print_order` and layer-entry adapters. Add behavior
   coverage through the existing public role/region configuration seam before
   replacing adapters. No private-source pinning or fixture proliferation.
2. Port actual layer tool selection/order. Source: `LayerTools::extruder`,
   `ToolOrdering::initialize_layers`, `collect_extruders`,
   `handle_dontcare_extruder`, `reorder_extruders_for_minimum_flush_volume`,
   `collect_extruder_statistics`, `mark_skirt_layers`. Destination:
   `gcode_emit/tool_ordering/` consumed by `layers`. Role witnesses must cover
   wall versus bottom/internal solid and newly active sparse infill. Do not
   replace upstream ordering with a sorted-ID heuristic.
3. Port writer/per-filament state and transitions. Source: `Extruder`,
   `GCodeWriter::toolchange` / `reset_e`, `GCode::set_extruder`; destination:
   `motion` state and a source-owned transition module. Exercise switch-back,
   retraction, E reset/conversion, offsets and template counters through the
   public baseline. Preserve global machine state and selected filament state
   separately. Reuse existing template and motion consumers where source-correct.
4. Connect tool-outer traversal and derived metadata/statistics. Source:
   `GCode::process_layer` and ToolOrdering statistics; destination: `layers`,
   `header` and existing final-output consumers. Run the existing complete
   baseline/modifier comparator against official and seeded until both GREEN.
   Partial metadata/body witnesses are diagnostic only, not this exit gate.
5. Run the full 47 activated controls with `--no-fail-fast`, then workspace,
   lint/format, WASM/browser and cross-target validation; request independent
   six-axis review of a frozen source snapshot before commit/push.

Each slice starts from a public behavioral RED and ends in a consumed runtime
path. Commit verified independently closed changes with their docs. If the
tool schedule/state/traversal dependencies cannot form a passing independent
slice, keep them together rather than committing unused modules or broken
intermediate runtime. Keep functional Rust files below 400 LOC.

## Reference reproduction and scope

### Dual-tool implementation resumed after prerequisite push

The region-phase prerequisite is pushed as `031b3304d41eb734c71a42395c208372374d7600`
on `codex/region-role-tool-emission`. Production work continues only in the
isolated `/tmp/ares-tool-ordering.jdZMyT`; the original restored worktree is
untouched. Fresh official public baseline/modifier RED before further edits:
`/tmp/ares-dual-tool-current-red.log`, exit100/2.452s. Both complete comparisons
are DIVERGENT at byte186/line7; artifacts are `case-9OeTWF` and `case-5HaKnL`
in `/tmp/ares-dual-tool-current-red-artifacts`. No fixture/assertion changed.

Source audit adds the invoked scheduling dependencies to spec/ARD:
`ToolOrderUtils.cpp:385-790` order solvers and grouped layer sequence assembly,
`ToolOrdering.cpp:498-580` area-based initial wall order and configured first
layer order, and `ParameterUtils.cpp:6-29` configured layer-range decoder.
`Print.cpp:2429-2430` / `3399-3400` builds and sorts the ordering before G-code
consumption. Keep collection's one-based/don't-care IDs separate from final
zero-based logical filament order and physical-hotend mapping.

Current first implementation step separates the seven existing active
extrusion counters into `motion/extruder_state.rs`, updating their real motion,
template and accounting consumers without changing arithmetic or output.
Writer XY/Z, wipe paths, lift and motion caches remain global. This active
frame is not yet a logical-filament registry: SEMM's physical shared E and
retraction slots must not be saved/restored as if they were private filament
state. The following registry/selection port must preserve that distinction.
Do not claim this frame alone implements a transition or closes the dual-tool
RED. It remains uncommitted with the coupled schedule/traversal/writer work
until a validated independent closure exists; no documentation-only commit.

Active-frame `cargo check -p ares-core --all-targets` terminal exit0/54.57s:
`/tmp/ares-dual-tool-extruder-frame-check.log`. Fmt/diff checks pass. Focused
G-code nextest and workspace Clippy are still running; no runtime PASS claim
yet. Interim independent six-axis review found no mechanical migration defect
and explicitly did not approve an independent commit or registry completion.
Processor/fan-mover E state is separate and was intentionally not migrated.
Remaining implementation must consume tool-specific conversion/configuration,
switch-back state and all-filament final statistics, not just the active frame.

Focused existing G-code regressions terminal exit0:272 PASS/6827 outside-filter
skips in52.727s, `/tmp/ares-dual-tool-extruder-frame-tests.log`. This includes
the public spiral-vase and multi-object identity regressions; it is not an
activated native parity sweep or a workspace/all-printer acceptance claim.

Workspace/all-targets/all-features Clippy terminal exit0/4m08s:
`/tmp/ares-dual-tool-extruder-frame-clippy.log`, unchanged core257 lib/273
lib-test diagnostics, no never-read/never-used/dead_code match. No suppression
added and no `-D warnings` claim. Public parity binaries are rebuilt without
the runtime CPU0 restriction before executing the approved single-CPU replay.

Fresh official public replay `/tmp/ares-dual-tool-extruder-frame-public.log`
terminal exit100/4.885s: single-tool infill-first complete baseline/modifier
PASS (2.414s), unchanged dual-tool complete baseline/modifier DIVERGENT
(2.469s) at the same byte186/line7, with unchanged normalized lengths.
Cases `case-XWvAc0` and `case-3yhYUQ` preserve the dual-tool results in
`/tmp/ares-dual-tool-extruder-frame-public-artifacts`. Their input hashes match
the pre-edit baseline/modifier, and both complete raw Ares files compare
byte-identical with the pre-edit cases (two `cmp` exit0, no normalization).
This proves the active-counter move preserved those existing outputs, not that
the missing transitions are implemented. All started build/check/test process
handles have been collected at terminal. The coupled dual-tool work remains
active and uncommitted; next implementation is the actual private/shared
registry and schedule-driven transition/traversal consumers.

Use the existing Orca parity runner and approved whole-process CPU-0 replay,
with `ARES_PARITY_REGION_FILAMENT_REFERENCE=1`; retain the actual official and
seeded binaries and artifact manifests. No new reference modifications.
Do not turn the confirmed failure into an ignored test or edit its assertion.

The user approved this separate milestone after single-tool closure. Isolated
`94d33b59` reproduces the baseline body defect; the modifier is separately
recorded as a newly accepted input that still lacks multi-tool output parity.
No further painting edge combinations are a current merge requirement.

## Extruder registry and template consumer slice (uncommitted)

Source boundary: `Extruder.cpp/.hpp` private counters and SEMM physical axes,
`GCodeWriter::set_extruders`, and `GCode.cpp::PlaceholderParserIntegration`
(`1625–1713,3908–3914`). Destinations: `motion/extruder_registry.rs`,
`motion/extruder_state.rs`, and `motion/writer_state.rs`; the latter owns live
template refresh/write-back previously embedded in `path/role_change.rs`.
No geometry copies, public tool API, reference modification or relaxed
complete-output comparison. Mode comes from actual motion configuration, not
a separately cached Boolean. Missing filament-map entries use physical0,
matching `Extruder::extruder_id`, not the first map entry; positive/in-range
entries are already checked at the effective-config boundary.

First attempted test build failed because `GCodeOptions` has no `Default`;
this is not accepted as RED. Corrected tests project the four typed source
views. `/tmp/ares-extruder-registry-red-retry.log` terminal exit100,3 FAIL in
0.026s: inactive position/retraction/extra were discarded, inactive negative
retraction returned Ok, and statistics had length1 instead of3. After wiring
all registered IDs, `/tmp/ares-extruder-registry-green-controls.log` exit0,
275 PASS in53.222s, including spiral-vase and multi-object regressions.

Intermediate Clippy `/tmp/ares-extruder-registry-clippy.log` exit0/4m00s but
259 lib/275 lib-test diagnostics, two above the accepted257/273. This is not
an acceptable final quality result. Both new argument-count diagnostics are
addressed using the semantic retraction/restart pair and area/density pair;
the moved vector validator now returns its error directly, without an
additional error-output parameter or unreachable fallback error.

Independent review found the empty-registry and relative-E output-variable
branches. `/tmp/ares-extruder-registry-output-contract-red.log` exit100,
9 tests:6 PASS/3 FAIL in0.032s, proving empty registries incorrectly published
and validated E output variables and relative-E incorrectly validated absolute
position. Passing controls exercise partial write-back on a later-tool error,
zero-filled unused IDs, SEMM same/different physical-axis switch-back and the
missing-map branch. Those three branch failures were then corrected; SEMM
coverage additionally writes an inactive filament's private template counters
and confirms its shared physical axis remains unchanged on activation.

Frozen final controls, Clippy and public parity rebuild are running; collect
their terminal outcomes before calling this slice validated. Do not call the
intermediate checks complete dual-tool parity or approve a commit from them.

Final consumer controls `/tmp/ares-extruder-registry-final-controls.log`
terminal exit0:281 PASS/53.537s,6827 outside the selected filter. All nine
registry tests, spiral-vase and multi-object controls passed. Final Clippy
`/tmp/ares-extruder-registry-final-clippy.log` terminal exit0:256 lib/272
lib-test diagnostics (one below the accepted257/273), no dead-code/never-used/
never-read diagnostic and no added suppression. `cargo fmt --all --check`
and diff check exit0. These are consumer-stage checks, not workspace or full
dual-tool closure. Public parity rebuild and WASM check are still pending.

Public parity rebuild `/tmp/ares-extruder-registry-final-parity-build.log`
terminal exit0 before CPU0 replay. WASM check for core/browser adapter
`/tmp/ares-extruder-registry-final-wasm.log` terminal exit0/3m52s including
build-lock queue; compilation only, not a fresh browser runtime assertion.
Official public replay `/tmp/ares-extruder-registry-final-public.log`
terminal exit100/4.816s: single-tool infill-first baseline/modifier complete
output PASS (2.347s); dual-tool baseline/modifier remain DIVERGENT (2.467s),
both first differences unchanged at byte186/line7 with the original lengths.
Cases `case-FDqLpd`/`case-6rP14N` are retained under
`/tmp/ares-extruder-registry-final-public-artifacts`. No fixture or comparator
was changed. All process handles started for this consumer stage reached
terminal; no pending build or test is being reported as successful.
Both input SHA256 values match the pre-edit baseline/modifier recorded above;
both complete raw Ares outputs also `cmp` byte-identical against pre-edit
`case-9OeTWF`/`case-5HaKnL` (two exit0, no normalization). This stage changes
the internal template consumer contract and adds behavioral coverage; it does
not implement the missing printing-body transitions.

Independent read-only six-axis revalidation closed this consumer-stage repair
list after checking the terminal evidence. It explicitly does not approve an
independent commit, complete dual-tool parity or the global printer/option
objective. Keep the coupled implementation uncommitted until the remaining
source-owned producer/transition consumers below are closed and reviewed.

Still coupled and required before the dual-tool closure: actual writer-used
tool collection including support/interface, actual initial tool, all-tool
machine-start write-back, per-filament conversion/configuration installation,
source ToolOrdering/ToolOrderUtils schedule,
per-tool object/island/region buckets, complete `set_extruder` transitions and
all-tool final statistics. Current object-only registry initialization and
initial0 are temporary compatibility wiring, not a source-complete schedule.
The unchanged inherited-filament baseline/modifier whole-file assertion must
become GREEN against both references before that coupled implementation is
committed. Fresh workspace/platform checks and six-axis review remain gates.

### Relative-E axis ownership continuation

The existing unchanged public dual-tool RED remains the tracer for this
coupled milestone; no private implementation test seam is added.
`Extruder.cpp::extrude` (30–49) resets only the current E axis before adding
each relative delta, retaining signed absolute usage. `Extruder::retract`
(58–85) also resets that axis when the remaining retraction is zero.
The Rust destinations are `motion/extrusion.rs::coordinate` and
`motion/travel/retract_core.rs::plain_retract`; firmware retraction already
calls `coordinate` even for zero remaining length. The active frame represents
the shared physical axis under SEMM and the private axis otherwise, so this
does not reset inactive-tool private counters or statistics. Relative-axis
resetting does not reset signed absolute usage; complete-output assertions
remain unchanged.
Independent review identified the layer-start duplicate retraction path and
the wipe pre-call zero-delta branch. Layer start now consumes the existing
`GCodeWriter::_retract` compatibility shell `plain_retract`, including its
firmware fake-length counters, rather than directly assigning retraction.
`GCode.cpp::retract` (7653–7683) owns whether the pre-wipe and trailing calls
actually happen: zero-delta calls still reset relative E; a suppressed tail
with no wipe makes neither call. A nonsuppressed firmware tail still calls
retract after wiping. Do not put this behavior into `reset_e`, which has a
distinct relative-E no-op contract.
The before-wipe call installs proportional restart extra; only an actual
trailing call reinstalls full restart extra. No-wipe suppressed-tail calls
preserve the previous extra. Firmware retraction consumes the caller's extra
instead of replacing it, and layer-start firmware now updates signed usage
through the existing fake-length counter path rather than assigning only
the retraction amount. Independent static six-axis re-review closed these
repair findings; terminal runtime evidence is still pending.
Initial controls/Clippy/parity-build logs are intermediate because these
review repairs were made while compilation was running; final verification
must rebuild the frozen repaired source and use distinct log names.
This is a source-owned counter prerequisite, not a complete ToolOrdering
producer or successful dual-tool output claim. Controls and native replay
must reach terminal before recording validation outcomes.

Terminal continuation evidence (frozen repaired source):

- `/tmp/ares-relative-e-axis-final-controls.log` exit0:127 PASS/39.130s,
  6981 outside the filter. Includes writer registry, motion, spiral-vase
  and multi-object controls; not a fresh full-workspace run.
- `/tmp/ares-relative-e-axis-final-clippy.log` exit0:256 core lib/272
  lib-test diagnostics, below the accepted257/273 baseline, no dead-code/
  never-used/never-read match or added suppression. Elapsed6m11s includes
  build-lock queue. Fmt and diff checks exit0; touched functional modules
  are below400 LOC.
- `/tmp/ares-relative-e-axis-final-wasm.log` exit0 for core and browser
  adapter wasm32 compilation,5m25s including lock queue. No fresh browser,
  Windows or macOS runtime claim.
- Unrestricted final parity build exit0 preceded CPU0 runtime. Official
  `/tmp/ares-relative-e-axis-final-official.log` exit100/9.718s and seeded
  `/tmp/ares-relative-e-axis-final-seeded.log` exit100/11.361s each ran
  2 tests:single-tool infill-first complete baseline/modifier PASS; dual-tool
  complete baseline/modifier DIVERGENT, unchanged byte186/line7 and lengths.
  No fixture, comparator, guard or ignored-test change.

Official dual cases `case-PH5pPi`/`case-QSabZJ` under
`/tmp/ares-relative-e-axis-official-artifacts`, seeded
`case-IuzM3Q`/`case-su0zss` under the corresponding seeded root, retain whole
inputs and outputs. Both input hashes match the recorded baseline/modifier.
Four raw `cmp` checks exit0:official Ares outputs versus the preceding
registry stage, then seeded versus official. No normalization was used.
The internal counter prerequisite therefore does not repair the missing
printing-body transitions or convert the unchanged dual-tool RED to GREEN.

Preserve intermediate evidence:initial controls exit0/127 PASS/38.303s;
initial Clippy exit101 exposed too-narrow `plain_retract` visibility
(E0364/E0603). Its visibility was corrected to the owning `motion` module,
not public API, before final checks. Initial parity build also exited0, but
only the final repaired checks above are accepted as continuation evidence.
Every started process handle reached terminal. No independent coupled commit
or global printer/option completion is claimed.
Independent six-axis revalidation checked the terminal evidence and closed
this relative-E prerequisite repair list. It approved neither an independent
coupled commit nor milestone/global completion. Next required implementation
is the actual ToolOrdering producer and initial-tool consumers, followed by
per-tool buckets/transitions and unchanged complete dual-tool GREEN.

### Actual layer-tool producer

Continue the same public dual-tool RED, without changing its acceptance seam.
Port `ToolOrdering.cpp::collect_extruders` (661–815),
`generate_first_layer_tool_order` (517–582), `handle_dontcare_extruder`
(188–315), `tools_for_layer` and the used-ID union from
`collect_extruder_statistics` (921–953) to `gcode_emit/tool_ordering/`.
Build `initialize_layers` from original physical/support/raft Z values,
independently of the G-code schedule's already-paired Z values. Writer registration and empty-layer
emission consume this result immediately; delete the old object-config-only
registration and geometry-presence tool predicate.

Capture per-region extrusion-role presence before island flattening, borrowing
`LayerFillEntities` without copying paths. Upstream collection reads the first
entity of each fill collection, not every later path. Thin fills count through
the collections appended by `Fill.cpp:1361–1369`. Carry only that source-owned
role annotation through existing island/order compatibility shells. Raft
adapter roles enter the support collector, not object sparse-infill selection.

Included now: materialized per-region walls and four fill-role classes,
support/interface selectors and current-tool inheritance, source nonsoluble
support candidate selection, area-based first-layer preference, configured
first-layer sequence and one-based/don't-care normalization, actual used IDs.
Deferred within the still-open coupled milestone: ToolOrderUtils flush-cost
reordering/custom later-layer assembly, initial-tool runtime installation,
per-tool buckets and transitions. Adjacent per-copy purge overrides and
custom colour-change tool overrides remain the spec's separate scope; this
stage must not claim their parity. Sorted collected IDs are not a final
printing schedule, and the producer stage alone cannot be committed as
complete dual-tool output.

Independent review rejected the initial paired-Z reuse:original
1,1.00009,1.00013 can form two ToolOrdering groups but one twice-averaged
G-code group. The producer now directly traverses each object's original
layers and queries tools by their original Z, also eliminating repeated
whole-merged-list scans per object. Source parentheses
`-0.2 * (width / scale)` and `sqrt(matrix_len) + EPSILON` are preserved.
Initial checks/control/Clippy/parity builds remain intermediate because these
repairs were applied during compilation. Initial Clippy reported263 core
lib diagnostics (seven new nesting diagnostics); source-owned target-layer,
area-collection, first-layer preference and nonsoluble-candidate functions
remove the new nesting instead of suppressing it. Final frozen-source checks
must establish the warning count and runtime outcomes anew.
The next Clippy snapshot removed all seven new nesting warnings but exposed
one new raft tuple `type_complexity` diagnostic (257 lib/273 test). Reuse the
existing `RaftSchedule` by reference rather than inventing a wrapper or
suppressing it; widen only its `gcode_emit`-local consumer visibility.
Source `Config.hpp:1284–1285` additionally fixes percent width resolution to
`nozzle * value / 100`, not a rearranged product. Independent static review
closed those repairs. Only `repaired-*` frozen runs are final continuation
evidence; earlier `final-*` names are intermediate, not acceptance results.
Initial `check` exited101 (import and distinct classic/public role types);
corrected direct import and exhaustive role conversion `check-retry` exited0
in17.08s. These compile outcomes are not the public RED/GREEN result.

Frozen producer continuation: `repaired-clippy` exited0 (256 core lib /
272 core test diagnostics, no dead-code or new type-complexity diagnostic);
`repaired-controls` exited0,303/303 PASS in50.869s,6805 outside the filter,
including the unchanged spiral-vase Z-ramp regression. `repaired-wasm`
exited0 for core and adapter compilation only; no browser-runtime claim.
Unrestricted `repaired-parity-build` exited0 before either CPU0 reference
runtime started. Fresh rustfmt and diff checks exited0.
The original-worktree temporary checkout around12:20 is not evidence for
this isolated worktree; rerun public `infill_first` rather than relying on
that window. Independent review closes static/quality prerequisites and
the finite303-test evidence, not actual-results while reference runs remain
live, nor the coupled milestone or an independent commit.

Reference terminal results: official48 tests46 PASS/2 FAIL in236.074s;
seeded48 tests47 PASS/1 FAIL in278.491s (both exit100,281 outside filter).
Fresh `infill_first` PASS4.743s/6.480s. Dual baseline/modifier remain RED at
byte186,line7 with unchanged lengths; official casesrnUFxu/Y61wld and seeded
Ht8w9N/pjVYLZ. Their input SHA256s remain78eca1c5…/30a532e0…, and both
official RAW Ares outputs equal previous relative-E outputs by `cmp` exit0;
seeded RAW Ares outputs equal the official pair by `cmp` exit0.

The extra official Strong FAIL is preserved, not relabelled PASS:
case-qS6g9O,byte31174,line1218 (402612 expected/actual), initial native travel
X113.388 Y113.135 versus Ares X113.244 Y113.297. Seeded Strong passed,
case-1jnpDm; its RAW Ares output equals official RAW Ares (`cmp` exit0).
Diagnosis uses the unchanged whole-output seam; no geometry or assertion
change. Three direct unmodified official CPU0 runs of the *same saved input*
(SHA256 d86d29e64e19f6dc0541ae4a8b3f3c8f1a6252dd1e051960a0a52ff9fcf85097)
exit0, directories `/tmp/ares-strong-official-replay.uiqxXI`, `.fnEkMC`,
`.EKQrtE`. The first replay starts at X113.244 Y113.297 instead: original
versus replay first non-generator difference remains line1218. Therefore
single-CPU alone does not eliminate all official reference nondeterminism.
All three full streams passed the existing generator-only ordered-byte
comparator: `/tmp/ares-strong-strict-replay-artifacts/replay-summary.json`
inventory3/compared3/passed3, sweep exit0 in0.963s. Symlink inventory
`/tmp/ares-strong-strict-replay.MLBDVt` points each case at that identical
saved3MF and its respective native output. This is cached-stream evidence,
not three additional native test invocations. The original public Strong
focused rerun also PASS1.956s (summary1.957s). Initial failure remains in
the official48-test result; neither this finite evidence nor seeded47 PASS
closes the dual-tool milestone or the all-printer/option goal.
Independent six-axis revalidation independently verified all three identical
input SHA256s, unchanged comparator/replay source,3/3 manifests PASS and
focused public PASS, and closed this finite producer repair list with no new
must-fix. Reference provenance in cached replay manifests remains unknown;
do not infer the internal cause of official variation. Dual-tool RED and
remaining consumers stay open; no independent commit or milestone approval.

### Source flush-cost ordering

Continue the unchanged public inherited-second-filament RED (last frozen
official/seeded results above). Port `ToolOrderUtils.cpp:381–778`
greedy/Hamilton/one-layer forecast and two-group sequence assembly, with
`ToolOrdering.cpp:1235–1397` matrix selection/multipliers and custom sequence
precedence, `ParameterUtils.cpp:6–29` flattened layer-range interpretation.
Destination: `gcode_emit/tool_ordering/reorder/`, immediately consumed by
the materialized `ToolOrdering` after don't-care normalization. Preserve
source f32 costs, greedy integer truncation, strict DP/permutation ties,
128-bit source cache keys, earliest custom layer's group parity and reverse
range precedence. Non-BBL ordering uses one group even with physical
multiple nozzles. Included: output-affecting sequence ordering. Deferred
within this coupled milestone: initial tool, buckets/transitions/statistics.
Adjacent `FilamentGroup` automatic dual-BBL map optimization and tower
partition insertion retain the spec's explicit separate boundary; do not
claim their output parity. No new private test seam or weaker comparator.

Independent source review found no new algorithm must-fix within the valid
finite matrix/tool-ID domain. Initial check exited0; first Clippy snapshot
exited0 but reported261 lib diagnostics (five new nesting/int-plus-one
warnings), so it is not accepted. Move per-nozzle matrix construction to its
source-owned function and preserve C++ do/while pruning through explicit
permutation advance before continue. Compute the signed layer number before
range comparison. Re-review confirmed unchanged operation/permutation order.
Use source-width i32 predecessors in the flat DP cache, not wide Option
entries. Source greedy must select an unvisited candidate; no default-index
fallback. Only frozen `repaired-*` runs count as final evidence:
repaired Clippy exited0,256 lib/272 test, no new reorder/dead-code/type-
complexity diagnostic. Earlier `final-*` controls303 PASS53.786s and builds
exit0 remain intermediate because repairs overlapped their compilation.
Frozen rustfmt and diff checks exited0. Repaired WASM core/adapter compilation
exited0 (8m03s including queue; not browser-runtime evidence). Repaired
unrestricted parity build exited0 before CPU0 official/seeded48-test runs.
Repaired controls exited0,303/303 PASS52.393s,6805 outside the filter;
unchanged spiral-vase Z-ramp PASS9.552s. Independent review verified these
terminal quality/control/compile results, not reference results still live.

During official reference execution, painted manual Slim failed
case-0Y90Mn,byte40689,line1577 (516460 expected/actual bytes), native
X113.388 Y113.135 versus Ares X113.244 Y113.297. Preserve the first FAIL.
Saved input SHA2564be0ccab9b52a43b6fc56072610e9daaeeca67a94f7b77c4e16b3c7e8d876e1b
and RAW Ares output both equal the preceding producer case-nsDowv by
`cmp` exit0; RAW Ares also equals seeded case-WV0uGR (`cmp` exit0).
Three direct CPU0 unmodified official runs of this saved input exit0,
directories `/tmp/ares-reorder-manual-replay.VZw19r`, `.FlzRqC`, `.9OzJKK`.
Original versus first replay's first non-generator difference is line1577:
the two solid-fill paths swap order. No internal cause is inferred.
The unchanged complete comparator accepted all three streams, report
`/tmp/ares-reorder-manual-strict-artifacts/replay-summary.json`: inventory3,
compared3,passed3; sweep exit0 in1.117s. The input symlinks in
`/tmp/ares-reorder-manual-strict.vmd7qj` each target the identical saved3MF;
each reference links its respective native stream. Replay manifests retain
unknown producer provenance, so this is cached-stream comparison evidence.
Original public focused rerun exit0,1PASS8.298s (test8.296s). Neither focused
PASS nor three replays replace the initial official whole-batch failure.
Both48-test handles reached terminal exit100: official46PASS/2FAIL240.399s,
seeded47PASS/1FAIL283.423s,281 outside the filter. `infill_first`
PASS4.782s/6.422s. The dual baseline/modifier remain byte186,line7 RED with
unchanged lengths. Official cases5kmsT6/4wBrWO, seeded4oZg5Q/USV5Ac retain
the earlier input SHA256 pair78eca1c5…/30a532e0…; all four RAW Ares outputs
equal the preceding producer-stage counterparts by `cmp` exit0. No fixture,
comparator or scope gate changed. All started build/runtime handles reached
terminal; references were compiled unrestricted before CPU0 execution.
This stage adds source ordering production, not complete dual-tool emission.
Independent six-axis revalidation closed this finite ordering repair list
with no new must-fix after independently checking the saved-input hashes,
three unchanged-comparator replay manifests and retained unknown provenance.
The selected public tracer uses two logical filaments; these runs are not
public runtime coverage of the >5 Hamilton or >20 greedy branches, automatic
dual-BBL grouping, or the all-printer/option inventory. Static source audit
and successful compilation do not establish that broader output parity.
No independent coupled commit or milestone completion is approved. Next:
source initial-tool installation and all-tool startup writer/template state,
then per-tool buckets/transitions and statistics, unchanged full dual GREEN.

### Registered writer state before machine-start

Source `GCode.cpp:2791,2829,3118,3895–3933` registers actual ToolOrdering IDs
and initializes `PlaceholderParserIntegration` before the machine-start
render. Replace the adapter's late writer initialization and single-slot
`StartOutcome`/`StartRetractState` with that source ordering. Destination:
`gcode_emit.rs` writer initialization, `machine::append_start` consuming
the live `motion::EmitState`, existing `motion/writer_state.rs` refresh and
write-back. Machine-start uses every registered private counter, the source
relative-E/vector contracts, and ordered partial write-back; remove duplicated
slot0 state application and obsolete start-position plumbing through layers.
The unchanged public dual3MF complete-output RED remains the tracer; no new
private seam or weakened acceptance. Included here: startup writer ownership
and all-tool counter/template consumer. Initial tool/temperature/role options,
first/non-support placeholders and body buckets/transitions remain the next
coupled source slices, not silently claimed finished. Empty machine-start
retains its existing no-output path; no runtime guard or fallback is added.

The first check/control compilations exited101 because start_config still
imported the deleted parent value alias. Import the existing value module
directly; do not restore an unused compatibility alias. Those runs contain
no runtime evidence. Freeze after this repair and rustfmt. Final fmt/diff
checks exited0; final Clippy exited0,254 lib/270 test diagnostics, with no
dead-code/never-used/never-read diagnostics. Final WASM core/adapter check
exited0 (compilation only). Final unrestricted parity build exited0 before
CPU0 references. Final controls exited0,303/303 PASS53.433s,6805 outside
the filter. Independent review verified terminal quality/build evidence;
official/seeded48-test complete-output checks remain pending. No initial-tool,
body-emission, commit or milestone approval is implied.

Both reference handles subsequently ended exit100: official46 PASS/2 FAIL
236.818s, seeded47 PASS/1 FAIL282.280s,281 outside the filter. Preserve these
whole-batch results. Infill-first passed4.757s/6.420s. Unchanged dual baseline
and modifier remain byte186,line7 RED with the same182130/173811 and
270578/226756 normalized lengths. Official cases-iTrs8R/wxcHsS and seeded
cases-SEwCRT/wmulHL retain the78eca1c5…/30a532e0… input SHA256 pair. All four
RAW Ares outputs equal their preceding reorder-stage streams (each cmp0).

Additional official nested painted Slim failure case-LHmsQ1: byte13170,
line495,449764 expected/actual normalized bytes; native M73 P4 R24 precedes
G1 Z.2 rather than following it. Saved input SHA256
02de532e5b68744fc389e42817dcefb0b376f0dcd207b96fbf045c73e6a558ec and
RAW Ares both equal previous reorder case-pphKjQ; RAW Ares also equals
seeded passing case-NfGgf0 (cmp0). Three direct single-CPU official runs of
this identical saved input exited0: /tmp/ares-startup-nested-replay.7jbqsT,
.sUJI5s,.Cr6jYY. Original versus first replay's first non-generator
difference is line495, the M73/Z ordering above; subsequent solid-fill
path ordering also differs. No internal cause is inferred. Original complete
ordered-byte comparator accepted all three replay streams; report
/tmp/ares-startup-nested-strict-artifacts/replay-summary.json records
inventory3,compared3,passed3. The replay input links in
/tmp/ares-startup-nested-strict.nvt5fJ all target the saved input; references
link each respective native stream. Unknown cached producer provenance is
retained. Public focused rerun also passed; neither replaces the initial
official whole-batch FAIL. No fixture, comparator, coverage guard or private
test seam changed. All started build/runtime handles reached terminal.
Initial-tool installation, first/non-support placeholders, per-tool motion
options, body buckets/transitions and all-tool statistics remain open; this
is finite startup-state verification, not coupled milestone completion.
Independent six-axis revalidation closed this finite startup ownership repair
list with no new must-fix after checking the terminal logs, saved-input hashes
and three strict full-stream comparisons. It does not approve the coupled
commit or milestone; the remaining source consumers above are still required.

### Ordered startup filament metadata

Use the unchanged dual3MF RED from the preceding frozen official/seeded runs.
Port ToolOrdering.cpp:1000–1059 cal_non_support_filaments and GCode.cpp:
2776–2806,2834–2857 initial/first/non-support metadata into the existing
machine/first_filaments module, replacing configuration-ID scanning with
actual ordered layer tools. Expose a borrowed iterator from ToolOrdering;
do not clone geometry or create a second schedule. Store the resolved
metadata once in EmitState and insert it into every existing live template
refresh, preserving physical remap and source early termination. Initial
non-support falls back to initial printing filament only as upstream does.
Included: metadata production and its machine/layer/role/end template
consumers. Deferred within this same coupled milestone: physical initial
tool installation, tool-indexed motion/temperature options, body transitions
and final statistics. This does not substitute a header-only repair for
complete output. Keep existing mapping behavioral tests; add no private seam.
Independent static review confirms ordered first-per-slot scanning, source
early exit, all-support/no-map behavior and Integer placeholder types. The
Type2 wipe-tower skipped-priming branch at GCode.cpp:2776–2783 selects
all_extruders.back(), rather than the ordinary first ordered tool; retain this
explicit adjacent tower boundary for subsequent initial-tool wiring. No claim
of complete tower initial metadata or physical activation is made here.

First check exited101 on printer_model's old field path. First Clippy
exited101 on an unconstrained hotend closure index (E0282), before runtime.
Use printer.remaining.printer_model and an explicit usize index; preserve the
existing next-tool compatibility values rather than inventing a new
next-hotend mapping. Check retry exited0, but all overlapping final-* runs
remain intermediate (controls303 PASS55.616s, builds0). Only repaired-*
frozen runs count: controls303/303 PASS51.266s,6805 outside the filter,
spiral-vase Z-ramp PASS9.825s. Clippy exited0,254 lib/270 test diagnostics;
fmt/diff0; WASM core/adapter compilation0 (not browser runtime). Unrestricted
parity build exited0 before both CPU0 official/seeded48-test executions.
Reference results are pending; no independent coupled commit approval.

Both reference handles reached terminal exit100: official47 PASS/1 FAIL
238.222s, seeded47 PASS/1 FAIL281.564s,281 outside the filter. Infill-first
passed4.790s/6.423s; nested painted Slim passed6.648s/8.911s. The sole
failing public dual tracer still has byte186,line7 divergence with unchanged
normalized baseline182130/173811 and modifier270578/226756 byte lengths.
Official cases-N5ytGg/dc7VKU and seeded cases-qQuaMu/6bxAVA retain the prior
input SHA256 pair78eca1c5…/30a532e0…. All four RAW Ares streams equal the
preceding startup-writer counterparts (each cmp0). No new reference failure
needs diagnosis; no fixture, comparator or coverage guard changed. All started
compile/control/reference handles are terminal and reaped. This finite
evidence does not exercise every first-tool ordering, Type2 tower priming,
all-printer/option matrix or physical initial/body transitions. Next coupled
slice: tool-indexed MotionOptions and physical initial selection, then body
buckets/transition templates and all-tool statistics; unchanged dual full
output must become GREEN before coupled milestone commit approval.
Independent six-axis revalidation verified the terminal quality/control and
both complete reference results and closed this finite startup metadata
repair list without new must-fix. It explicitly leaves physical selection,
tool-indexed motion, Type2 priming, body transitions and statistics open;
no independent commit or milestone completion is approved.

### Tool-indexed motion and ordinary initial selection

Continue the unchanged dual3MF RED from repaired initial-filament runs.
Port Extruder.cpp:143–235 logical filament getters, physical travel_slope,
GCode.cpp FILAMENT_CONFIG consumers and GCode.cpp:3144,3227,7710–7746
ordinary initial writer selection into motion/options/build and the existing
first-chunk preamble. Filament-specific fields must consume the selected
logical ID; nozzle geometry/offset and travel slope consume its mapped
physical ID. Remove first-element-only consumers rather than keeping a
fallback copy. Preserve numeric order, get_at out-of-range-first semantics,
nullable override inheritance, machine-start unselected write-back, BBL
pre-render init versus compatible post-render selection and source E reset.
Included: selected-tool motion parameters and ordinary initialization;
deferred in this coupled milestone: multi-tool body buckets/transitions,
temperature sequences and final accounting. Type2 priming/tower ordering
remains the documented adjacent boundary. Public fixture/comparator unchanged,
no new private seam. New functional modules stay below400 lines.
Skirt's existing first-non-Nil planning consumer and its behavioral assertion
remain unchanged; selected motion instead uses get_at and Nil inheritance.
The preamble is moved into layer_gcode/startup.rs before adding tool selection
so the owning module remains below400 lines. Do not reinterpret the BBL
pre-render non-support initialization as its later set_extruder handoff when
initial/non-support IDs differ; that handoff needs the full coupled transition
consumer. No support-first or tower initialization parity is claimed here.

Independent static review found compatible normal startup ordered T/G92
before PA, contrary to GCode.cpp:7739–7747. Move PA before compatible
selection/toolchange only when the source multiple_extruders predicate is
false: GCodeWriter.cpp:78 tests largest registered ID>0, not tool count.
Retain late PA for BBL and the multiple branch (GCode.cpp:8048). Re-review
closed that concrete ordering issue. Overlapping final-* checks remain
intermediate: Clippy254/270 exit0, WASM0. Initial check exited101 when the
still-used skirt helper export was removed; restore that separate unchanged
consumer, not a selected-motion fallback. Check retry/final-check exited0;
remove the newly orphaned parent tags import after the startup split.

The existing exact KSR MotionOptions assertion actually failed at tests.rs:63
(final-controls log): source value0.05235987755982988, old expected
0.05235987755982989. Extruder.cpp:235 evaluates3*PI/180, whereas to_radians
multiplies by pre-divided PI/180. Replace only that expected expression with
the independently worked exact literal0.05235987755982988; keep strict
equality, source production operation order and the public fixture/comparator.
No new private seam is added. Only repaired-* runs after this final freeze
may establish terminal acceptance; prior overlapping runs do not.

Repaired frozen quality gates are terminal: fmt/check and git diff --check
exit0; Clippy exit0 with254 lib/270 test diagnostics; WASM core/adapter
compile exit0 (not browser runtime). The repaired unrestricted public
parity build exited0 before starting both CPU0 reference batches. Repaired
controls and official/seeded48-test full-output batches remain pending;
independent revalidation confirms the strict literal assertion and PA repair
but does not close runtime acceptance before those handles terminate.

Repaired controls reached terminal exit0:303/303 PASS55.220s,6805 outside
the filter. Exact KSR motion assertion PASS4.971s; spiral-vase Z-ramp
PASS9.156s. The official dual tracer has already retained byte186,line7
divergence with the unchanged baseline182130/173811 and modifier270578/226756
normalized lengths. Cases-ylGuRC/BI0g6t retain input SHA25678eca1c5…/
30a532e0…; both RAW Ares outputs equal the preceding ordered-metadata
cases-N5ytGg/dc7VKU (cmp0). Both48-test reference batches are still pending;
do not convert partial control passes into a complete reference result.

Official batch terminated exit100:46 PASS/2 FAIL256.801s. Besides the
unchanged dual tracer, Organic manual case-yNEibv diverged at byte6778,
line263: M73 and the next move exchanged order; normalized lengths both
1110784. Keep that initial failure, not a retrospective47/1 label. Its
input SHA256 is d131533c1b6e3db58359a954e27c372a7de6ab59e4e924f9d848e16f5babdfe2.
RAW Ares output equals previous ordered-metadata case-EEAPFq (cmp0).
The manifest explicitly reports runner_cache/unknown producer provenance;
this evidence alone does not identify an official native nondeterminism cause.
Diagnosing-bugs uses the unchanged public full-output RED and captured input;
no fixture minimization or comparator alteration is appropriate to this
complete-output contract. Ranked probes distinguish native output variation,
Ares regression, and reference-cache provenance without changing production.
Three direct official CPU0 runs on that same saved input all exited0 in
replay directories cqI1SA/TLS8Yq/ktSE0P. Original cached versus direct output
first differs at the same M73 order (excluding only generator date), with
later fill-order differences too. Existing strict replay sweep compares all
three complete outputs: inventory3/compared3/passed3, exit0,14.693s, report
/tmp/ares-tool-indexed-organic-strict-artifacts/replay-summary.json. Original
public Organic manual focused rerun PASS12.881s. No source, fixture or
comparator change was needed; retain the cached failure and unknown provenance
alongside these stronger saved-input results. Seeded batch remains pending.

Seeded batch terminated exit100:47 PASS/1 FAIL300.373s,281 outside the
filter; Organic manual passed. Its only failure is unchanged dual output.
Cases-MBjJqC/gd7nZ4 retain the same input SHA256 pair and both RAW Ares
streams equal preceding cases-qQuaMu/6bxAVA (cmp0). All four official/seeded
dual RAW comparisons are unchanged. All started controls, quality builds,
reference batches, three direct Orca handles, strict replay and focused
public rerun are terminal and reaped. Final diff --check exit0. No fixture,
comparator, coverage guard or production change occurred during the repaired
freeze. Independent six-axis terminal revalidation is requested for this
finite prerequisite, not coupled commit approval. Remaining same-milestone
work: full initial multi-tool set_extruder branch (including BBL non-support
handoff), per-tool body/object/island/region emission, transition templates,
temperature context and all-tool accounting. Type2 tower priming remains an
explicit adjacent source boundary. Neither all-printer/option coverage nor
full initial/body parity is established by these finite48-test results.

Independent six-axis terminal revalidation closed this finite prerequisite's
repair list without new must-fix: selected logical/physical ownership,
registry/options/offset synchronization, PA ordering and strict source-domain
travel-slope equality were verified alongside actual terminal logs. The
review retains official46/2 and seeded47/1, confirms Organic strict3/3 only
for those supplied streams, and states WASM evidence is compile-only. It
explicitly does not approve a coupled commit, complete milestone or full goal;
the unchanged public dual RED drives the next source-cited transition slice.

### Shared writer tool selection

Continue the frozen public dual RED (official/seeded input hashes unchanged).
Port GCodeWriter.cpp:547–585 toolchange_prefix/toolchange and Extruder reset_E
ownership into motion/tool_selection.rs, replacing startup-local installation
and command assembly. Use a shared registry/options/offset selection for both
BBL init_extruder and compatible toolchange, without cloning EmitState or
geometry. Preserve source max-ID multiple_extruders predicate, declared-
diameter compatible command gate, flavor/manual/BBL prefixes, comments and
forced reset_E semantics. Included: writer selection and its actual startup
consumer; deferred within this coupled milestone: GCode::set_extruder's
template/retract/temperature ordering and body tool buckets. Do not equate
the writer operation with the full GCode::set_extruder transition or claim
the public dual test is GREEN. Existing303 controls and unchanged dual/public
Normal/Tree references remain the acceptance seam; no new private test seam.

Writer module80 lines, startup185, registry238; no new geometry copies or
public API. Independent static six-axis review found no new must-fix in the
writer boundary, including no-command installation, max-ID predicate,
prefix priority, forced E reset and preserved PA order. Check exit0; frozen
controls303/303 PASS54.208s,6805 outside filter; Clippy exit0,254 lib/270 test
diagnostics; fmt/check and diff0; WASM core/adapter compile0. These are not
browser-runtime or full transition proofs. Unrestricted parity build remains
live; reference runs must start only after that specific handle exits0.

That unrestricted parity-build handle terminated0 before official/seeded
CPU0 runs. Both48-test batches terminated100: official47 PASS/1 FAIL235.020s,
seeded47 PASS/1 FAIL278.293s,281 outside filter. Only unchanged dual tracer
failed at byte186,line7, baseline182130/173811 and modifier270578/226756.
Official cases-Kc2AKB/phtd2B and seeded9Ops6R/4Fhbhf retain input SHA256
78eca1c5…/30a532e0…; all four RAW Ares streams equal preceding tool-indexed
options counterparts (cmp0). Organic manual passed this time; prior cached
failure remains recorded above. All started handles are terminal and reaped.

User explicitly requests local logical checkpoint commits now rather than
holding every prerequisite for dual full GREEN. This supersedes the earlier
no-coupled-commit gate only for local verified prerequisite checkpoints, not
mainline integration, push or a parity-completion claim. Split by compile
dependencies: layer-tool role producer/order/empty-layer consumer, registry
plus live template ownership and selected-tool options, then shared writer
tool selection. Reconstitute index-only intermediate files when adjacent
hunks share a file; never discard or rewrite the final working-tree content.
Check each staged snapshot independently and document its actual gates,
without assigning the final frozen runtime result to an untested snapshot.
Every commit includes source-cited documents and explicitly says public dual
complete output remains RED. Body buckets, full transition templates,
temperature context and all-tool final statistics remain incomplete.

Local checkpoint1 staged snapshot (producer) independently passed
cargo check --workspace --all-targets, exit0,53.53s; fmt --check and cached
diff --check0. Independent review matched all17 staged files to the exported
snapshot and found no new must-fix or unused writer-only scaffolding.
This snapshot has compile/format evidence, not a newly executed48-test
runtime claim. Later registry/metadata getters and writer installation are
staged only with their actual consumers. Dual complete output remains RED.

Checkpoint1 is local75699e7b. Checkpoint2 combines the actual registry,
relative-E/private-counter migration, common template refresh/write-back,
machine-start lifecycle, ordered initial metadata and selected-tool options
because the shared EmitState fields and template callers must change together.
It retains the previously verified startup-local compatible command shell;
checkpoint3 replaces that shell with the shared writer implementation. Its
exported registry snapshot is independently being checked; fmt0. Final
working-tree source remains untouched by index-only reconstruction. No claim
that this intermediate snapshot ran the final48-test batches is made.

Checkpoint2 registry snapshot reached terminal workspace/all-targets check
exit0,53.16s, with no warnings. Independent review matched all44 source files
to the snapshot, verified local startup has no dependency on the unstaged
writer module and found no new must-fix. Snapshot fmt0 and cached diff0.
Its freshly executed evidence is compile/format, not full runtime parity.

Checkpoint2 is localf07ce210. Checkpoint3 restores the exact frozen shared
writer-selection source, replacing the temporary startup-local shell. Only
motion.rs module registration, motion/tool_selection.rs and startup.rs differ
from checkpoint2; the remaining source already matches the final frozen
controls303/303 and official/seeded47/1 integration runs above. Export and
check the final staged snapshot too; no push or mainline integration is
authorized by this local checkpoint request. All unfinished body/transition/
temperature/accounting behavior remains explicitly open, not silently skipped.

Final writer staged snapshot independently passed workspace/all-targets
check exit0,53.35s, without warnings; snapshot fmt0/cached diff0. Independent
review matched its three source files and plan to the export, confirmed the
first two commit boundaries and found no new must-fix. The final shared writer
source is byte-identical to the previously frozen, runtime-verified integration
source. All three snapshot-check handles have terminated and been reaped;
none was restarted on a timeout. Local checkpoints preserve incomplete dual
full-output status and do not authorize push, main integration or full-goal
completion. Next implementation remains the actual per-tool body buckets and
GCode::set_extruder transition consumer, followed by all-tool accounting.

### Per-tool object/island/region body buckets

Continue the unchanged public dual complete-output RED from both writer
reference batches; fixture, assertion and comparator remain untouched.
Port GCode.cpp:4864–5130 ObjectByExtruder bucket construction, support
dontcare selectors, mixed-wall separation, and5241/5322/5434 per-tool
instance/island traversal, plus ToolOrdering.cpp:102–141 LayerTools::extruder
role selection. Rust destination: layers/tool_buckets with functional
object/support modules and actual layers.rs/object_entry/support_entry/
raft_entry consumers. Move owned paths into buckets; keep object/support/
raft geometry shared, and preserve canonical region IDs and island slots.
Filter support roles exactly as extrude_support6178–6242; do not emit the
same support body on every tool. Writer selection uses the already ported
writer operation at the outer tool boundary, not ad-hoc T words in entities.
Included in this slice: role-owned object/raft buckets and support role
dispatch, actual outer tool traversal and regional four-stage consumption.
Deferred within this same incomplete milestone: the higher-level
GCode::set_extruder template/retract/temperature side effects, per-tool
skirt/brim allocation, subset-specific instance chaining and final accounting.
These are explicitly not waived for complete dual parity. Wipe overrides,
tower integration and Type2 priming remain adjacent source boundaries.
Existing single-tool controls must remain passing; both full dual comparisons
remain the final closure criterion, even if this slice changes body bytes.

Independent six-axis review found three source-contract fixes before acceptance:
FillCollection dispatch now combines every child infill/solid predicate and
the collection role (the first-child rule belongs only to collect_extruders);
raft reuses the same support dontcare selectors and selected-role/ironing
passes; writer tool selection invalidates last_object_copy so the destination
object configuration is reinstalled even for consecutive tools on one object.
Read-only review confirmed these repairs statically. The initial controls303
run passed52.208s, but the four initial check/build handles overlap these edits;
all are intermediate receipts, not repaired frozen-source acceptance. Reap
each handle and rerun controls, quality and full public references after the
repair freeze. No comparator, dual fixture or assertion changed.

The mechanical extraction then exposed missing imported parent modules and
two stale timelapse_gate references; those failed check receipts are retained
as intermediate, not waived. Repair the imports and pass Input's timelapse
value. layers/tool_entry is the actual per-tool instance dispatch, not an
unused compatibility facade. Fixed [Option;2] wall dispatch avoids a new
small Vec allocation on every ordinary wall collection. Final static review
found no remaining must-fix in these repairs. Functional files are282/206/250
LOC or smaller. Repaired freeze: core check0; fmt0; diff0;303 controls all
PASS55.021s; workspace/all-targets clippy0 with253lib/269test warnings, no
dead-code diagnostics (prior254/270); WASM core+adapter check0. Both public
48-case batches and workspace nextest are still running, not accepted yet.
The unchanged dual assertion remains RED byte186,line7 in both references:
baseline182130expected/175043actual, modifier270578expected/233743actual.
Official artifacts M8Lafv/JJy2n6 and seeded cOjnk1/kNgoIj retain inputs and
complete files. Corresponding raw Ares files cmp0 across both references;
the modifier now emits97T0/T1 commands, as does native, but this is only a
body-dispatch diagnostic, not complete transition or output parity.

The first workspace run exposed sortable_fill_collections_flatten_before_
island_print_ordering at tool_regions.rs44 (region0 into an empty vector).
The compiled test executable reproduced exit101 in0.00s. Diagnosis compared
the test's collection_ranges[region0] with the actual fill producer at
fill_entities.rs129–142: the producer installs perimeter_ranges and
thin_fill_ranges for every canonical region, even when empty. This test
omitted that metadata. Supply the same empty region0 ranges, retain every
flattening assertion and add assertions for its preserved internal-solid
tool usage. No production fallback/defensive guard, no erased coverage and
no changed public3MF fixture or full-output assertion. The initial workspace
failure remains recorded; rerun focused+full tests and quality before commit.

Both frozen production reference handles now terminated and were reaped:
official48=47PASS/1FAIL290.398s; seeded48=47PASS/1FAIL395.665s. Only the unchanged
dual full-output test fails in both; all47 prior single-tool/region/support/
raft/tree controls preserve complete output. The region-range repair only
changes unit-test construction, not production/reference source. Workspace
initial run and repaired focused/full/clippy handles still need terminal
receipts. Do not accept an in-flight handle or erase the first workspace RED.

Initial workspace handle terminated/reaped:7555run,7554PASS37slow/1FAIL9skip,
264.377s; its only failure is the reproduced unit-test region-range omission
above. No other regressions; repaired full rerun remains mandatory/in flight.

Focused repaired nextest terminated/reaped:1/1PASS0.023s (test0.013s),7107
outside selection. Original flatten geometry/order assertions and new region
usage assertions all pass. Full repaired workspace and clippy still in flight.

Repaired region-metadata workspace terminated/reaped:7555/7555PASS35slow,
9skip260.126s. KSR initialized ordered bytes and spiral-vase body both pass.
The matching clippy run ended0 but269→271test warnings from the two new
single_range_in_vec_init literals; not accepted as final quality. Replace
those literals with equivalent once(0..0).collect() (no lint exemption),
then rerun full workspace+clippy on that final test-source freeze. Production
source/reference receipts are unchanged. Both final acceptance handles still
in flight: /tmp/ares-body-buckets-acceptance-workspace.log and -clippy.log.
No per-tool bucket commit yet; complete their receipts and independent review
before making the user-authorized local logical checkpoint.

Final acceptance clippy terminated/reaped exit0 (no dead-code diagnostics);
only final acceptance workspace handle remains live. Final static/runtime
review and logical local commit remain pending, not full transition closure.

Final test-source freeze acceptance workspace terminated/reaped exit0:
7555/7555PASS35slow,9skip255.465s. Final clippy0,253lib/269test, no new/dead
code warnings; fmt0/diff0. Production WASM0 and reference47/1 receipts remain
applicable (only equivalent test-range construction changed afterward).
All handles begun by this slice have terminated and been reaped; no timed-out
job was restarted. Request final independent six-axis receipt review before
the local logical checkpoint. The full dual assertion remains untouched RED;
GCode::set_extruder, per-tool skirt/brim, subset instance chaining and all-tool
accounting are still mandatory next slices, not waived or claimed complete.

Final independent six-axis review found no remaining must-fix and accepts
only this local checkpoint after fmt0 (terminated/reaped). Include all
bucket modules, registrations, consumers, metadata test repair and docs in
one logical commit; record both47/1 references and open boundaries in its
body. This does not authorize push or full dual/milestone/goal completion.

### GCode::set_extruder transition lifecycle

9bc65b17 is the verified local body-bucket checkpoint. Resume the unchanged
public dual RED: focused official nextest1FAIL2.460s, byte186,line7, retained
in /tmp/ares-set-extruder-red-official.log. Minimise diagnosis by observing
the first printing T0→T1 in the same public two-nozzle overhang scenario;
do not minimise away baseline/modifier complete-output acceptance.
Ranked hypotheses: missing set_extruder side effects/templates; incorrect
per-tool skirt/brim/instance ordering; incomplete usage/header statistics.
Native body before T1 contains old-tool wipe/retract, change_filament template
and new-tool temperature/PA; Ares still calls writer selection alone.

Upstream boundary GCode.cpp7710–8060 set_extruder and7666–7704 retract,
GCodeWriter.cpp toolchange/retract, PlaceholderParserIntegration3900–3933.
Destination gcode_emit/tool_transition functional modules plus shared template
render consumer and actual layers/startup callers. Included next vertical
slice: old filament_end and new filament_start render/write-back with correct
old physical/new logical IDs, live writer state, layer placeholders, EOL and
template failures. Next same-milestone slices: actual toolchange retract/wipe,
change_filament dynamic values/template, counters, manual/custom-T handling,
temperature/PA, cursor invalidation, then per-tool skirt/brim and accounting.
Adjacent tower/ramming/ooze planning remains source-cited deferred, not common
behavior waived for the Snapmaker fixture. Existing startup/template helper
code is a compatibility shell to share/replace, not a second transition owner.

Confirmed test seam remains public3MF export/import→slice output. Add a
comment-only filament-lifecycle tracer on the same native two-nozzle scenario
to isolate ordering around the first body T1. Its native-derived behavior
assertions are a diagnostic, not a complete-output comparator substitute;
the original baseline/modifier full assertions remain unchanged and RED until
all coupled source behavior is ported. Obtain tracer RED before production
edits, then port this actual lifecycle and rerun both references/controls.

Tracer compile attempts1/2 failed from an assumed ParityCase.actual field
and unavailable multi-thread Runtime::new; repair the test to call the public
slice_project API with the same deterministic metadata/context and the
existing current-thread runtime. These are setup failures, not behavior RED.
Unrestricted tracer build3 exit0; official tracer1FAIL2.790s and seeded
tracer1FAIL2.961s both pass native ordering assertions before failing Ares's
missing old-tool end before first printing T1. Retain complete raw/reference
artifacts D67TYd/PcYo9K and strict full-output DIVERGENT183192/175281bytes.
The original dual complete assertion remains unchanged RED. Tracer clippy0.
Initial actual lifecycle consumer core check0,17.53s; shared writer alone is
now wrapped with actual old-end/new-start render/write-back. Reproduce physical
ID mapping directly from Print.cpp3280–3286 (missing mapping returns0, not
get_at element0). Re-freeze after this source correction; any build started
before that correction is intermediate only, not final acceptance.

The initial lifecycle CLI build handle has terminated/reaped exit0 but its
source overlapped the mapping fix; keep it intermediate. Frozen native CLI
build,303controls,workspace/all-targets clippy and WASM core+adapter handles
are confirmed live, waiting/compiling on the shared target directory. Do not
restart on observation timeout. Await frozen build0, then run both lifecycle
tracers and original dual full-output comparisons before acceptance; independent
six-axis review is in flight. No new lifecycle commit, push or closure yet.

Independent review found two must-fix template-consumer contracts before
acceptance: (1) base_config's cut/EC singular aliases use index0; restore all
four for the current logical filament (old end/new start), matching
GCode.cpp8008–8013. (2) legacy [vector_option] expansion in template.rs263
also uses index0, whereas PlaceholderParser.cpp915 uses the explicit logical
filament argument to placeholder_parser_process. Pass an immutable logical
filament context through actual rendering; do not infer it from the end
template's physical filament_extruder_id, nor alter explicit modern indices.
Extend the public native tracer with differing per-tool alias/vector values
and obtain behavior RED on this lifecycle source before repairing those
contracts. Current comment-ID tracer alone cannot prove them. Frozen checks
already begun remain intermediate until these review fixes and final reruns.

Intermediate lifecycle quality receipts terminated/reaped: clippy0 with
253lib/269test warnings; WASM0; fmt0; unrestricted frozen CLI build0. These
do not close the two template-context review issues. Core303 handle54672
and official/seeded lifecycle+original-dual handles93567/77913 are confirmed
live and must be reaped before their results are assessed; their logs use
ares-tool-lifecycle-frozen-controls and ares-tool-lifecycle-intermediate-*
under /tmp. Keep unfinished coupled code/tests/docs in the worktree, not a
commit or push, until repair and independent revalidation.

### Local checkpoint status after the current reference round

The user requested logical local commits without waiting for complete dual
parity. The four existing checkpoints are 75699e7b (layer tool producer and
ordering), f07ce210 (live extruder registry and machine-start write-back),
5b2da82a (shared writer selection), and 9bc65b17 (actual body tool buckets).
Each commit message names its verified scope and explicitly retains the full
dual RED. Independent read-only review confirms no message amendment is
needed. The branch is four commits ahead of its own remote; none is pushed.

Intermediate handles54672/93567/77913 terminated and were reaped: core
303/303 PASS; each reference round reported the original dual RED and a
lifecycle tracer failure. Inspecting the saved complete Ares stream proves
the tracer failure was a test-boundary false negative: old end is immediately
adjacent to T1, and body[..tool] excludes the newline opening the T1 match.
Replace newline-containing substring matching with exact line matching in
that same pre-T1 interval. Marker contents and ordering remain unchanged;
neither full comparator nor original dual fixture/assertion is changed.
Independent review confirms this correction, subject to real reruns.

The unrestricted corrected-tracer build exited0. Fresh official and seeded
two-test runs are retained in /tmp/ares-lifecycle-line-{official,seeded}.log
with full artifacts under the corresponding -artifacts directories. These
handles58336/25536 terminated/reaped exit100: corrected lifecycle tracer PASS
3.761s official /3.396s seeded; unchanged original full dual FAIL5.001s
official /6.739s seeded. Each round has one PASS and one FAIL. Fmt and diff
checks exit0. These are intermediate lifecycle evidence, not final acceptance: the scalar alias
and immutable legacy-vector context review fixes remain open. Keep those
coupled files uncommitted; do not amend or combine the four verified commits.

### Selected filament aliases: public diagnostic loop

Extend the confirmed public3MF export/import seam with distinct EC values
3/7 and false/true. Retain native-first assertions on old end, new start and
the later old end after tool1 was selected; preserve full stream artifacts
and the original complete-output acceptance unchanged. The initial cut probe
exports printer vectors10/18 and false/true but native templates normalize
them to10/10 and false/false with the default cut override level. This is
fixture setup evidence, not a production RED. The calibrated native run
passes all native assertions, then fails Ares old-end bool rendering (0
instead of false),1.864s, /tmp/ares-alias-tracer-red2.log, full case-yHpKVs.

For distinct cut values, additionally select source EnableFilament(2) and
set per-filament cut overrides10/18,false/true. Verify effective project
configuration before using native output as the oracle. Do not infer runtime
values solely from the printer vectors or waive any alias assertion.

The EnableFilament public fixture passes all native assertions, then fails
Ares old-end aliases1.853s, /tmp/ares-alias-filament-red.log, case-t63Gc1.
This is the valid four-alias RED. Production now shares alias selection
between base context and live writer refresh, selecting the logical filament
and preserving bool output rather than footer numeric serialization. Initial
build failed only on helper re-export visibility (E0364/E0603); corrected
module visibility, reaped exit101, and rerun unrestricted build2. No behavior
GREEN, final verification, commit or push is claimed by this compile attempt.

Independent six-axis alias review found no new production must-fix, but
requested effective project read-back of EnableFilament(2) and both filament
cut override vectors. Add those checks before final runtime validation. This
fixture proves selected alias values/types, not override priority (printer
and filament cut inputs coincide). Legacy-vector indexing stays open. Build2
handle15920 and intermediate clippy96390 remain confirmed live; no restart
on timeout and no GREEN acceptance before terminal runtime results.

Unrestricted build2 handle15920 terminated/reaped exit0 after the effective
cut configuration assertions were added. Fmt/diff checks pass. Actual nextest
runs are started and confirmed live: official44435 and seeded89978 select
both lifecycle tracers plus the unchanged original dual; core controls80168
and workspace/all-target clippy96390 share the build directory. Logs are
/tmp/ares-alias-green-{official,seeded}.log, /tmp/ares-alias-controls.log and
/tmp/ares-alias-intermediate-clippy.log. These handles currently wait/run;
reap them, do not restart on timeout. No final GREEN or new commit/push yet.

Alias round receipts terminated/reaped: clippy96390 exit0,253lib/269test,
CLI31; official44435 has lifecycle3.731s PASS and aliases3.747s PASS, original
dual4.956s FAIL; seeded89978 lifecycle4.812s PASS, aliases2.842s PASS, original
dual6.657s FAIL. Each reference round is2PASS/1FAIL. Core303 handle80168 is
still confirmed live. Do not attribute these receipts to the following new
legacy-context test source.

Next vertical slice, same confirmed public seam: add native-first legacy
vector comments with EC3/7, explicit modern indices0/1, and an assignment
of mutable current_extruder=0. Validate first old end, new start and later
old end; the implicit legacy index must remain the immutable logical tool
argument, independent of that mutable symbol. Source boundary remains
PlaceholderParser.cpp::legacy_variable_expansion915 and GCode.cpp7773/8042.
After valid RED, pass the explicit logical context through rendering and its
recursive nodes; keep explicit modern indices unchanged. Existing default
render callers are not claimed to close remaining startup/end lifecycle
ports. Unrestricted legacy tracer build61755 currently waits on the shared
target while core controls80168 run; reap before native replay.

Independent review closes the finite four-alias repair after inspecting both
runtime logs and effective override read-backs, but not the overall lifecycle
or regression gate. Legacy tracer is statically valid; its context is only
independent of ordinary current_extruder assignment. Upstream explicit
filament_change() may modify parser context (PlaceholderParser.cpp1719–1722),
and is not exercised or claimed implemented by this slice. Await valid RED
before its render implementation; do not declare global context immutability.

Core30380168 terminated/reaped exit0,303PASS53.699s; original alias code
passes those controls. Legacy build61755 exited0, but the assignment probe
fails natively with process156 /Cannot modify a read-only variable/ for
current_extruder,0.962s in /tmp/ares-legacy-context-red.log. This invalid native
input is setup evidence, not a valid production RED. Correct the static review
assumption: current_extruder is a read-only symbol, not a writable ordinary
variable. Remove that illegal assignment; retain logical tool1's legacy7 and
modern explicit3/7 observations at start and later end. Legal tracer build
20641 exited0. Full contract still passes an explicit logical argument, not
the end-template physical filament_extruder_id. No render implementation
before the legal tracer's native-first RED is observed.

Legal native-first tracer9941 terminated/reaped FAIL1.865s: all native
assertions pass, Ares new tool1 legacy value is tool0's3 instead of7 while
modern explicit indices stay3/7. Retain complete case-umdixi and strict
DIVERGENT in /tmp/ares-legacy-context-legal-red.log. Implement explicit logical
filament context in template::render_for_filament and recursive rendering;
actual tool_transition passes its logical old/new argument. Existing render
callers retain their prior tool0 call context; remaining startup/end lifecycle
ports are still open, not a new fallback or completion claim. Unrestricted
green build is running; await runtime GREEN and independent revalidation.

Independent read-only six-axis review finds no additional must-fix in this
finite legacy index slice; actual template346LOC and test212LOC, no new
copies/cache/suppressions. Existing tool0 render call sites and explicit
filament_change() are not silently claimed ported. Final unrestricted CLI
build91937 terminated/reaped exit0. Frozen fmt/diff0; live checks queued/run:
clippy27940 (/tmp/ares-lifecycle-context-final-clippy.log), WASM32562
(/tmp/ares-lifecycle-context-final-wasm.log), workspace84768
(/tmp/ares-lifecycle-context-final-workspace.log). Start official and seeded
four-test nextest rounds selecting all three lifecycle diagnostics plus the
unchanged original dual; logs /tmp/ares-lifecycle-context-final-*.log and
full artifacts under the matching -artifacts directories. These process
handles must be reaped before asserting GREEN; no new commit/push yet.
Official65742 and seeded81696 are confirmed live with the three quality
handles27940/32562/84768; source is frozen while these checks run or await
the shared build lock. Observation timeout is not terminal or a reason to
restart any of these five processes.

Frozen quality handles terminated/reaped: clippy27940 exit0,253lib/269test
(CLI31),2m16s; WASM32562 core+adapter check exit0,2m33s. Workspace84768
and focused official65742/seeded81696 remain confirmed live. Keep source
frozen. Broader unchanged Normal/Tree/region finite references are additionally
started with all twelve activation flags and strict complete comparisons:
official22081 /seeded9538, logs /tmp/ares-lifecycle-context-controls-*.log,
full artifact roots with the same prefix and -artifacts suffix. This batch
includes the new lifecycle diagnostics but does not redefine original dual
full-output acceptance or imply all-printer/all-option coverage. Await all
terminal results before independent runtime review and any local commit.

Focused official65742 and seeded81696 terminated/reaped exit100, each3PASS/
1FAIL. Official lifecycle15.240s, legacy16.038s, aliases15.458s all PASS;
seeded21.672s/19.756s/15.820s all PASS. Original full dual remains unchanged
FAIL17.365s official /25.365s seeded, still header byte186,line7. These are
actual activated native-first diagnostics, not the workspace's env-gated
early-return receipts. Independent runtime revalidation requested for the
two finite context fixes only. Workspace84768 (7558 tests) and broader
51-case controls22081/9538 remain confirmed live; do not commit or claim
global parity from these focused windows.

Independent reviewer inspected the actual focused logs and closes both
finite must-fixes (all four aliases and legacy logical default index), with
the lifecycle window also PASS in both references. Approval is explicitly
not all GREEN: each round3PASS/1FAIL, clippy retains253/269 warnings and WASM
is compilation only. No commit or full dual closure until workspace84768
and both broader51-case batches22081/9538 terminate and are revalidated.

Frozen workspace84768 terminated/reaped exit0:7558/7558PASS,36slow,9skip,
260.337s, /tmp/ares-lifecycle-context-final-workspace.log. Keep the activated
native-reference receipts separate from env-gated workspace tests. Both
51-case reference handles22081/9538 still run; their terminal totals and
independent runtime review remain required before a local checkpoint.

Final broader reference handles22081/9538 terminated/reaped exit100:
official50PASS/1FAIL378.832s, seeded50PASS/1FAIL425.235s. Both batches
preserve all47 existing strict complete-output controls and pass all3 new
native-first lifecycle diagnostics. The only failure is the unchanged public
dual baseline/modifier complete assertion,10.850s /14.108s respectively.
Do not label the diagnostic3 as complete-output parity cases. Final fmt/diff
checks exit0, all current validation handles are terminal/reaped, and the
source remains frozen. Submit these final receipts for independent six-axis
approval of a local lifecycle checkpoint only; no push/full dual closure.

This checkpoint includes old-end/new-start ordering, physical end/logical
start IDs, live template write-back, per-tool cut/EC aliases and explicit
logical legacy-vector context. Required remaining source slices still include
startup/end callers, full toolchange retract/wipe/counters/custom change and
temperature/PA handling, per-tool skirt/brim and subset instance ordering,
and all-tool output accounting. They stay open in this milestone; a local
checkpoint does not waive them or close all-printer/all-option scope.

Final independent six-axis review found no new must-fix and approves only
this finite code+docs local checkpoint. Commit body must retain full dual RED
and the remaining startup/end/full-transition/per-tool allocation/ordering/
accounting scope. No push/main or complete multi-tool/all-printer claim.

## Printing toolchange retract/wipe slice

The lifecycle checkpoint is committed locally as 9604a0b5. Continue the same
milestone, retaining its original full baseline/modifier assertion. Port
`GCode.cpp::set_extruder` 7751–7755 and `retract` 7652–7704,
`Wipe::calculateWipeRetractionLengths` 310–358,
`GCodeWriter::retract_for_toolchange` 1015–1023, and
`Extruder::retract_length_toolchange` / `retract_restart_extra_toolchange`
205–212 into the existing motion/travel sequence and tool_transition consumer.
The two toolchange getters use physical extruder identity, unlike ordinary
filament retraction. Reuse the live counters and wipe geometry; do not swap
options temporarily or copy motion state. Included: old-tool retract/wipe,
restart-extra assignment, reset_E, lazy lift and unconditional wipe-path
reset before old-end. Subsequent eager/change template, temperatures/PA,
startup/end and allocation/accounting remain open required source slices.

TDD seam is the previously approved public exported 3MF -> slice bytes with
native-first observations; no private-state injection. A new tracer changes
only public toolchange lengths to 6/9 and extras to 0.25/0.5, verifies their
export, then reads old-tool counters through legal end-template placeholders.
Strict full artifacts are retained as DIVERGENT rather than replacing the
original full-output acceptance. Require native-first RED, then actual GREEN,
unmodified controls and independent six-axis review before checkpointing.

Initial command31287 terminated exit100 before invoking Orca because the
artifact variable was misspelled; setup failure, not RED. Corrected command
15235 invokes native output but fails the second native assertion: exported
6/9 and 0.25/0.5 become runtime6/6 and0.25/0.25 in this printer variant.
Retain case-APsYec and explicit native config assertions; this is not unequal
per-nozzle coverage. Calibrated command13592 terminated/reaped exit100,
1.870s, /tmp/ares-toolchange-retract-calibrated-red.log, case-IiB4n3. Native
assertions all pass; Ares old-end reports0/0 instead of6/0.25. This is valid
public-seam RED before production changes. Implement mode-aware shared
retract/wipe, physical toolchange getters, old-tool lazy lift and unconditional
wipe reset; subsequent eager/custom change remains open.

First GREEN85701 terminated/reaped exit0, native-first test PASS1.877s.
Independent review found a required semantic fix: Hilbert suppression reads
`GCode::on_first_layer` (`GCode.hpp:670`), not m_layer_index. Carry the actual
current layer's id/bottom-Z projection in EmitState, initially from the
source-leading object-or-support chunk layer (`GCode.cpp:4559–4591,4664`),
then refresh at each object/support/raft emission matching m_layer changes.
Use this only for the new toolchange Hilbert gate; lift enforce's chunk-index
contract remains unchanged. The current public tracer is not evidence of
mixed-object/elevated-support Hilbert coverage. Quality checks88026/59052/
84353 were interrupted deliberately and reaped exit130 before this fix;
do not count them as completed validation. Rerun final checks on frozen fix.

Independent static revalidation closes the on_first_layer fix with no new
must-fix. Frozen official focused30599 terminated/reaped exit100:4PASS/1FAIL
5.233s, new retract tracer4.636s. Failure is solely the unchanged original
dual complete comparison; all three previous template diagnostics also pass.
Final clippy15789 terminated/reaped exit0,2m12s,253lib/269test warnings and
no dead_code matches; final WASM82924 core+adapter check terminated/reaped
exit0,2m29s. fmt/diff checks exit0. Workspace26732, seeded focused67346,
official broader37466 and seeded broader13903 remain confirmed live. Each
broader batch includes the47 prior strict complete-output controls,4 limited
native-first diagnostic tests, and unchanged original dual contract. Await
terminal results and independent actual-log review; no full/parity or commit
approval from these partial receipts.

Concurrent workspace26732 has a real FAIL in the unmodified command-lifecycle
test timeout_returns_before_descendant_closes_inherited_output_handles:
bounded descendant did not finish (5.864s). Preserve /tmp/lifecycle-Ydrjsu,
command-m7Bcnu; direct child SIGKILL/reaped after500ms, all snapshot/live
streams0bytes, no direct-child.pid or descendant finished marker. Independent
read-only review supports a helper startup-path blockage hypothesis but does
not establish scheduling as its cause or prove no descendant existed. Do not
label this known or count this full round as PASS. Single-case diagnostic21960
is running with original assertions. After all current handles terminate,
rerun unchanged full workspace without external Orca batches; preserve both
the initial result and clean rerun evidence, and require revalidation.

All first-round handles are now terminal/reaped. Workspace26732 exit100:
7558PASS/1FAIL,279.307s, solely the lifecycle helper above. Isolated21960
exit0/PASS3.065s with unchanged assertions; this supports environment
sensitivity, not erasure of the initial FAIL. Seeded focused67346 exit100:
4PASS/1FAIL257.291s, solely original full dual; new retract196.914s.
Broader37466/13903 exit100: each46PASS/6FAIL273.987s/280.284s. In addition to
the original full dual, default auto Organic and manual default/organic/grid/
snug styles differ. These first invocations mistakenly restricted only the
Orca child, not the entire replay; they do not implement the already-approved
whole-process CPU0 plan above. Ares Organic generation workers() reads
available_parallelism; merge_areas.rs chooses2- or4-leaf buckets from that
count. Previous saved PASS case-OGDQvB and new case-1BbebX have identical
project_settings.config and Orca body (after date header), but Ares support/
skirt geometry differs. This is evidence of an execution mismatch, pending
the actual corrected replay, not permission to waive output assertions.

Source remains frozen. Start corrected whole-process taskset -c0 official/
seeded52-test controls, retaining both original and corrected logs/artifacts
at /tmp/ares-toolchange-retract-{controls,cpu0}-{official,seeded}*. No new
reference patches or production single-worker clamp; both consumers run at
the previously approved same one-CPU replay boundary. After these handles
terminate, run full unchanged workspace without concurrent external batches.

Corrected whole-process CPU0 official36432 terminated/reaped exit100:
51PASS/1FAIL250.346s. All47 prior strict complete-output controls and4
native-first diagnostic windows PASS; only original full dual FAIL5.048s.
Each of the five Organic cases that failed under the mismatched execution
domain passes without source/input/assertion changes. Seeded89586 remains
live, with those five cases already PASS; await its complete terminal totals
and the subsequent clean full-workspace run before local commit approval.

Corrected seeded89586 also terminated/reaped exit100:51PASS/1FAIL296.345s,
only original dual FAIL6.706s. All47 strict complete controls and4 diagnostic
windows pass; the five Organic domain mismatches are resolved in both
references without production changes. Keep the mismatched domain failures
as evidence, not reference nondeterminism or ignored tests. All native batch
handles are now terminal/reaped. Rerun the unchanged lifecycle test alone,
then the entire unchanged workspace without any external Orca batches.

Independent actual-log revalidation closes all five suspected Organic
regressions in the approved shared CPU0 domain only; it does not establish
multiworker determinism. Clean lifecycle22443 terminated/reaped exit0,
PASS3.026s with original assertions after all native batches ended. Now run
clean full workspace97216 alone, /tmp/ares-toolchange-retract-clean-workspace.log.
No source changes since the reviewed first-layer fix; final Clippy/WASM/fmt
receipts still apply. Do not replace the earlier 7558PASS/1FAIL workspace
record or call the remaining live full process complete.

Clean full workspace97216 terminated/reaped exit0:7559PASS,9SKIP,
35slow,256.465s. All tests ran with unchanged configured parallelism and
without concurrent external Orca batches. Log:
/tmp/ares-toolchange-retract-clean-workspace.log. This closes the current
checkpoint's workspace gate; retain the earlier overloaded 7558PASS/1FAIL
record and both unchanged lifecycle reruns rather than recasting that run
as successful. Final fmt76559 exit0 and git diff --check exit0; final frozen
source Clippy15789 exit0 (253lib/269test warnings, no new dead code) and
WASM core/adapter82924 exit0 remain applicable.

Independent static six-axis revalidation found no new must-fix and confirmed
the real first-layer versus lift-counter separation. Final runtime approval
is requested against the actual clean workspace receipt. The local code+
documentation checkpoint remains limited to printing toolchange retraction,
wipe reset and lazy normal lift. Official/seeded shared-CPU0 each51PASS/1FAIL
does not close full dual-tool parity: the original complete-output comparison
remains RED. Startup/end callers, subsequent transition behavior, ordering/
accounting and the global printer/option matrix remain open; no push or main
merge is claimed by this local checkpoint.

Independent final six-axis review verified the actual clean terminal log
and approved this finite code+docs local checkpoint with no new must-fix.
Approval excludes push/main merge, complete transition/accounting parity
and global goal completion; original full dual output remains RED.

## Scheduled filament header IDs

Current source is checkpoint1f15f498, clean before this slice. Fresh unchanged
full baseline/modifier native replay3273 exit100,2.472s; byte186,line7 expects
`; filament: 1,2`, emits `; filament: 2`. Retain
/tmp/ares-tool-transition-current-red.log and its artifacts. Hypotheses:
initial-only selection, first-layer-only selection, or lost logical identity.
Inspection falsifies those: header.rs serializes the configuration usage
COUNT, not IDs. Upstream `GCode.cpp:2623–2632` serializes the actual
`Print::get_slice_used_filaments(false)` IDs, populated by
`Print.cpp:2514` from `ToolOrdering::all_extruders`.

Same approved public 3MF import/slice/output seam: add a native-first header
tracer using the unchanged two-filament export helper. Obtain its RED before
changing production; then borrow schedule.tool_ordering.all_extruders() in
header.rs and format one-based logical IDs. Include only scheduled-ID header
output; existing config inventory is not removed or reused as geometry truth.
Statistics/transition/all-printer/all-option coverage remain deferred/open.
Require both references' tracer GREEN, original full comparisons retained,
workspace/fmt/clippy/WASM and independent six-axis checkpoint review.

Header tracer4807 terminated/reaped exit100,1.778s: native header assertion
passes first; Ares emits `2` instead of `1,2`. Log
/tmp/ares-filament-header-red.log. This is valid public-seam RED before
production edits. Borrow the existing Schedule in append_header, retaining
five arguments and avoiding a new too-many-arguments warning. Keep max-Z,
layer-count and header-disable behavior unchanged; serialize the schedule's
borrowed all-tools list directly, no new ID inventory or geometry copies.

Source frozen after implementation. Build73535 exit0,1m15s; fmt16401/88722
exit0; diff check0. Clippy43533 exit0,2m27s, unchanged253lib/269test warnings;
WASM core+adapter89995 exit0,2m43s (compile, not browser runtime).
Independent static six-axis review found no new must-fix and confirmed ID
ownership, unchanged count/header-disable behavior and no new argument warning.

Official shared-CPU0 broad65918 terminated/reaped exit100:52PASS/1FAIL,
254.196s, /tmp/ares-filament-header-official.log and matching artifacts.
New header diagnostic PASS3.601s, all47 strict complete-output controls and
the4 prior limited diagnostics PASS. Only original full dual FAIL4.969s:
baseline's first difference advances to byte998,line33 total_toolchanges16
versus0; modifier first differs at byte718,line25 M73 P0 R17 versusR16.
Header correction does not close these remaining runtime/accounting gaps.
Seeded64723 remains live; require its terminal totals and a subsequent full
workspace alone before local commit approval. No comparator/fixture changes.

Seeded64723 terminated/reaped exit100:52PASS/1FAIL,300.331s. Header tracer
PASS4.725s; all47 strict complete controls and4 prior diagnostics PASS.
Only original full dual FAIL6.686s, with the same new first differences.
Log /tmp/ares-filament-header-seeded.log and matching artifacts retained.
All external batches are terminal. Start unchanged full workspace alone,
/tmp/ares-filament-header-workspace.log; source remains frozen. Do not claim
workspace GREEN or final checkpoint approval before its terminal receipt.

Full workspace14113 terminated/reaped exit0:7560PASS,35slow,9SKIP,
257.593s. Original configured num-cpus parallelism, no concurrent external
Orca batches, no assertion changes. /tmp/ares-filament-header-workspace.log.
Frozen-source fmt/diff/clippy/WASM gates and both actual52/1 reference rounds
are complete. Request final independent six-axis approval for this header-ID
code+test+docs checkpoint only; full dual remains RED and no push/main merge
or global printer/option completion is implied.

Independent final six-axis review verified workspace's actual terminal
receipt and approved the finite header-ID code+test+docs local checkpoint,
with no new must-fix. Approval excludes push/main merge and complete dual
or global goal closure. Next original full-output gap is schedule-derived
total_toolchanges and remaining transition/time/extrusion accounting.

## Non-tower scheduled total tool changes

Start checkpoint11742228, clean. Fresh unchanged public full replay18814
exit100,2.473s, /tmp/ares-tool-count-current-red.log and matching artifacts:
baseline byte998,line33 total_toolchanges16vs0; modifier byte718,line25
M73 R17vsR16. Hypotheses: uninstalled prediction, missing cross-layer changes,
or physical/logical confusion. Inspecting placeholders.rs confirms prediction
still defaults to0; upstream owner is `GCode.cpp` DoExport's
total_toolchanges_from_ordering2289–2300/resolve_total_toolchanges2304–2309.

Same approved public 3MF-to-output seam, native-first diagnostic in
compatible_regions/tool_templates.rs: comment templates expose integer
total_toolchanges and division-by3, and compatible summary must report16.
Obtain actual RED before changing production. Port one-pass ordered logical
transition counting (skip first/repeated IDs, preserve last across layers).
Compute once in existing InitialFilaments immutable export shell, install
Integer through shared metadata refresh and reuse value in positive-only
finish.rs summary. Keep source sequential empty global ordering prediction0;
no new support guard or output assertion relaxation. Tower count precedence,
dynamic toolchange_count and other accounting remain deferred/open.

Independent read-only boundary review agrees with this metadata owner and
requires the above logical/sequential/tower distinctions. Verify tracer and
prior strict full-output controls against both references, retain original
dual RED, then workspace/fmt/clippy/WASM and final six-axis local review.

Red build20940 exit0,11.42s. Native-first tracer99828 terminated/reaped
exit100,1.848s, /tmp/ares-tool-count-red.log and case-n5Et0N. All native
template and summary assertions pass; Ares emits `0 0` instead of `16 5`.
Implement only after this valid RED: ToolOrdering one-pass prediction,
InitialFilaments immutable Integer installation and shared positive-only
compatible tail summary. Aggregate weight/cost arguments into their existing
pair to retain five finish parameters without new argument-count warnings.

First implementation patch matched the earlier first_non_support call rather
than the struct initializer: fmt37464 exit1 and build69908 exit101 (parse
error/missing field), /tmp/ares-tool-count-build.log. Both terminal before
correction. Move the initializer to its intended struct; these are compile
failures, not a new behavioral RED or a verified source snapshot.

Corrected fmt5841 exit0 and build88964 exit0,1m15s. Independent static
six-axis review finds no new must-fix and confirms logical transition
semantics, sequential0, shared Integer/summary owner and deferred tower
precedence. Source frozen. Clippy13091 exit0,2m17s, unchanged253lib/269test
warnings; WASM core+adapter79595 exit0,2m33s (compile only). Final fmt56876
and diff checks exit0. Prebuild all workspace tests55201 with unrestricted
CPU before approved whole-process CPU0 replays, avoiding single-CPU compile
cost; this build remains live and is not yet accepted as complete.

Prebuild55201 exit0,2m47s. Official84634 terminated/reaped exit100:
52PASS/2FAIL257.274s; seeded50673 exit100:53PASS/1FAIL303.498s. New total
diagnostic PASS3.748s/4.918s, all prior windows pass. Original dual remains
RED: baseline now first differs byte1726,line124 (native M73 P6 R14 versus
Ares G92 E0); modifier remains byte718,line25 M73 R17vsR16. Actual logs
/tmp/ares-tool-count-{official,seeded}.log and matching artifacts retained.

Official also fails manual Snug painted at byte6778,line263: M73 P3 R28
falls before rather than after the next motion. Do not erase this FAIL or
report official53/1. Case-mz42Be current FAIL and prior header slice official
case-0cJAce PASS have byte-identical input.3mf (SHA9802720d91bebeb8c8789462512e99e1f31699fb0c2eb04d6462adc1b0b0171d)
and byte-identical Ares.gcode (SHAb55b8c274069dfb712f0333c84d6f1eeb0015b9a83c41eae7d0f7d4a322431e1),
both cmp exit0. Native outputs differ after excluding generated-date line:
first M73 position, then reordered small solid-fill blocks around original
line2143, followed by more shifted M73 positions. Thus not an Ares-output
regression from this slice; native cause is not established. Current seeded
PASS case-urSYzY has the same Ares SHA. Preserve all raw artifacts/provenance
limits; do not alter geometry or filter M73 in the acceptance comparator.
Run unchanged public Snug official replays alone, then full workspace alone.

Independent review confirms byte-identical input and Ares output against the
prior PASS, and actual native differences; do not infer a specific native
mechanism. Unchanged official public Snug replay72604 exit0/PASS3.149s,
case-Ab0lbo; replay86788 exit0/PASS3.135s,case-wE1Fbd. Both replay input.3mf
and Ares.gcode cmp exit0 against failing case-mz42Be; original full comparator
PASS without filtering M73 or geometry. Logs
/tmp/ares-tool-count-snug-official-replay{1,2}.log and artifacts retained.
These separate exact-input PASS receipts do not change official batch52/2.
All external batches/replays terminal/reaped; start full unchanged workspace
alone, /tmp/ares-tool-count-workspace.log. Source still frozen. Await actual
terminal summary and final independent six-axis approval before checkpoint.

Full workspace84987 terminated/reaped exit0:7561PASS,35slow,9SKIP,
258.345s, /tmp/ares-tool-count-workspace.log, unchanged configured num-cpus
parallelism and no external batches. Source stayed frozen; fmt/diff/clippy/
WASM receipts remain applicable. Request final independent six-axis local
code+test+docs approval. Keep official52/2 and seeded53/1 plus the separate
exact-input official Snug2PASS receipts; complete dual remains RED and neither
reference internal cause nor tower/dynamic/full accounting is claimed closed.

Independent final six-axis review verified actual7561PASS and all retained
reference/replay receipts; approved this finite non-tower prediction
code+test+docs local checkpoint with no new must-fix. Approval excludes
push/main merge, complete dual parity, tower precedence, dynamic counters
and global printer/option completion.

## Printing pressure advance restoration

Start clean b1e2558c. Fresh unchanged full dual replay63049 exit100,
2.467s, /tmp/ares-transition-current-red.log; baseline case-LgubvM byte1726
line124 M73 position, modifier case-hO9U0g byte718 line25 M73 remaining
minutes. Three hypotheses distinguish absent custom transition blocks,
writer lift/motion state, and independent timing errors. Raw baseline diff
shows all three remain plausible: missing printer custom block, SpiralLift,
temperature and pressure-advance commands plus changed motion speeds.
Do not fix or filter M73 independently to claim closure.

Select the independently bounded fixed PA command consumer first:
GCode.cpp::set_extruder8048–8053 and GCodeWriter.cpp::set_pressure_advance
370–391. Existing startup.rs formats it correctly for this fixture but
tool_transition never calls it. Move that existing formatter/getter unchanged
to one shared functional module, preserve startup callers, and append after
printing filament-start. Adaptive PA processor state, custom change template,
eager lift, temperature and complete dual accounting remain open.

Public native-first outputs helper, unchanged public 3MF seam: insert a
comment-only filament-start marker and assert each of sixteen printing
markers is followed by M900 K0.02 before its object body. Build93646 running;
obtain behavioral RED before production edits. Original complete comparator
still runs and retains its divergent artifacts. Verify both references,
unchanged finite complete controls, workspace/fmt/clippy/WASM and independent
six-axis read-only review before a finite local code+test+docs checkpoint.

Red build93646 exit0,11.79s. First replay8142 exit100,1.812s is NOT valid
behavioral RED: native scanning also matched template text in trailing config
and unwrapped a missing object. Production still untouched. Limit this new
diagnostic to LAYER_CHANGE..EXECUTABLE_BLOCK_END, rebuild47274 exit0, then
replay53064 exit100,1.822s: native all16 windows and count pass first, Ares
fails missing pressure advance. /tmp/ares-transition-pa-red2.log and
case-hdYfHr retain actual RED. Only then extract existing helper mechanically
and call it after new printing filament-start.

Independent preflight approves this limited consumer boundary, not acceptance.
Review identified pre-existing formatter scientific-notation gap (e.g. valid
PA1e-5), decimals=0 trimming (large values require input reachability evidence),
and BBL-first flavor priority mismatch. Keep them explicitly open for a
subsequent source-cited writer-format slice; this checkpoint proves J1 PA0.02
and unchanged prior startup controls, not all PA values/flavors. No adaptive
PA reset implementation or full dual closure is claimed.

Source frozen after mechanical extraction and consumer. Static six-axis
review has no new finite-scope must-fix, requires actual runtime receipts.
Clippy70660 terminated/reaped exit0,1m20s, unchanged253lib/269libtest and31
CLI parity warnings; WASM86997 exit0,1m37s, core+adapter compile check only.
Final fmt/diff checks exit0. Start full workspace prebuild with unrestricted
CPU after quality checks, then approved whole-process CPU0 reference replays.

Full native test prebuild53801 terminated/reaped exit0,2m47s. Start official
42698 and seeded82586 unchanged broad controls plus new tracer:55 tests each,
all12 explicit reference activation flags, approved whole-process CPU0.
Logs /tmp/ares-transition-pa-{official,seeded}.log and matching artifacts.
Do not infer completion from live tail output; await both terminal receipts,
then run workspace alone without native references competing for CPU.

The CLI-only replay target nevertheless rebuilds its ares_core feature
artifact after workspace prebuild; batch logs initially show compilation,
not test progress. Preserve live handles. Validated rustc1568815 alone is
compiling this worktree's ares_core (feature set empty), affinity0. Expanded
only that compiler and its threads to CPUs0–63 to finish the build; cargo,
nextest and future reference/Ares runtime children remain constrained to0.
No production worker clamp or replay-domain change. Future prebuild must
also match the exact CLI target feature set, not assume workspace cache hit.

Both reference batches terminated/reaped: official42698 exit100,
54PASS/1FAIL261.269s; seeded82586 exit100,54PASS/1FAIL308.969s. New PA
diagnostic PASS3.670s/4.808s, all7 limited diagnostics and all47 prior strict
complete controls pass both references. Original complete dual remains the
sole RED: baseline byte1726,line124 M73 placement and modifier byte718,
line25 M73 remaining minutes; e.g. official case-n4HtYF/case-G7DQfn raw
outputs retained. Expected/Ares lengths baseline182130/175924 and modifier
270578/237223; no comparator relaxation. Earlier official Snug uncertainty
receipt remains historical, not erased by this batch's passing control.

Verified both live nextest parents1568795/1568821 affinity0 after compiler
exception. Reference runtime domain stayed single CPU. All external batches
ended before full unchanged workspace starts alone, with configured num-cpus
parallelism, /tmp/ares-transition-pa-workspace.log. Source remains frozen;
await actual full terminal receipt and independent final six-axis approval.

Full workspace81077 terminated/reaped exit0:7562PASS,35slow,9SKIP,
255.408s, /tmp/ares-transition-pa-workspace.log. No external reference
batches overlapped this run. The prior spiral-vase ramps-Z regression test
passes10.572s. All changed Rust files remain below400lines. Source stayed
frozen; retained fmt/diff/clippy/WASM receipts apply. Request final independent
six-axis finite local checkpoint approval; not push/main or complete dual,
all-printer/option, adaptive PA or formatter-domain closure.

Independent final six-axis review verifies actual7562PASS and reference54/1
receipts, approves finite shared fixed-PA consumer code+test+docs LOCAL
checkpoint with no new must-fix. Approval excludes push/main merge, full dual
or global completion and all explicitly deferred formatter/transition work.

## Custom change-filament lifecycle

Start clean0e039a63. Current unchanged full dual replay88488 exit100,
2.458s, /tmp/ares-custom-change-current-red.log, case-fqfXrG/case-JLy3IO.
First differences unchanged (baselineM73 placement/modifierremainingminutes).
Ranked falsifiable hypotheses: template unconsumed (no injected marks),
wrong dynamic context (marks but wrong IDs/counter), or state/command
suppression (correct marks but duplicate T/incorrect Z at new start).

Public native-first child tracer tool_templates/change.rs injects a minimal
custom comment with previous/next IDs, runtime count, layer and relative-E;
one whitespace-prefixed T[next_extruder] and G1 Z{toolchange_z+2}. Compare
all17 native blocks (initial+16printing), first dynamic records, first new
start Z2.2, exactly1 T per block and fan-resume flag. Unchanged complete
comparator still runs. Red build45726 running; obtain valid native-first RED
before production. Same previously approved public 3MF→slice byte seam.

Source boundary set_extruder7749/7858–7980/7991–7997,
custom_gcode_changes_tool241–264, retract7652–7704/eager_lift652–681.
Port coupled context/render/select/startup consumer, not a counter-only
checkpoint. Runtime count owned by generator state; initial included.
Reuse live template owner and shared writer tool selection. Required eager
retract executes before optional render even for empty/manual-skip templates.
Custom supplied command suppresses emitted writer string but not writer
installation/reset. Last custom Z propagates to next filament-start.
SEMM purge/tower, ooze temperatures and remaining output timing stay open;
do not claim original full dual closure until both complete outputs pass.

Initial45726 build and76242 replay both exit101 missing child module;
fixed path attribute before any production change. Build36143 exit0.
Replay14735 exit1001.854s is native expectation failure, NOT behavioral
RED: machine-start selectsT0, so source previous_extruder is0 rather than-1
(GCode.cpp3141/7817, GCodeProcessor.cpp::get_gcode_last_filament2932).
Correct native literal accordingly; build50925 exit0. Replay80611 exit100
1.866s also NOT valid RED: final body cooling consumes the internal
FORCE_RESUME marker, whereas initial marker survives. Assert initial marker,
compare remaining native flags rather than requiring internal markers in
all final body blocks. Rebuild in progress; production remains untouched.
These oracle calibrations affect only the new diagnostic, never original
full comparator/fixture. Implement machine-start last-filament tracking as
part of the cited context lifecycle; don't hardcodeinitial0.

Corrected diagnostic build53021 exit0. Replay93624 terminated/reaped exit100,
1.836s, /tmp/ares-custom-change-red4.log and case-mVXdy7: all17 native block,
ID/count/layer/start-Z and single-T checks pass; Ares blocks are empty.
This is the valid behavior RED. Then implement shared initial/body lifecycle,
runtime state count/start-filament, dynamic common fields, live render/write-
back and custom-command suppression. Required injected eager SpiralLift
reuses motion's retract/lift; split spiral output function into a child mod
to avoid growing lift.rs beyond400. Existing eager callers retain previous
hop-type dispatch; the custom call forces source SpiralLift. Build64423
running after fmt. No GREEN/complete dual claim yet.

Implementation patch retries (duplicate target/order mismatch) failed before
applying the combined eager-call patch; spiral mechanical extraction was
already applied in its preceding successful patch. Corrected patch completed
before build64423 began. No compilation failure counts as behavioral RED.
Coverage remains only public common identity/layer/command/Z lifecycle:
purge-volume dynamic fields/SEMM state, all other dynamic values, eager
trajectory controls and manual-first-skip need their own source/output
evidence before broad transition claims. Full original acceptance stays open.

Build64423 exited0 (1m17s). Probe62814 exited100 (1.885s): all17
custom contexts/counts/single-T/new-start-Z match, but Ares retains every
body FORCE_RESUME_FAN marker while native consumes them. This is still RED,
not a complete lifecycle checkpoint. Static review required ordinary (not
toolchange) injected wipe/retract, missing-map hardware0, clearing eager
zero-target lift/pending state, and source signed-u32 custom T scanning;
fixed all four before build4912, which exited0 (1m17s). No running build
was edited. Reviewer readback confirms those fixes, not runtime acceptance.

Fan consumer boundary: CoolingBuffer.cpp543,890–898,1000–1005. Resolve
force events in the printing-layer cooling flush in the same ordered pass
as role requests; retain baseline vs current effective speed and auxiliary
resend. Existing priority-normalized role markers are a compatibility shell
for active requests. Initial custom event remains outside CoolingBuffer and
must survive. Do not strip globally or weaken the native-first flag checks.

Independent source review gives a concrete remaining wider-domain gap: an
Overhang request from toolA can remain active after switching to toolB whose
control is disabled; source END only clears when control is enabled, unlike
the existing normalized BASE shell. SupportInterface/Ironing identities also
remain absent. Explicitly defer these source consumers, not their eventual
acceptance: the finite same-fan-configuration tracer cannot prove them.

Fan build92120 exit0. Official public replay38625 exit0,1.880s,
/tmp/ares-custom-change-fan.log: all17 native/actual contexts, single-T,
new-start-Z and retained-startup/consumed-body event flags match. Neither the
original comparator nor its input/assertions was relaxed. Begin retained
official/seeded controls and quality/full-workspace gates; original dual
complete output and global printer/options closure remain open.

Official broad19185 exited100 after default fail-fast:7PASS/1FAIL,
48unrun; retained full dual baseline byte1726/line124 and modifier byte718/
line25 remain RED. This is not a completed broad receipt. Clippy74470
exit0 (1m19s) had254lib/270test, one more than baseline due the new
DoubleEndedIterator::last scanner; WASM60406 exit0 (1m36s, compile only).
After both terminal, replace last with next_back, preserving last-valid
selection without unnecessary iteration. Re-run final gates, and use
--no-fail-fast for both broad references so known RED does not hide controls.

Final frozen-source receipts: fmt/diff check53374 exit0; clippy44978 exit0,
1m32s, /tmp/ares-custom-change-final-clippy.log, unchanged253lib/269test
and CLI31 warnings. WASM9055 exit0,17.57s,
/tmp/ares-custom-change-final-wasm.log (core+adapter compile only).
Prebuild70605 exit0: workspace4m14s (includes lock wait), then exact CLI
1m16s; no CPU0 compilation during the final replays.

Official99476 exited100 after all56 tests (132.102s), seeded93400 exited100
after all56 (179.427s), /tmp/ares-custom-change-final-official.log and
/tmp/ares-custom-change-final-seeded.log. Each:55PASS/1FAIL, with47 strict
complete-output controls and8 limited diagnostics passing. The new custom
17-block tracer passes1.874s/2.427s. Only inherited_filament's original
complete dual comparator fails: baseline byte1726/line124, modifier
byte718/line25, unchanged first-difference positions. All outputs/artifacts
retained; do not call these whole-suite reference GREEN results.

Final workspace26803 ran alone after both reference processes terminated:
exit0, /tmp/ares-custom-change-final-workspace.log,7563PASS (36slow),9skip,
256.857s. Spiral-vase ramp regression is PASS. All changed/new functional
Rust modules are below400LOC. No original full-output assertion, fixture or
warning suppression changed. Request final independent six-axis approval
for this coupled common-template LOCAL code+test+docs checkpoint only.
Full dual output, differing-tool fan/request identities, SEMM/tower/ooze,
all-printer and complete option-domain acceptance remain open; no push/main
or global milestone completion is authorized by these finite receipts.

Independent final six-axis review read the actual terminal receipts and
approved this finite coupled custom-lifecycle code+test+docs LOCAL checkpoint
with no new must-fix. Approval covers common context/render/write-back,
writer selection, eager and same-fan-configuration cooling consumer only.
Differing-tool request identity, SEMM/tower/ooze, complete dual and global
printer/option domains remain open; no push/main/global completion approval.

## Ooze temperature consumer

Previous goal turn made verified progress: local85b95cb1 plus7563PASS and
both55/56 reference receipts. Authoritative worktree starts clean85b95cb1.
Exact CLI prebuild exit0; fresh original dual30968 exit100,2.589s,
/tmp/ares-dual-after-custom-red.log, baselinecase-6X8eVD and modifiercase-
5IzwM7. Same baseline1726/line124 and modifier718/line25 first differences.
The baseline's one model/no modifier/0%sparse already reproduces; retain
both full fixtures rather than weaken or replace the failing comparator.

Shown ranked predictions: time-only would leave no non-M73 differences;
missing transition side effects would cluster at T/M104/M109; invalid writer
position would first change post-transition travel/lift/retract. Raw diff
rejects time-only: waiting nozzle commands missing at startup/every transition,
plus feedrate, fan and duplicate Z travel differences. Source attribution is
OozePrevention267–309/2280 and set_extruder7788/8044; the source's removed
M104 comments instead come from real ExportLines backtrace1320–1384.

Port Ooze pre/post into tool_transition/temperature with shared writer
temperature formatter. Use actual leading layer id before tool iteration,
then existing per-object layer-id updates, not on_first_layer or global count.
Pre only with old selection and enabled independent-nozzle ooze; idle wins,
otherwise delta gate; post only delta!=0. Keep raw cooldown and source order.
Shared machine formatter replaces only nozzle non-wait shell, not bed/chamber.
Preflight independent review approves this boundary, requires all above
distinctions and unsigned/flavor wait contracts, and does not approve complete
dual or a checkpoint. Existing original full test is the valid RED; coupled
postprocessing/fan/feedrate/Z acceptance remains open. Do not fake a GREEN
window or emit removed-command comments. No production modification yet.

Then implemented shared writer nozzle formatter and actual Ooze pre/post
consumer, with leading physical layer id installed before the tool loop.
No test/fixture/assertion changes. Build13143 exited0 (1m15s).
Original complete11557 exited100 (2.600s), /tmp/ares-dual-ooze-replay.log,
baselineJF8FeZ/modifierCbqR53: same first-difference positions, still RED.
Read-only native/actual extraction of existing waiting-temperature commands
has identical sequence/value in both cases (17baseline,94modifier); this is
diagnostic evidence, not a new window test or whole-output acceptance.
Ares now emits actual M104...;cooldown, whose suppression/preheat remains
the ExportLines coupling. Feedrate, repeated Z and fan differences remain.

Frozen-source fmt/diff checks exit0; clippy96812 exit0 (1m19s), unchanged
253lib/269test and CLI31; WASM30999 exit0 (1m35s, core+adapter compile only).
Official broad3379 terminated/reaped exit100 (132.475s),
/tmp/ares-dual-ooze-official-controls.log:56tests,55PASS/1FAIL, only original
full dual;47strict complete-output controls and8limited diagnostics pass,
including raft/support/skirt/retraction controls for the active-layer-id
change. New/shared modules37/60LOC. Independent static six-axis readback
found no new must-fix, but requires runtime gates and the real ExportLines
coupling before checkpoint approval. Seeded and final full workspace have
not been re-run on this snapshot; previous85b95cb1 receipts do not prove it.
Keep this unfinished coupled source+docs slice uncommitted, without push or
global completion; next action is real cooldown/preheat owner, not a comment
filter or new substitute acceptance test.

Independent result/source six-axis复验 confirmed actual55/1, unchanged
warning corpus and no immediate new must-fix, while rejecting checkpoint /
push/main/global closure. Keep the required follow-up boundary explicit:
`ExportLines` (`GCodeProcessor.cpp:682–960`), `insert_lines` (`833`),
postprocess tool handler (`1300–1390`) and configuration (`2038–2043`).
Typed process preheat_time/preheat_steps already exist. Replace the current
straight-line processor export shell with the source buffered-time/backtrace
consumer against real motion elapsed caches; carry original tool/layer and
logical-to-physical identities. Do not infer removal from command spelling
alone or add a whole-file filter. Reproduce native preheat insertion and
cooldown replacement, retaining the original complete dual assertion.

### ExportLines implementation handoff (not implemented in Ooze checkpoint)

Upstream boundary is `GCodeProcessor.cpp:682–960` (`ExportLines`,
`LineData`, `Backtrace`, `update`, `append_line`, `insert_lines`, `write`,
`flush`), the tool callback at1300–1390 and `apply_config`2038–2043;
`PrintConfig.cpp:11607–11622` owns XL identification. Rust destination is
`gcode_emit/processor/export_lines/`, replacing the processor's straight-line
export shell, not adding a second analysis pass or a whole-file filter.

Included next: borrowed effective preheat/temperature/physical-map inputs;
normal-mode motion-cache timestamps; source deque buffering and strict
time-threshold flushing; exact previous-tool and case-insensitive start
barriers; in-window cooldown replacement; stepped temperature insertion;
source layer-marker count for initial-versus-normal temperature. Activation
is ooze enabled, positive preheat time, and XL or independent multi-filament
configuration. Clamp preheat_steps to at least1 at the configuration boundary.

`ExportLines::update` snapshots the post-increment motion-cache id, whereas
the existing M73 move callback probes a pre-increment id. Expose distinct
lookup results from the same Estimate cache; do not change progress lookups
or recompute geometry. Replacements occur while walking the window even
when a barrier ultimately prevents insertion. Insert after the found older
entry, retain its timestamp, and suppress duplicate insertion timestamps.
Buffer retention is1.1f times a dynamic max_backtrace_time: initially120f,
updated to max(current, configured preheat time) only in the enabled T
branch. It is not the configured preheat time from the first line. Process
T before appending that T to the deque.

Deferred adjacent behavior: stealth-mode cache and its XL Q parameter,
viewer line-id map/file offsets and native filesystem export. Browser core
still returns bytes. Any deferred behavior required by the unchanged
selected dual-tool fixture must be implemented before its complete closure.
Final acceptance remains the original two complete-output comparisons;
temperature-only observations cannot replace them.

Physical preheat/cooldown identity specifically uses the zero-based
`physical_extruder_map` (`GCodeProcessor.cpp:2050,1366,1381`), with absent
entries falling back to the logical tool id. Do not substitute the separate
one-based `filament_map` used for nozzle offsets. XL is identified by both
source printer_notes markers, not a guessed printer-model string. The reader
command boundary is space/tab/semicolon/CR/LF/NUL (`GCodeReader.hpp:180–195`),
and custom commands remain case-sensitive except for the start barriers.

Retain cache entries' source `(id, remaining_internal_g1_lines, elapsed_time)`
identity when exposing ExportLines timestamps. For arcs, update walks the
eligible remaining-segment chain, increments its lookup counter for each
entry, and snapshots only when the final cache id equals that counter.
Geometry segment count alone is not that walk when seam/drop holes exist.
Preserve the already-verified M73 behavior while porting this separate
consumer; cover any required cache change with original arc controls.

### Ooze local checkpoint final receipts

No production or test changes during this final gate run. Fresh original
complete dual91615 terminated/reaped exit100 in2.611s,
`/tmp/ares-export-backtrace-current-red.log`; full assertion remains RED.
Seeded48331 terminated/reaped exit100,179.511s,
`/tmp/ares-ooze-checkpoint-seeded.log`:56 run,55PASS/1FAIL,281 skipped;
only original complete dual fails. Together with the prior official55/1
on this same frozen source, these retain47strict complete-output controls
and8limited diagnostics, not a claim of all-printer/option coverage.

Workspace67301 terminated/reaped exit0 after2m46 compile and255.025s run,
`/tmp/ares-ooze-checkpoint-workspace.log`:7563PASS,35slow,9skipped.
Spiral vase ramps and current-layer-spacing behavioral regressions both PASS.
Current fmt/diff exit0; `/tmp/ares-ooze-checkpoint-clippy.log` exit0,
253lib/269test/CLI31 unchanged; `/tmp/ares-ooze-checkpoint-wasm.log` exit0,
core+adapter compile only. No comparator/fixture/coverage removal.

Finite checkpoint scope is OozePrevention pre/post, leading actual layer id
and shared writer nozzle formatter, plus its spec/ARD/plan. Complete dual
output remains RED; real ExportLines consumer, fan/F/Z and time differences
remain unfinished. No push/main/full-parity approval is inferred. Exact CLI
prebuild must run again before the next CPU0 replay after this workspace build.

Checkpoint policy correction: the user's explicit local logical-commit
authorization permits the independently verified Ooze pre/post source slice
while complete dual parity remains RED. The independent reviewer agrees
that ExportLines/fan/F/Z closure is not a prerequisite for that finite local
checkpoint, conditional on current seeded and workspace gates showing no
new unexplained regression. This is not push/main/full-parity approval.

Final independent six-axis复验 read actual diff and receipts and approves
these9files as a finite local code+docs checkpoint, with no new must-fix.
The approval explicitly excludes push/main/complete parity/global completion;
commit body must retain full dual RED and ExportLines/fan/F/Z open scope.

### ExportLines RED / implementation preflight

On clean1f42b99e, exact CLI prebuild exit0; original public complete dual
90121 terminated/reaped exit100 in2.613s (`/tmp/ares-export-lines-red.log`).
The baseline one-model/no-modifier case still reproduces; preserve both
original fixtures. Ranked predictions are missing consumer (raw cooldown
survives), activation/mapping mismatch (wrong target), or timestamp mismatch
(wrong insertion/removal window). Missing production consumer is confirmed.

Independent preflight accepts source boundary with three requirements:
group existing envelope+borrowed effective settings as ProcessorConfig,
keeping the existing five-argument process boundary rather than adding6;
use ByTime only when enabled, otherwise preserve source BySize flushing;
retain actual arc remaining-chain metadata for the new timestamp consumer.
Private processor tests only adapt configuration construction, with default
settings representing source-disabled backtrace. Production always borrows
real effective settings. No geometry re-analysis or M73 lookup change.

First CLI build73689 exit0,1m15; full19785 exit100,2.593s,
`/tmp/ares-export-lines-first-replay.log`:baselineN90k6e/modifieri6AqAb,
unchanged first differences1726/718. Read-only extraction of real removed
M104/preheat/cooldown lines yields identical sequences(21/94), not a new
acceptance test or complete GREEN. Fan/F/Z/time differences remain.

Independent source review found the cooldown char-axis overload uses strtod,
not the adjacent fast_float overload. Fixed signed +, repeated invalid words,
bare axes, prefix-consumption versus no-conversion and C-locale hex floats.
Hex helper preserves double rounding with normal/subnormal guard/sticky and
ties-to-even before f32 conversion. It is private to cooldown T words, not
a new general reader framework. External nonfinite/unrepresentable values
cannot match physical IDs; no native-UB or NaN payload bit parity is claimed.
Review withdrew an ordinary0-temperature concern after reading configuration
normalization2076–2082; retain lazy source-equivalent normalization with citation.

Clippy first55686 terminated exit101 due closure E0631; corrected closure.
Next91221 exit0 but254/270 included new nonminimal_bool; corrected guard.
Quality97300 and84674 exit0 (1m19/1m20),253/269 unchanged. Required nesting
flattening follows source window/inserter callbacks, withoutallow attributes.
All subsequent gates must run on the final matching-range guard snapshot;
these intermediate quality receipts alone do not prove final approval.

### ExportLines frozen gates and reference ordering evidence

Final matching-range snapshot CLI80889 exit0 (1m17),
`/tmp/ares-export-lines-frozen-build.log`; clippy55549 exit0 (1m19),
253lib/269test/CLI31 unchanged, `/tmp/ares-export-lines-frozen-clippy.log`.
fmt/diff exit0. WASM97064 exit0 (17.35s),
`/tmp/ares-export-lines-frozen-wasm.log`, core+adapter compile only.
No production edits occurred after these gates began.

Official19515 terminated/reaped exit100 (131.164s),
`/tmp/ares-export-lines-official-controls.log`:56run,54PASS/2FAIL,
281skipped. Preserve both failures: original complete dual RED and the
later-higher-density compatible-regions baseline failure. The latter's
first normalized difference is byte12097/line524: native travels to
X113.388 Y113.135 before X113.244 Y113.297; Ares uses the reverse block
order. Native exchanges two Internal solid infill blocks and moves M73.

Current failure case-u3KBIj/ares.gcode and prior passing checkpoint
case-btWlnD/ares.gcode have identical raw SHA256
9947b3e77058ea7889262e570560b60807bd662ae698965e3a968f0ad0b64c4d.
Both roots remain under /tmp/ares-export-lines-official-controls and
/tmp/ares-dual-ooze-official-controls respectively. Their saved input ZIPs
have different container hashes/timestamps, but all nine decompressed
members compare identically (including escaped [Content_Types].xml).
This establishes unchanged input content and Ares output, with observed
reference ordering variation; it does not identify its internal cause.
Focused original complete-comparator reruns16405/85349 terminate exit0,
2.306s/2.284s, logs/artifacts /tmp/ares-export-lines-density-repeat-one
and /tmp/ares-export-lines-density-repeat-two. These passes do not replace
the retained broad54/2 receipt or weaken any assertion.

Seeded67501 terminated/reaped exit100 (179.815s),
`/tmp/ares-export-lines-seeded-controls.log`:56run,55PASS/1FAIL,
281skipped. Only original complete dual fails; later-density full comparator
passes. These finite controls retain47strict complete-output comparisons
and8limited diagnostics, not all-printer/options coverage.
Workspace23643 terminated/reaped exit0:2m48 compile,257.884s run,
`/tmp/ares-export-lines-workspace.log`:7563PASS,35slow,9skipped.
Spiral vase ramps and current-layer-spacing behavioral controls pass.
Final fmt/diff checks remain required after receipt-only documentation edits.
Request independent finite local checkpoint approval; complete dual RED,
fan/F/Z/time differences, all-printer/options coverage and push/main closure
remain outside this approval. New reproducible-generation documents are a
separate user-requested slice and are excluded from this checkpoint.

Independent final six-axis readback approves the16file staged closure as a
finite local checkpoint, with no source must-fix. Final fmt69151 and cached
diff check terminate exit0. Approval excludes reproducibility documents,
push/main, complete dual parity and global completion; preserve official54/2,
seeded55/1 and unchanged-Ares/reference-variation evidence in commit body.
