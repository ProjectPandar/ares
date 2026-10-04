# Tree option activation plan

Boundary: ARD-0103 and matching spec. Previous checkpoint e59ff8a1 is clean
and pushed; previous goal turn is progress. User limits this turn to the
activation slice; latest instruction additionally requires main integration
and final validation before pausing.

1. RED public generic-angle min/max/seeded native movement sensitivity.
2. Select source-owned Slim path; verify full outputs without relaxation.
3. RED public style domain under Normal; select Tree for Tree-only values.
4. Audit listed Tree controls against their upstream consumers; correct
   Organic vs non-Organic and manual-brim dependencies in the harness.
5. Run official and seeded matched-domain complete pairs, full workspace
   Nextest, clippy, fmt, WASM; independent six-axis review; commit/push.
6. Synchronize with latest origin/main in a clean worktree, repeat final
   full Nextest/clippy/fmt/WASM, fast-forward push main, pause goal and wait.
   Do not implement unrelated residuals or begin another slice.

Four broad scan handles22812/49613/51376/46197 confirmed live at turn start.
Keep their existing runs/artifacts; old mixed-worker scans are exploratory,
matched scans use earlier frozen test-only WIP. Do not label them this slice's
immutable binary or restart because of observation timeout.

Independent upstream audit confirms owner mapping and identifies sibling
dependencies: branch_distance sample-step lower bound, preferred-angle and
tip-diameter source clamps, and brim's no-raft first-layer condition. Correct
those siblings as documented in ARD-0103. Do not claim every raw value must
produce different moves when upstream clamps it; preserve raw endpoint
readback and full comparisons.

Generic-angle RED13674 terminal/reaped100:1 failed,46.155s,3 actual strict
complete pairs PASS; native min/max support movements equal under the old
Organic co-injection. /tmp/ares-tree-activation-angle-red.log and artifact
root with the same prefix preserve the unchanged inputs and outputs.
This proves an activation blind spot, not a core output regression.
Select Slim at the source-owned harness boundary and then revalidate.

Phase2 RED62988 terminal/reaped100:103.195s, generic-angle tracer GREEN
38.682s with all3 strict complete pairs PASS and movement sensitivity;
style tracer RED64.509s after all7 strict complete pairs PASS, because raw
tree_slim still produced Normal Grid native moves. Same-prefix artifacts/log
/tmp/ares-tree-activation-phase2-seeded retain both observations. An earlier
module-path build17484 failed101 (no pairs), fixed with an explicit mod path;
invalid build output is preserved in /tmp/ares-tree-activation-module-path-error.log.

Now select Tree-auto for the four Tree-only styles; retain Normal-auto for
default/Grid/Snug. Add full public-domain readback/parity and native movement
witnesses for the known classic distance/diameter/brim dependencies and the
two Organic clamp controls. These are the same activation slice, not another
milestone or relaxed comparison. Unbounded brim-width has only the existing
min/seeded probes; it is not credited as full legal-range completion.

First frozen gates: workspace26006 terminal0,7617/7617,37 slow,9 skipped,
267.725s; clippy60917 terminal0,20.06s,baseline253/269/30; fmt/diff and
WASM65072 terminal0,51.66s compile-only. Logs /tmp/ares-tree-activation-final-*.
These do NOT accept the subsequent tip correction. Reference32550 official
terminal100,354.840s and95744 seeded terminal100,438.041s:3/4 functions PASS,
23 actual complete pairs PASS each; the tip function fails at native input
validation before producing output for minimum0.1. Preserve the rejected
exports/process captures; do not count a rejection as complete parity.

Print.cpp:1580–1610 adds the overlooked explicit-Organic raw-style gate:
tip>=support flow width and tip<=branch diameter (schema max10). Thus prior
Common.hpp clamp reasoning alone did not establish input legality. Independent
review requires separate public contracts: raw Grid normalized Organic full
schema0.1/100/seeded (clamped to branch10), explicit Organic legal0.1/10
finite controls, and separately recorded explicit Organic100 native rejection
(not Ares invalid-input parity). Co-inject support width0.1 and object/first
height0.08 for the minimum; read back every sibling. Same input geometry and
strict whole-output comparator; no endpoint removal or output waiver.

Tip correction diagnostic10774 terminal100: both focused functions RED,
327.190s;8 actual complete pairs,6 PASS and2 DIVERGENT, plus1 expected
native-only rejection. Artifacts/log /tmp/ares-tree-activation-tip-seeded.
Explicit Organic legal-min input SHA256
7938ad12dd28ee21ecf38791779f4787075ff661ce4ad21043a9f9bb7f1ddde2
is preserved at case-s2h8ql/input.3mf. First complete-output difference:
byte49575/line1902, M73 timing, normalized4557138 vs4557131 bytes.
Diagnostic inspection (not acceptance normalization) identifies one actual
support fill-connection difference near native line135516 at Z9.36:
native continuous inner contour vs Rust fragmented paths. Outer contour
matches; no proven production root cause. Owners under investigation are
SupportCommon.cpp:705–757/1764–1816, FillRectilinear.cpp:3610–3634 and
FillBase.cpp::connect_base_support:2247–2697, existing Rust raft/fills and
fill/base_support boundaries. Do not claim a timing-only fix or blame native
instability. Raw Grid minimum reproduces the same difference; explicit
maximum10 and raw maximum100/seeded complete pairs PASS.

