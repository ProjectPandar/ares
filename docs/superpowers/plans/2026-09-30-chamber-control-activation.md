# Plan: Chamber-control activation

Base 0fa2e0f4, isolated `/tmp/ares-cut-gate-isolated.rnuJct` verified clean.
Source/Rust boundary: ARD-0049 and companion spec. Use the previously agreed
CLI domain contract and project-to-complete-G-code seams.

1. Add the capability Boolean tracer; capture incorrect support activation.
2. Correct only chamber capability's support classification and co-inject
   the selected family's required machine/filament/template settings.
3. Expand to all 12 generated cases and two startup-suppression cases;
   retain original and Ares complete output with command witnesses.
4. Run selected sweep, Nextest workspace, fmt/Clippy, platform compilers.
5. Independent six-axis review, repair/re-review, code+docs commit/push.

The baseline full-option sweep is separate diagnostic evidence, not this
patch's verification or global completion.

## Execution evidence

Capability tracer RED: exported activation was 0 instead of 1
(`/tmp/ares-chamber-red.log`, exit 100, 0.769 s;
`/tmp/ares-chamber-red.JPxMwT`). Family activation plus the three exact
support-classification corrections made both capability values pass
(`/tmp/ares-chamber-capability.log`, 1.867 s;
`/tmp/ares-chamber-capability.3XEDRT`).

The expanded suite initially passed 7/8 tests. Its auxiliary-fan witness
also captured the separate earlier cooling initialization command
(`GCode.cpp:2822`), not a production difference. Restricted that local
witness to the source-owned custom-role/start-template interval; the full
output comparator remains unchanged. Repaired suite: 8 PASS, 14 direct
complete-output comparisons, 2.518 s (`/tmp/ares-chamber-reviewed.log`,
`/tmp/ares-chamber-reviewed.Y7OzrO`). No core production code changed.

Selected sweep: 5/5 domains PASS, 12 generated/strictly compared, 0 rejected,
8.502 s (`/tmp/ares-chamber-sweep.log`, `/tmp/ares-chamber-sweep.4KOr6s`).
All 14 direct artifact manifests report PASS; retained input/output sizes
and SHA-256 digests were checked. The invocation used the official 2.4.2
binary with SHA-256
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.
The cached-byte manifests do not independently establish producer identity;
the live invocation and binary check are separate provenance evidence.

Final isolated checks all exited 0: `cargo nextest run --workspace`
(7406 PASS, 9 skipped, 253.913 s; `/tmp/ares-chamber-workspace.log`),
`cargo clippy` (existing 248 core warnings), explicit parity-test Clippy
(existing 28 test warnings), `cargo fmt --all -- --check`, and `git diff
--check`. Logs: `/tmp/ares-chamber-clippy.log`,
`/tmp/ares-chamber-test-clippy-final.log`, `/tmp/ares-chamber-fmt.log`.
Compiler checks passed for wasm32-unknown-unknown (`ares-wasm`),
x86_64-pc-windows-gnu and aarch64-apple-darwin (workspace); logs
`/tmp/ares-chamber-{wasm,windows,macos}.log`. These are compiler checks,
not native runtime tests on those platforms.

Independent reviewer `review_completion_gate` approved the eight-file slice
after static pre-review and terminal-evidence re-review. Requirements,
logic, boundaries, code quality, coverage and actual results passed with no
required repairs. Approval is limited to this slice, not global parity.

The earlier whole-inventory run on base 82175d5f completed in 1456.912 s
with exit 100 (`/tmp/ares-options-current.log`, report and full artifacts
`/tmp/ares-options-current.pydj2g`). It reports 546/650 domains complete,
1639 generated, 1464 strictly compared and 1 rejected. Its 104 non-PASS
domains include application-proof errors and known unsupported/nondeterministic
boundaries; this is not a claim of 104 core slicing defects. It predates
air-filtration and this chamber activation, so is not this patch's exit
evidence and does not replace or merge the historical tracked summary.
