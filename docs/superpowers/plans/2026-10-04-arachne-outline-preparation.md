# Arachne outline preparation plan

Source/destination/included/deferred boundary: ARD-0100 and matching spec.
Start from clean committed/pushed68ef6e91 on codex/printer-option-matrix;
do not modify the user's main worktree.

1. Replay unchanged public no-detection actual-overhang RED; capture exact
   native/Rust outline stages at its corner before modifying production.
2. Compare upstream preparation functions; test one ranked prediction at a
   time. Identify the earliest divergent stage, not merely a plausible gap.
3. Port that bounded source contract, re-run the full unchanged assertion
   and finite previous GREEN controls. Extend public legal-domain coverage
   only as needed to make the changed behavior sensitive.
4. WorkspaceNextest, Clippy/fmt/WASM; independent six-axis re-review. Commit
   code/tests/docs and push each verified finite slice. Preserve every RED
   receipt and defer adjacent raw work explicitly if still unresolved.

## Baseline feedback loop

`cargo nextest run -p ares-cli --test orca_parity
-E 'test(arachne_overhang_without_detection)' --no-fail-fast`
with explicit seeded reference, seed0/CPU0 and external artifact root.
34717 terminal100:0/1PASS,1.931s,
/tmp/ares-outline-baseline-seeded.log, strict byte9371/line404,
expected191340/actual191204 normalized bytes. The preceding filter typo
selected zero tests and exited4; it is not a successful test receipt.
This seam is already user-approved and retained unchanged from the checkpoint.

The STL is small and reaches the missing corner without overhang clipping;
this is the existing reduced differential signal, not a claim that every
remaining3MF configuration field is proven load-bearing. Prior stage evidence
places the missing branch before stitch/simplify/materialize, but no matching
native prepared-outline capture yet establishes the precise first divergence.

Ranked falsifiable predictions:
1. Missing near-self-intersection repair: native prepared vertices differ
   from Rust around close nonincident segments before raw generation.
2. Combined cleaner differs: after matching preceding stages, separate native
   degenerate/collinear pass preserves or shifts a corner absent in Rust.
3. Raw generation gap: final prepared outlines match exactly but raw variable
   width junctions differ. Only then investigate SkeletalTrapezoidation.

Use available GDB17.2 and existing separately seeded debug symbols first;
no official modification and no new seeded build. Source citations do not
alone establish causality. Diagnostic scripts live under /tmp; no production
change or implementation-completion claim at this planning checkpoint.

## Matching native stage evidence

Existing seeded ELF build-ID d1d964043a0097f4c33db3fa1bbd5991477aeef9
matches its separate292MB DWARF; explicit symbol-file required (wrapper ELF
has different build-ID). GDB runs21423/64993/33711 all terminal0 on the exact
baseline exported3MF, seed0/CPU0. Full ordered contours/holes and parameters
are in /tmp/ares-outline-native-{stages,final,owner}.log. The first inspection
8981 only tested variable access; truncated prints are not geometric evidence.
Final debugger-produced G-code differs from saved reference only on its
automatic generation-date line (diff-U0 verified); no comparator change.

Owner capture identifies calls58/59 as object layer_id50/51, Z10.2/10.4,
object-local coordinates, no holes. Their complete source surface contours
match Rust's owner probe81958 (terminal1000/1, retained RED),
/tmp/ares-outline-owner-probe-seeded.log. Thus mesh slicing is not implicated
by the observed one-unit mismatch. Rust surface_simplify_resolution=12000;
native value optimized out. Native last_p corner X14664483 vs Rust14664482;
next layer13464484 vs13464483. The divergence already exists on entry to
WallToolPaths::generate, before its offset/simplify/cleaner stages. Native
first/second near-self, degenerate, collinear, small-area and final-union
stages leave these two simplified contours unchanged. These cleaner gaps
are real contracts but are not yet causal for this first observed divergence.

