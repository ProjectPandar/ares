# Plan: Air-filtration option activation

Source/Rust boundaries and deferrals: ARD-0048 and companion spec.
Agreed seams: CLI coverage contract and project-to-complete-G-code output.

1. Add one live Boolean tracer and capture inert-fixture RED.
2. Activate the six-key family with correctly scoped dependencies; remove
   only air filtration from support geometry/unsupported classification.
3. Extend through the other Boolean gates and both numeric domains;
   preserve all 14 complete paired outputs and run selected sweep.
4. Run workspace Nextest, fmt, Clippy and Tier-1 compiler checks.
5. Independent six-axis review, repair/re-review, commit code+docs, push.

## Evidence

Base 82175d5f, isolated worktree `/tmp/ares-cut-gate-isolated.rnuJct`.
The worktree matched that commit before checkout. Main's unrelated changes
are preserved. Verification is pending; no overall completion is claimed.

The master-gate tracer failed before activation: exported hardware
capability was 0 instead of 1 (exit 100, 0.770 s;
`/tmp/ares-air-red.log`, artifacts `/tmp/ares-air-red.oUKuZ8`). Adding
only family dependencies made both complete outputs pass (1.883 s;
`/tmp/ares-air-master.log`, `/tmp/ares-air-master.i543tU`).

Extending the family then produced two failures: capability probes
incorrectly enabled model supports, and the coverage contract routed the
capability to support geometry (`/tmp/ares-air-family-red.log`, 6 PASS /
2 FAIL, 2.495 s, `/tmp/ares-air-family-red.osmnoz`). Removing only this
key from the three support classifications made all eight tests pass
in 2.540 s (`/tmp/ares-air-live.log`, `/tmp/ares-air-live.vNxdUr`).
The six live tests directly compare 14 original outputs without retries;
the other two check domain coverage and support classification.
No production core or comparator code changed.

Independent static review found no required repairs. Final full checks,
selected sweep and final review were pending at this checkpoint.
WASM, Windows GNU and macOS ARM compiler checks have exited 0; logs:
`/tmp/ares-air-wasm.log`, `/tmp/ares-air-windows.log`,
`/tmp/ares-air-macos.log`. These are not runtime platform certification.

The actual selected sweep passed all 6 domains: 14 generated and strictly
compared, zero rejected (exit 0, 26.887 s;
`/tmp/ares-air-sweep.log`, `/tmp/ares-air-sweep.lsQ3el`). It does not replace
the historical all-option report. Artifact audit confirmed all 14 direct
comparison manifests PASS. Witnesses include S0/S255 at both speed
endpoints, S168 at the during-print seeded value and S127 at completion's
seeded value; disabling either global gate emits no P3 commands.
Original binary SHA-256 remains
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.

Initial full workspace: 7398 PASS, 9 skipped, 259.625 s
(`/tmp/ares-air-workspace.log`). Default `cargo clippy` succeeded with
warnings in 27.83 s (`/tmp/ares-air-clippy.log`), followed by fmt/diff checks.
An additional test-target Clippy run found new nesting/nonminimal-Boolean
warnings in the witness expectation. Replaced that conditional wrapper
with an equivalent early continue; independent repair review accepted it.

Final direct live suite again passed all eight tests / 14 full outputs
in 2.537 s (`/tmp/ares-air-final.log`, `/tmp/ares-air-final.2Bsn68`).
Re-ran the full workspace after this test-only cleanup: 7398 PASS, 9
skipped in 257.873 s (`/tmp/ares-air-workspace-final.log`). Test-target
Clippy succeeded in 1.49 s with 28 existing warnings, none in the new
air-filtration test (`/tmp/ares-air-test-clippy-final.log`). Final fmt and
diff checks passed; the verification chain exited 0. All four changed
Rust files are below 400 lines (289/352/295/172).

Final independent six-axis review approved only this eight-file code/doc
slice for commit/push with no required repairs. The broader baseline option
scan is separate diagnostic work and is not evidence for this patch's exit.
The overall all-option/all-printer parity goal remains open.
