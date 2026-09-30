# Plan: Cut-retraction mode coverage

Follow ARD-0046 and the matching spec. This is validation of the
`LongRectrationLevel` / filament-override / G-code placeholder rewrite,
not a new Ares pipeline.

1. Add a failing CLI domain-contract test requiring named modes 0/1/2.
2. Expand authored probes; retain the incomplete-range gate.
3. Activate conflicting machine/filament settings and a rendered
   cut-retraction witness; require requested/effective application proof
   and complete-output parity through the agreed project-to-G-code seam.
4. Run the selected sweep, workspace, lint and platform checks.
5. Obtain independent six-axis review, repair/re-review, commit/push only
   this slice with its documents.

## Evidence

Isolated base 1cecd903 at `/tmp/ares-cut-gate-isolated.rnuJct`.
At plan creation, verification was pending. All integer endpoints, multi-tool cutting and
other contexts remain outside the evidence of this limited slice.

Domain RED failed on the missing modes (0.494 s, exit 100,
`/tmp/ares-cut-gate-red.log`); after expanding probes, GREEN passed
(0.500 s, `/tmp/ares-cut-gate-green.log`).
Live activation RED found default distance 18 instead of requested 12
(0.798 s, `/tmp/ares-cut-gate-live-red.log`, root
`/tmp/ares-cut-gate-live-red.FWy3yE`). Conflicting dependency injection
fixes that missing activation.

The first witness expectation incorrectly assumed numeric Boolean
formatting; reference rendering is false/true. A corrected run at
`/tmp/ares-cut-gate-verified.JyyvVj` matched the entire mode-0 and mode-1
streams, then showed an inserted M73 after the mode-2 template move.
The local witness now checks the expected prefix; whole-stream comparison
still preserves and compares every M73 line. Logs:
`/tmp/ares-cut-gate-live.log`, `/tmp/ares-cut-gate-verified.log`.
At that checkpoint, no production fix or output normalization had been introduced.

The subsequent full-stream run exposed a production defect, not another
witness issue: mode 2's first difference is byte 1158 / line 46, original
true/16 plus G1 E-16 versus Ares false/12 with no move. It reproduced
in two fresh roots: `/tmp/ares-cut-gate-runtime.WFKgi2` (2.464 s)
and `/tmp/ares-cut-gate-repeat.847xon` (2.411 s), both exit 100.
Logs use the corresponding `/tmp/ares-cut-gate-runtime.log` and
`/tmp/ares-cut-gate-repeat.log` paths.

Diagnosis keeps the small standard cube and mode-0/1 controls instead
of minimizing away the complete-output seam. Ranked hypotheses:
runtime override application missing; placeholder parser reads the full
configuration; later initialization overwrites a resolved value.
A targeted temporary diagnostic compares resolved vectors with template
vectors immediately before scalar aliases, then must be removed.
Repair scope was expanded to the core placeholder boundary; review approval
was withheld pending that regression and the original E2E passing.

The targeted diagnostic confirmed hypothesis two: in mode 2 the resolved
runtime vectors are true/16 but the template dictionary still contains
false/12 from the full-config block. Diagnostic log:
`/tmp/ares-cut-gate-diagnose.log` (2.448 s, exit 100), root
`/tmp/ares-cut-gate-diagnose.bJdnbf`.
The temporary DEBUG-cut-gate instrumentation is removed. A small
`placeholders/cut_retraction.rs` module now inserts the two runtime
vectors before existing scalar aliases. This keeps the parent below
400 lines and leaves the full-config footer untouched.
Review also requested exact G1-list assertions inside the witness block;
those now require no move for 0/1 and exactly G1 E-16 F1800 for 2,
while allowing processor-inserted M73 lines for full-stream comparison.

After the repair, the original three-mode loop passed all comparisons
(2 tests, 2.449 s, `/tmp/ares-cut-gate-fixed.log`, root
`/tmp/ares-cut-gate-fixed.VwTWbN`). Added reverse Boolean conflicts
(machine true, filament false) to verify a concrete false override is not
treated as absent. All six mode/conflict combinations matched complete
output (3 tests, 2.466 s, `/tmp/ares-cut-gate-six.log`, root
`/tmp/ares-cut-gate-six.aYkpKt`).

The actual selected sweep compared 3/3 cases with no rejections and
correctly returned INCOMPLETE/exit 100 because the coInt endpoints are
unverified. Artifacts: `/tmp/ares-cut-gate-sweep.LdHcAc`, log:
`/tmp/ares-cut-gate-sweep.log`. No historical full summary is merged
with this partial run. Workspace/lint/platform checks were then started.

Broader existing live smoke regression passed 29 tests in 13.955 s
(`/tmp/ares-cut-gate-smoke.log`, root
`/tmp/ares-cut-gate-smoke.SOU9qK`). This is the existing smoke set,
not a new all-printer sweep. The official reference SHA-256 was rechecked:
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.

Smoke artifact audit found 47 matching labels and one retained intermediate
DIVERGENT attempt for top-surface concentric (M73 placement). That case
matched a subsequent original output under the existing three-run reference
budget; this is not an oracle-determinism claim. The six dedicated cut-mode
comparisons use direct `compare_case`, not that retry wrapper, and all
six first reference outputs matched. WASM/Windows/macOS compiler checks
completed successfully (platform chain exit 0); these are compile checks,
not native/browser runtime certification.

Full workspace Nextest completed with 7385 passed and 9 skipped in
257.266 s (`/tmp/ares-cut-gate-workspace.log`). Final fmt and diff
checks passed. Platform logs are `/tmp/ares-cut-gate-` followed by
`wasm.log`, `windows.log`, and `macos.log`. Independent repair review
found no required changes. Clippy then completed successfully with warnings
in 1m 29s (`/tmp/ares-cut-gate-clippy.log`); the chained verification
process exited 0. This is not a zero-warning claim. Independent six-axis
final review approved the eleven-file code/documentation slice for
commit/push with no required repairs. The integer domain remains INCOMPLETE
and the overall all-printer/all-option parity goal is still open.
