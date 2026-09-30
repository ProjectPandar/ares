# Printer preset identity implementation plan

Source/destination and non-negotiable scope: ARD-0062.

1. Extend existing preset-identity seam tests to the printer key. Require a
   requested authored case, correct machine name and unchanged unrelated
   fields. Observe the failing regression before changing activation.
2. Add the printer string domain and metadata activation with the smallest
   change to the existing helper. No production change without output evidence.
3. Exercise two direct identity values through exact export proof, final
   config literal and complete output. Re-run all four identity domains.
4. Full checks, independent six-axis review and repair/re-review; code and
   docs committed/pushed together. Preserve unrelated dirty main changes.

Status: implementation, direct regression, full checks and final independent
six-axis review complete. Worktree is based on 26dbef57. Its full scan
remains live in a different worktree and must not be restarted or edited.

RED `/tmp/ares-printer-identity-red.log`: baseline-only case fails requested
identity assertion (0.957s). After adding the authored value, activation RED
`/tmp/ares-printer-identity-activation-red.log`: two failures (4.978s), including
original Slice/Process -17; `/tmp/ares-printer-identity-red.5fDBf4` retains evidence.
Name-only activation passes the unit check but original Export/Process still
rejects -17 (4.767s), `/tmp/ares-printer-identity-green.mdpmX2`. Minimal one-key
sweep reproduces it in 3.275s, `/tmp/ares-printer-identity-minimal.8uahkL`.
Compatibility-list regression then fails before repair (0.939s),
`/tmp/ares-printer-identity-compat-red.log`.

Replacing only old-name references in the process compatibility list resolves
the source-owned rejection. `/tmp/ares-printer-identity-final.log`: 2 PASS
(5.647s); `/tmp/ares-printer-identity-final.82l1CH` retains eight first-reference
complete paired outputs (two values across each of four identities), strict
applied-value proof and final reference literals. No production/comparator or
compatibility-policy relaxation was needed.

Selected sweep `/tmp/ares-printer-identity-sweep.log`: 4/4 domains,
4 generated/strictly compared, 0 rejected (5.373s), artifacts
`/tmp/ares-printer-identity-sweep.gSbVhZ`. All 12 successful manifests from
direct and selected runs have verified file lengths/SHA256. Manifest producer
identity remains unknown; recorded invocation and hashes are not independent
reference provenance. Independent pre-review approves the implementation.

Full workspace Nextest: 7436 PASS, 9 skipped (269.909s),
`/tmp/ares-printer-identity-workspace.log`. Both `cargo clippy` and parity-test
Clippy complete successfully (existing 248 core / 28 parity warnings),
`/tmp/ares-printer-identity-{clippy,test-clippy}.log`; `cargo fmt --check` and
`git diff --check` pass. Tier-1 compiler checks pass for wasm32-unknown-unknown
(ares-wasm), x86_64-pc-windows-gnu and aarch64-apple-darwin (workspace), logs
`/tmp/ares-printer-identity-{wasm,windows,macos}.log`. These are compiler checks,
not claims of native runtime tests on all platforms. Final independent
six-axis review approves the eight-file identity slice with no required
repairs. Approval does not cover the live full scan or global parity completion.
