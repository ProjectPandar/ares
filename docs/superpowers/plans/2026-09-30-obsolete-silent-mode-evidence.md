# Plan: Obsolete silent-mode input evidence

Base 180ee58d, clean isolated `/tmp/ares-obsolete-silent.M6IJE6`.
Source/Rust scope: ARD-0052. Reuse the agreed exported-3MF application-proof
and direct project-to-complete-output seams.

1. Reproduce both-value sweep; add an offline projection regression RED.
2. Implement only the exact obsolete-key/default proof rule.
3. Add rejection checks and two direct actual-Orca cases; rerun the sweep.
4. Nextest workspace, fmt/Clippy, Tier-1 compiler checks.
5. Independent six-axis review, repair/re-review, code+docs commit/push.

Diagnosis: the original scan and a selected repeat reject true because export
contains false; false already passes full comparison. Selected RED exit 100,
2.489 s (`/tmp/ares-silent-mode-red.log`, `/tmp/ares-silent-mode-red.hsUtzp`).
The unconditional obsolete-key consumption excludes firmware, missing second
limits and preset overwrite as activation fixes. Rust's project import
already consumes this key; the older SliceOptions seam is distinct and is
not changed here.

Offline RED: the old proof lacks obsolete-normalization metadata
(`/tmp/ares-silent-proof-red.log`, exit 100, 0.011 s); GREEN after the single
source-owned rule: 1 PASS, 0.011 s (`/tmp/ares-silent-proof-green.log`).
The expanded suite passes all three tests including two direct complete-output
comparisons, 1.897 s (`/tmp/ares-silent-direct.log`, `/tmp/ares-silent-direct.u6YPmW`).
The ordinary Boolean sweep passes 1/1 domains, 2 generated/strictly compared,
0 rejected, 2.888 s (`/tmp/ares-silent-sweep.log`, `/tmp/ares-silent-sweep.m8Nyx8`).
Its proof JSON retains requested 0/1, effective 0 and the explicit source rule.
Workspace Nextest: 7411 PASS, 9 skipped, 254.647 s
(`/tmp/ares-silent-workspace.log`). WASM browser-adapter, Windows GNU workspace
and macOS aarch64 workspace compiler checks pass
(`/tmp/ares-silent-{wasm,windows,macos}.log`); these are compiler checks, not
native/browser runtime tests. Default Clippy and explicit parity-test Clippy
pass with the existing 248 core / 28 parity-test warnings; fmt and diff checks
pass (`/tmp/ares-silent-{clippy,test-clippy,fmt}.log`). Independent six-axis
read-only review approved this eight-file slice with no required repairs.
All four direct/sweep manifests' input/output sizes and SHA-256 hashes were
verified. The invoked official v2.4.2 binary hashes to
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.
Invocation and binary identity are separate provenance evidence; retained
cache manifests alone do not independently prove their producer identity.

The full-option scan on base 180ee58d completed in the separate clean worktree:
exit 100, 1320.516 s, 553 PASS / 88 INCOMPLETE / 9 FAIL, 1639 generated,
1474 strictly compared, 1 rejected (`/tmp/ares-options-180ee58d.log`,
`/tmp/ares-options-180ee58d.TQGGY0/option-coverage-summary.md`). These inherited
report classifications do not prove exhaustive unbounded domains, nor do
97 non-PASS domains mean 97 core defects. This scan predates the present
silent-mode proof repair and is not merged with its selected result or used
as this patch's acceptance. Global parity remains incomplete.