Native/Rust outer and inner spacing407079, inset3, wall_0_inset-21460,
min_bead340000, min_feature100000, transition_length400000, filter100000,
distribution1, maximum_resolution500000 and deviation25000 match. Native
retains layer_height0.1999999999999993; Rust truncates it to199999 lattice
units before raw construction. This is a separate source-contract gap, not
yet proven to cause the branch loss; do not mix its repair into the first
controlled experiment.

Next single-variable prediction: source2140–2142 simplifies to Polygons then
offset_ex(Polygons), whereas Rust reconstructs ExPolygons via an extra union
and offsets each expolygon separately. If this prerequisite contract causes
the one-unit offset mismatch, using the existing simplify_expolygon_polygons
and offset_paths_tree APIs will match last_p, with no outline/raw algorithm
change. Re-run the unchanged public complete assertion; preserve any RED
and compare stage evidence before selecting the following production change.

Polygons-offset84710 terminal1000/1,1.985s,
/tmp/ares-outline-polygons-offset-seeded.log. Target layers50/51 now have
complete ordered last_p matching native owner capture, including corner
14664483/4957080 and13464484/4957080. Public complete output is STILL RED
at9371/404 with unchanged lengths: this confirms the input-contract mismatch,
not that it caused the missing branch. The extra ExPolygon union reunited
touching simplified polygons before the source-selected offset operation.

Next single-variable correction: preserve native floating millimeter layer
height through RawWallToolPathConfig and top-surface regeneration, converting
directly to float for thresholds (WallToolPaths.cpp:80–95,518–519). Remove
the private lattice truncation rather than round or add a fallback. Existing
fixtures adapt only to the config unit/type change; no public fixture or
output assertion change. Capture final prepared outlines during this run to
distinguish unchanged input geometry from raw-threshold effects.
First height run40513 exits101 at compile: missed shared concentric config
consumer, no behavioral test executed. Adapt only its existing integer height
back to millimeters, preserving prior concentric semantics; upstream owner
FillConcentric.cpp:100. Earlier concentric input truncation remains deferred.

Height-fixed77943 terminal1000/1,1.969s,
/tmp/ares-outline-floating-height-fixed-seeded.log. After both prerequisite
repairs, full ordered prepared outlines now match native final for target
layers50/51 exactly; stored height also matches native0.1999999999999993.
Public complete output remains RED9371/404,191340/191204: neither prerequisite
alone resolves missing branches. No blanket raw-input equality claim.

Further input readback reveals degrees/radians mismatch: native
WallToolPaths.cpp:490 converts wall_transition_angle10 degrees via
Geometry.hpp:301–305 (`PI * angle /180`) before beading/raw constructors.
Rust WallToolPaths raw config expects radians (existing tests and concentric
already supply radians), but wall/top-surface callers supply option10 directly.
Third single-variable correction converts at those three caller sites; do not
change raw config semantics or concentric. Remove every temporary probe before
this public rerun. This is a source-owned input rewrite, not speculative
outline/Voronoi algorithm replacement.

Independent source review finds the final angle-consumer precision boundary:
WallToolPaths.cpp:547 passes beading_strat->getTransitioningAngle(), after
factory float narrowing and double storage, not the initial double radians.
Use existing strategy.transitioning_angle() in TrapezoidationConfig. The
pre-review3263/clippy48012/WASM79045 snapshots are not final acceptance of
this correction. Re-run after terminal handles; do not cancel/restart live
jobs or relabel their earlier binary as post-review source.

Pre-precision3263 terminal0:3/3 completePASS,5.511s,
/tmp/ares-outline-radian-angle-seeded.log; no-detection actual-overhang,
nonzero thresholds0.1 and50%, reversal-disabled hole are now GREEN without
changing assertions/fixtures/comparator. Earlier prerequisite runs stayedRED;
The final degree-to-radian correction resolves the still-RED no-detection
tracer after the first two controlled changes. The other two tracers pass
the combined input repairs; their individual causality is not isolated.
This is not yet acceptance of the final reviewed precision boundary.
Pre-reviewWASM79045 terminal0,17.35s,compile-only. Clippy48012 remains live
and will not replace a fresh post-review gate.

