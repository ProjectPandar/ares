# Locked Zag width range plan

Boundary, decisions and exclusions: ARD-0045 and matching spec.

1. Add an endpoint/interior coverage regression for generated CLI plans.
   Observe RED on the old two-value plans.
2. Reuse the existing context-width generator for these two keys, add
   near-lower-edge candidates, and preserve explicit global incompleteness.
3. Execute all generated cases through the existing activation E2E tests,
   verifying typed settings, context and real sparse extrusion.
4. Add separate original-reference rejection checks for 0%, both exact
   lower boundaries and both just-above-upper values.
5. Run selected sweep, workspace/lint/platform checks and independent
   six-axis review; repair/review before combined code/doc commit/push.

## Verification record

Isolated checkout `/tmp/ares-locked-range-isolated.kGVd20`, base 20aaf485.
RED log: `/tmp/ares-locked-range-red.log`.
Earlier exploratory endpoint artifacts remain in
`/tmp/ares-locked-width-probe.r4raHE`; they motivate the cases but do not
replace this slice's live regression evidence.

Domain RED failed on missing literal-zero endpoint (0.482 s, exit 100),
then GREEN passed (0.493 s, exit 0; `/tmp/ares-locked-range-green.log`).
Initial live run `/tmp/ares-locked-range-live.HyXLHH` matched all 14
positive streams, but the rejection test incorrectly expected upper
violations at Slice rather than Export. Captured -18/key-specific
validation and `PrintConfig.cpp::validate` establish the correct earlier
stage; the test was repaired to require precise stages/codes, not relaxed
to accept any failure. Initial log: `/tmp/ares-locked-range-live.log`.

Fresh corrected live run `/tmp/ares-locked-range-verified.Pcp5FP` passed
all 4 tests in 5.313 s, including 14 full-stream comparisons with actual
sparse-extrusion witnesses and 10 exact rejection cases. Log:
`/tmp/ares-locked-range-verified.log`. The real selected sweep also compared
14/14 cases and correctly failed completion with two INCOMPLETE domains
(9.704 s, Nextest exit 100): `/tmp/ares-locked-range-sweep.log`, root
`/tmp/ares-locked-range-sweep.9vmNyI`. Its external summary is not merged
into the historical all-option summary. Rejection cases are separate
feasibility evidence, not successful G-code comparisons.

The shared raw-rejection candidate now advances by at least 0.1% or one,
because adding one to FLT_MAX does not advance its f64 representation.
Retained width metadata correctly separates the inherited schema ceiling
from the 2 mm/500% context ceiling. These raw-rejection metadata candidates
remain nonexecuted by the sweep; the ten E2E rejections are independent.

At this intermediate checkpoint, fmt and diff checks passed; workspace
Nextest, Clippy, Tier 1 compiler checks and final review were pending.

The first workspace run stopped with 1158 passed and two old plan-test
failures (21.356 s); 6222 tests were not run. Those assertions accepted
omission wording by keyword and required width metadata to imply legal
completion. They are replaced with behavior checks: every unproven plan
has an explanation, and complete comparisons of a limited width context
still report INCOMPLETE. Completion flags are not weakened to satisfy the
old tests. This failure required the full-suite rerun recorded below.

All ten plan tests then passed (0.953 s,
`/tmp/ares-locked-range-plan-tests.log`). Independent repair review found
no acceptance weakening: dedicated Locked tests keep the completion flag
false, and the shared test now verifies the actual incomplete result.
Scope is ten files after this necessary test repair.

The review's metadata wording suggestion was applied only to Locked
percent zero. A fresh selected sweep at
`/tmp/ares-locked-range-reviewed.vbZgwJ` again compared all 14 streams and
correctly returned exit 100 (9.650 s, `/tmp/ares-locked-range-reviewed.log`).
WASM, Windows and macOS compiler checks all completed successfully;
logs are `/tmp/ares-locked-range-` plus `wasm/windows/macos.log`.
They are compilation evidence, not cross-platform runtime parity.
Reference binary SHA-256 was rechecked as
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.

The repaired full workspace run completed with 7382 passed, 9 skipped,
in 256.771 s (`/tmp/ares-locked-range-workspace-final.log`).
`cargo clippy --workspace --all-targets` then completed successfully
in 1m 29s, with warnings (`/tmp/ares-locked-range-clippy.log`); this is
not a zero-warning claim. The chained verification process exited 0.
Final fmt and diff checks passed. All six changed Rust files remain below
400 lines (maximum 358). Independent six-axis final review approved this
limited code-and-documentation slice for commit/push with no required
repairs. Both domains remain INCOMPLETE; the overall parity goal is open.
