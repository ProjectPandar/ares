# Organic worker-domain validation plan

Boundary: ARD-0102 and matching spec. Previous goal turn made progress via
0bd83fd9 reporting checkpoint. Current branch starts clean at that commit.

1. Replay the unchanged saved public input; verify baseline/nonregression.
2. Isolate configuration, initial tips and branch growth; check execution
   inputs before changing production. Preserve all RED observations.
3. Add automatic raw Grid/Snug style coverage, verify native activation,
   effective values and unchanged strict complete output in matched domains.
4. Run full gates and independent six-axis review; commit/push finite tests
   and source-boundary docs. Continue actual matrix residuals afterward.

## Evidence-driven diagnosis

Saved input SHA e45bff975bfcb3eaf157ad5427a8b7d15d30a799d6e5cb22fa7491b2e07c23a1;
reference from /tmp/ares-support-report-official/support-report-XAdZOH/case-m0v7Hz.
Replay root /tmp/ares-tree-auto-replay.aHO5QM retains original input/reference.
6418 terminal100:1.241s, unchanged complete-output RED byte897/line33,
/tmp/ares-tree-auto-replay-0bd83fd9.log. The source signal is the original
first-layer support/skirt movement discrepancy, not only M73.

Old immutable0c60ab28 baseline ELF SHA
f39b9e368b511014ceca4396829d2b79366ed4ab1a61d674a94af6beebd60e13;
33614 terminal101,1.21s, same saved pair and same first difference. Raw Ares
output cmp equals current. No recent code regression is established.
Existing default Tree control70551 terminal100:5.855s, same symptom, retained
/tmp/ares-tree-auto-control-0bd83fd9; no fixture/assertion change.

Initial predictions: differing config conversion; differing initial tip set;
or branch-growth divergence after matching tips. Source review exposes an
earlier execution mismatch: TBB bucket sizes use worker count. Existing
tree-support plan already requires both processes in the same domain.
Changing only Ares affinity to CPU0 is the single-variable test:85732
terminal0,1/1 completePASS,3.517s, /tmp/ares-tree-auto-matched-cpu0.log.
Thus no production repair is justified for this signal. This is a retained
small saved-input tracer, not exhaustive minimality of every preset field.

The previous ARD0101 plan's native/native reproducibility and actual geometry
differences are valid observations, but interpreting them as an Ares geometry
bug was premature: Ares/native execution inputs differed. Preserve the records
and add this correction, rather than deleting RED or widening normalization.
Default/native multiworker equivalence remains open.

Original stock22812 and Option49613 are still live loaded earlier binaries;
these runs restrict only Orca, so label them mixed-worker exploratory evidence,
not matched-worker acceptance or global parity. Do not cancel/restart them
because a domain mismatch was observed; preserve their artifacts and terminal
results separately from new matched-domain runs.

Matched-domain broad runs started on the frozen test-only WIP: stock51376
root/log /tmp/ares-stock-matrix-organic-matched-wip-seeded; Option46197
root/log /tmp/ares-option-matrix-organic-matched-wip-seeded. taskset CPU0
wraps Nextest, so its Ares child and native reference inherit the same domain.
These are live, not completed coverage. Do not stamp their WIP binary as HEAD.

Separate coverage limitation: the existing support_style sweep only enables
support, leaving the normal baseline for Tree-specific raw styles. Upstream
then normalizes these to Grid, so raw-value complete comparisons alone do not
prove active Tree-style coverage. Keep that report scope explicit; correct
per-value activation in the next finite source-cited test slice, not by
extending the current Grid/Snug checkpoint before commit.

Official matched54310 terminal0:4/4 functions,7 paired complete outputs PASS,
161.240s,1 slow, /tmp/ares-organic-domain-official.log. Its persisted
support_type report is PASS4/4 (unlike mixed-worker RED), with the same report
assertions. Clippy41898 terminal0,3m09s including build-lock wait: unchanged
253 core-lib/269 core-test/30 CLI parity warnings; existing KSR10 unchanged.
Fmt/diff/WASM53746 terminal0; WASM compile-only0.09s (core/adapter unchanged).
Seeded24183 terminal0:4/4 functions,7 paired complete outputs PASS,
197.362s,1 slow, /tmp/ares-organic-domain-seeded.log.
Four-domain29236 terminal/reaped0:1/1 function PASS,284.453s,1 slow,
/tmp/ares-organic-domains-matched-seeded.log. Persisted summary records
4/4 bounded domains,16 generated/16 strictly compared/0 rejected: bool2,
support_type4,support_style7,generic branch_angle3. These are raw-value
comparisons with the activation limitations below, not exhaustive active
Tree geometry coverage.
Workspace24806 terminal/reaped0:7613/7613 PASS,37 slow,9 skipped,267.328s,
/tmp/ares-organic-domain-workspace.log. Native-reference optins are off in
this routine workspace run; its green is not additional live parity evidence.
The four broad scan handles22812/49613/51376/46197 remain live after these
finite gates; no all-printer/all-option completion claim is made.

The same activation audit applies to generic tree_support_branch_angle:
TreeSupport.cpp consumes it on non-Organic paths, while TreeSupportCommon.hpp
uses tree_support_branch_angle_organic. The current generic range baseline
normalizes Grid to Organic, so even a min/max/seeded full-output PASS is only
raw-value evidence, not active generic-angle geometry coverage. Include this
owner/style co-injection correction in the next finite activation slice.