Unchanged-input cached public replay11284 terminal101,79.28s, reproduces RED;
input/reference /tmp/ares-tip-diagnostic-replay, outputs
/tmp/ares-tip-replay-red. Temporary targeted boundary probe23682 terminal101
also reproduces RED; diagnostic only, its parent began CPU0 then moved to
CPU1 while retaining one worker. The probe confirms Rust emitted fragmentation
before G-code emission. Native debugger81478 exited0 on the same input,
but its selected Organic-hollow breakpoint was not entered because this
preset uses rectilinear base fill; it is not source-boundary equivalence
evidence or an official CPU0 repeatability verdict. No debug code is retained.

User/Claude update explicitly permits bounded checkpoint closure with this
known RED retained when the fill-connection gap cannot be fixed promptly.
Do not expand the activation slice into an indefinite connector port. Keep
both failing public minimum assertions, unchanged original fixture, all raw
domain values and strict comparator. No failure is credited as parity PASS;
native-only rejection is not Ares invalid-input parity. The four full quality
gates must pass, with opt-in native parity outcomes reported separately.
Commit/push the completed harness correction, synchronize latest origin/main,
repeat the four full gates, push main and pause. No further slice is started.

User-requested bounded cleanup: four earlier broad scans22812/49613/51376/
46197 received SIGINT and were reaped100 after3348.261/3040.128/2316.726/
2316.271s respectively. Their same-prefix logs and partial case artifacts
remain; interrupted inventories are not completed all-printer/all-option
coverage. Verified old test/native children no longer running. Diagnostic
5456 was deliberately terminated after capturing native Z9.36 base-flow and
source polygons at SupportCommon.cpp:1802; no completed reference output or
connector root-cause verdict is credited. No more probes or repair slices.

Final code snapshot:7 harness/docs paths only, no production changes;
Rust LOC392/381/64/232, all below400. Requested support style appears in
case labels to distinguish raw Grid normalization from explicit Organic.
fmt --check and diff --check PASS; diagnostic probe search empty. Independent
six-axis static review finds no new must-fix issues under the updated
checkpoint authorization; final approval awaits terminal quality/reference
receipts on this snapshot. These replace, not reuse, the earlier quality gates.

Final checkpoint terminal receipts, all reaped:

- Workspace89958 exit0:7618/7618 PASS,37 slow,9 skipped,263.646s.
- Clippy92632 exit0:253 core-lib/269 core-test/30 CLI parity/10 KSR
  warnings, unchanged baseline; logged5m36s includes lock wait.
- WASM38731 exit0:core+adapter wasm32-unknown-unknown compile-only;
  logged6m12s includes lock wait. fmt --all -- --check and diff --check PASS.
- Official60720 exit100:5 functions,3 PASS/2 FAIL,196.851s;28 actual
  complete pairs,26 PASS/2 DIVERGENT;1 separate expected native rejection.
- Seeded13896 exit100:5 functions,3 PASS/2 FAIL,211.837s;28 actual
  complete pairs,26 PASS/2 DIVERGENT;1 separate expected native rejection.

Logs/artifacts: /tmp/ares-tree-activation-checkpoint-{workspace,clippy,wasm,
official,seeded}.log and same-prefix reference artifact roots. Ares runtime
and reference each use CPU0 for the native pairs; seed0 on seeded replay,
untouched official2.4.2 otherwise. Only rustc diagnostic-build thread affinity
was widened, not either runtime worker domain. Both DIVERGENT pairs are
tip0.1 (raw Grid and explicit Organic); no additional native failures.
Slim branch angle/distance/diameter/brim and all7 styles complete pairs and
native movement witnesses PASS. Do not label the two failed opt-in functions
as green because the default workspace does not enable reference execution.

After feature commit/push, fetch latest origin/main and use fast-forward
ancestry synchronization in this isolated worktree. The original user's
dirty main worktree remains untouched. Repeat all4 quality gates on the
integration commit, logs /tmp/ares-tree-activation-integration-*.log; obtain
independent terminal confirmation and non-force push HEAD:main. Integration
receipts will be reported externally with the exact commit, not by adding
a documentation-only follow-up commit. Then pause the uncompleted global
matrix goal; no new slicing/connector slice or background sweep is started.