Added public legal-angle range control at the same approved seam:1,59 and
`1 + LCG(0x5eed)*58/2^32`, explicitly verified in saved3MF and complete output.
It also asserts native boundary extrusion moves differ, so domain coverage
cannot be satisfied by configuration-footer differences. Existing helpers
retain original presets when no angle override is supplied. No code-source
pinning or private golden movement assertion is introduced.

Frozen final source now runs seeded64831, officialCPU0 reference, workspace,
Clippy and WASM in fresh logs /tmp/ares-outline-final-*.log. Their terminal
results are pending, not implied by earlier focusedGREEN. No probe remains.

Seeded64831 terminates100:15run14PASS1FAIL,26.159s. All retained default-angle
RED tracers and prior finite controls now PASS; new range min1/max59 complete
comparisons also pass. Interior fails saved-config application readback:
requested38.93081125989556, official exporter serializes38.9308. No interior
complete output was compared. Specify the seeded draw at four decimal places
(38.9308) before export, then require unchanged strict effective-value equality;
do not relax application verifier or full comparator. Review also strengthens
native geometry sensitivity: only positive extrusion with XY is compared,
tracking M82/M83 and G92, ignoring E/F-only or retraction differences. Fresh
final tests required after both test corrections, not a claimed domainPASS.

Official80200 terminates100:15run14PASS1FAIL,29.313s. The new range helper
parsed M84's bare axis E as a number, causing ParseFloatError(Empty), not a
complete-output mismatch. Dispatch G1/G92 before parsing E; preserve strict
numeric parsing for those commands. Track relative extrusion cumulatively
for a later M82 switch, while G92 resets the absolute accumulator.

Reviewed production receipts: seeded63393 terminal0,15/15 functions,94.464s;
official CPU0 13813 terminal0,15/15 functions,78.632s. Each run actually
compares26 paired complete outputs, including all three angle values, retained
hole/overhang tracers, stock Prusa MINIIS and Slim/Strong/Hybrid/modifier controls.
Logs: /tmp/ares-outline-reviewed-{seeded,official}.log. Manifest binary SHA
d2751ae561ffd8606048de921a3438c80c3ca6497ba39099ec6a11d3a36a09c3;
working-tree runs are not mislabeled as immutable HEAD68ef6e91.
After the final relative-E helper correction, seeded13386 terminal0: range
test1/1,25.632s, all three complete outputs PASS and native positive-extrusion
XY boundary sensitivity PASS; /tmp/ares-outline-final-witness-seeded.log.
Saved-original generator-only replay4513 terminal0:1/1,1.616s,
/tmp/ares-outline-reviewed-saved-original.log. This is not a fresh official
export run. Final production WASM21677 terminal0,17.55s, core and adapter
wasm32-unknown-unknown compile only, not browser-runtime verification.

Final helper-snapshot official74243 terminal0: range1/1,19.584s,
/tmp/ares-outline-final-witness-official.log; all three strict pairs and
positive-extrusion XY sensitivity PASS. Frozen Clippy49554 terminal0,2m07s,
/tmp/ares-outline-frozen-clippy.log:253 core-lib,269 core-test,31 parity-test
warnings, unchanged baseline; existing KSR10 unchanged. Fmt12155 terminal0.
Earlier full runs31030/55918 also terminal0:7611/7611,9 skipped,
571.833s/541.085s respectively. These pre-final-helper snapshots do not replace
frozen11315 acceptance. Retain their logs without relabeling run ownership.

Frozen workspace11315 terminal0:7611/7611 PASS,120 slow,9 skipped,366.853s,
/tmp/ares-outline-frozen-workspace.log. Final fmt/check87800 terminal0;
cached and unstaged diff whitespace checks clean. All seven changed Rust files
are below400LOC (maximum302); staged diff contains no DEBUG/eprintln probes.
Independent six-axis static review reports no new source/test must-fix; final
immutable patch approval follows these frozen receipts before commit/push.

This finite input prerequisite slice is GREEN at its retained public gates.
Outline cleaner/raw algorithms, raft/Adaptive PA, earlier concentric height
quantization and all-printer/all-Option matrix completion remain deferred.
Do not extend the checkpoint or count these adjacent domains as completed.
