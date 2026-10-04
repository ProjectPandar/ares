# Investigation plan: Qidi Grid collection ordering

Experiment1 rejected: no_sort=false produced0/2QidiPASS,60/68PASS,712/823controls
PASS,4/8officialPASS. That flag-only edit was reverted; no gate or acceptance
claim applies to it. Artifacts `/tmp/ares-qidi-{two,full68,
controls823,official8}/replay-summary.json` preserve every failure. Source follow-up
found `Fill.cpp:1325 params.can_reverse=false` for Grid, separately propagated
per path by FillBase.cpp:183-185. This proves the one-field mapping incomplete;
corrected experiment uses existing Oriented paths plus sortable collection.

[Spec](../specs/2026-10-04-grid-collection-ordering.md),
[ARD](../../architecture/ard-grid-collection-ordering.md).

1. Establish strict RED on7cfa0b33, branch `pi/qidi-grid-residuals`, worktree
   `/tmp/ares-pi-matrix-vendors`, isolated target
   `/tmp/ares-pi-matrix-vendors-target`, build jobs2.
   Input `/tmp/ares-qidi-replay` contains only unchanged frozen Qidi Q1 Pro0.2
   case-WE5l76 and X-Max4 0.2 case-b8PGl3 pairs.
   `ARES_PARITY_REPLAY=/tmp/ares-qidi-replay`
   `ARES_PARITY_ARTIFACT_ROOT=/tmp/ares-qidi-red`
   `cargo nextest run -p ares-cli --test orca_parity`
   `-E 'test(replay::orca_parity_replay_sweep)' --nocapture` exits100 (0/2PASS).
2. Two independent source semantics: Grid collection no_sort=false and each
   generated path materialized as existing Oriented (non-reversible), matching
   Fill.cpp:1325 and FillBase.cpp:164,183-185. Preserve reverse-on-odd-layer and
   connector behavior; do not modify generic ordering.
3. Repeat Qidi2, full68,823controls and8 original official CPU0 references.
   First hypothesis is falsified if full output remains different; preserve
   failures and investigate source boundaries, never weaken comparisons.
4. Run workspace Nextest, fmt, default Clippy, WASM build core+adapter. Inspect
   diff/probes/ownership. Push and print REVIEW-REQUEST, then wait for APPROVED.
   Rebase latest main, rerun gates and ff-only main push after approval.
5. Only then continue #11 upstream PerimeterGenerator extra overhang logic;
   never resolve that existing guard simply by deleting it.

No new isolated test is added after code: existing public complete-byte E2E
replay and immutable Orca fixtures are the pre-established regression seam.

## Final candidate evidence on7cfa0b33

Root cause confirmed: the existing Grid collection-level no_sort flag suppressed
both sorting and reversal. Upstream independently allows path ordering but forbids
reversal. Reusing Oriented paths plus no_sort=false restores both contracts;
no generic ordering, geometry, timing or comparator changes were required.
Production delta is two materialization choices plus one source comment.

| Check | Result | Artifact |
| --- | --- | --- |
| Baseline Qidi2 | 0/2PASS, exit100 | `/tmp/ares-qidi-red/replay-summary.json` |
| Corrected Qidi2 | 2/2PASS | `/tmp/ares-qidi-oriented-two/replay-summary.json` |
| Original68 vendor residual subset | 68/68PASS | `/tmp/ares-qidi-oriented-full68/replay-summary.json` |
| Prior-PASS stock controls | 823/823PASS | `/tmp/ares-qidi-oriented-controls823/replay-summary.json` |
| Original official CPU0 representatives | 8/8PASS | `/tmp/ares-qidi-oriented-official8/replay-summary.json` |
| Newly sliced official2.4.2 CPU0 Qidi2 | 2/2PASS | `/tmp/ares-qidi-official-fresh-green/replay-summary.json` |
| Workspace Nextest | 7594/7594PASS,9 existing skips | `/tmp/ares-qidi-oriented-workspace.log` |
| fmt check | exit0 | `/tmp/ares-qidi-oriented-fmt.log` |
| Default cargo clippy | exit0,249 pre-existing core warnings | `/tmp/ares-qidi-oriented-clippy.log` |
| Actual WASM core+adapter build | exit0 | `/tmp/ares-qidi-oriented-wasm.log` |

Aggregate exits `/tmp/ares-qidi-oriented-exits.txt` are all0. Fresh original
Orca invocation logs and input/output pairs: `/tmp/ares-qidi-official-fresh`.
Previous flag-only regressions remain in their separate `/tmp/ares-qidi-*`
experiment artifacts; no successful result was spliced into those failed runs.
All checks use unchanged full ordered-byte comparison (generator line only).
No new probes, suppressed warnings, guards, ignored tests or deleted assertions.

### Integration freeze

Coordinator has frozen main merges for the lint geometry slice. Branch push and
REVIEW-REQUEST are allowed. Even after APPROVED, wait for an explicit thaw notice
before latest-main rebase/revalidation and ff-only integration; do not infer thaw
from the approximate45-minute estimate. #11 remains next after this slice.

