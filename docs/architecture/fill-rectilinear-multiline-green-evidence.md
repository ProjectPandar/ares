# Rectilinear multiline: directed GREEN

Base `5a9687ac020962f254b6262f2184f3345025c70f`, approved
[SPEC a56766fa4](fill-rectilinear-multiline-dispatch-spec.md).
[RED evidence](fill-rectilinear-multiline-red-evidence.md) is frozen at
`4e90b5c98`; original test commit `4d366f3b6` and approved setup correction
`98084d50c` remain separate. RED is only mergeable together with GREEN.

## Source port and mechanical prerequisite

Separate mechanical commit `cfbce13a8` moves the existing linear-pattern
sparse-anchoring arm to `sparse_anchoring/linear.rs`. It passes byte-exact
reconstruction after restoring the original indentation and undoing precisely
two rustfmt line wraps of bbox calls. Proof:
`/tmp/worker6-rectilinear-linear-arm-mechanical-proof.json`. No logic, assertion
or function-body value changed in that extraction; arms.rs is now 345 LOC.

GREEN `f1aff124ed29356e8258177c2f1aeb300f8cada9` adds a private 59-line
`fill/rectilinear_multiline.rs` adapter. Source-owned eligibility is only
Rectilinear with count>1 and not `density>0.9999f`, per
`FillBase.hpp:50`/`FillRectilinear.cpp:3387-3399`. It preserves f32 angle/layer/
thickness/reference expressions from `FillBase.cpp:275-319`, then delegates
exactly one `{0,0}` sweep to the existing multiline generator and Positive
numeric policy (`FillRectilinear.cpp:2996-3045`). Spacing remains unchanged.

Emission adds that branch within `fill_entities/monotonic.rs`, retaining flow,
collection sorting and residual processing. Anchoring adds the identical branch
within the extracted linear arm with the source overlap0
(`Fill.cpp:1477-1498`). Count1/full and other patterns retain their former
generation expressions. `MonotonicFillOutput` is an existing crate-private
result re-export used by the adapter; no public API is added.

No anchor_projection, bbox, option schema, guard, clipping primitive, shared
comparator or test assertion was edited. Changed Rust files stay below 400 LOC.

## Fresh directed results

Root `/tmp/worker6-rectilinear-green.FG23Hh` tests exact source HEAD f1aff124e.

| Mode | Exit | Summary |
| --- | ---: | --- |
| official | 0 | 12/12 PASS; 21.487s |
| seeded | 0 | 12/12 PASS; 29.147s |

The 1217 other tests are filtered, not tested by this directed selection.
Both legs have independent fresh Native caches/artifact roots and targets;
compiled dependencies are copied from completed worker6 targets, with CLI/test
fingerprints rebuilt. Outer taskset CPU11, jobs1/threads2; actual Native
sampling records 52 entries, all CPU11 and zero violations.

All five REDs (R2/R3/R10/R2A0/RH2) now pass public validated API and CLI
complete ordered generator-only byte comparisons and exact line counts; sparse
first differences are absent. Each Native export is checked for effective
requests and actual sparse consumption versus a count1 baseline.

All seven controls R1/RH1/RF100/M2/Z2/G2/S2 preserve BOTH API raw and CLI raw
bytes exactly versus RED in each mode: no normalization is needed for this
invariance check. All five additional count1 baselines also preserve both raw
streams. All 17 captures per mode have identical input ZIP member names/bytes
versus RED. Test and input-helper files are byte-identical to RED, preserving
every assertion and comparator. Audit:
`/tmp/worker6-rectilinear-green.FG23Hh/green-review-audit.json`.

Each capture retains runtime input, Native/API/CLI complete outputs, channels
and comparison.json under the external mode artifacts directory. No Native
artifact is tracked. The two historical setup errors remain separately recorded.

## Review and remaining gates

This is a directed code REVIEW request, not MERGE-READY. Strict/fmt/wasm32 are
running sequentially on CPU11, jobs1 and standard clean env; result root is
recorded at `/tmp/worker6-rectilinear-engineering-current`. Full option_coverage
two-mode validation, exact-hash self-hosted workspace and canonical exclusive
full remain pending. No push is authorized by these directed results.
