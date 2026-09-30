# Plan: Preset identity activation

Base 6009561e; reused clean `/tmp/ares-options-current.rPJrPW` after verifying
its complete index tree equals the committed tree. Preserve unrelated main
worktree changes. Agreed seams and source boundary: ARD-0057.

1. Reproduce the three export mismatches and add activation regression RED.
2. Bind only the owning preset metadata for each identity.
3. Verify generated and second identities in project and final G-code, then
   compare six first official complete outputs and run the selected sweep.
4. Full tests, formatting, lint, Tier-1 compiler checks; independent six-axis
   review, repairs/re-review; code+docs commit and push.

Baseline RED: `/tmp/ares-preset-ids-red.log`, exit 100, 2.055 s.
Artifacts `/tmp/ares-preset-ids-red.EX9jkQ`: requested `ares-parity-probe`
becomes GFL99, Creality Generic PLA and the default Ender-3 process name.

The separately completed immutable base-17a214e0 full scan has 555 PASS,
89 INCOMPLETE and 6 FAIL; 650 domains, 1638 generated, 1476 strict comparisons,
1 rejected. `/tmp/ares-options-17a214e0.log`, exit 100, 1506.341 s;
report `/tmp/ares-options-17a214e0.4mXlYz/option-coverage-summary.md`.
It predates thumbnail and current identity fixes; counts are not merged.

Activation regression RED: 0.500 s
(`/tmp/ares-preset-ids-activation-red.log`), original filament ID survives
instead of the request. After owner binding: 2 PASS, 4.145 s, six direct
first-reference complete comparisons (`/tmp/ares-preset-ids-direct.log`,
`/tmp/ares-preset-ids-direct.mnfbvQ`). Selected sweep: 1 PASS, 3.634 s,
three generated/strict comparisons, zero rejected
(`/tmp/ares-preset-ids-sweep.log`, `/tmp/ares-preset-ids-sweep.Pqgs1q`).

Independent pre-review found no mandatory repair; its coverage-hardening
suggestion was adopted: both tests find every named key explicitly, so a
missing target domain cannot silently reduce coverage. Final targeted run:
2 PASS, 11.226 s with six original complete outputs
(`/tmp/ares-preset-ids-final.log`, `/tmp/ares-preset-ids-final.3IIBju`).
WASM adapter, Windows GNU workspace and macOS aarch64 workspace compiler
checks pass (`/tmp/ares-preset-ids-{wasm,windows,macos}.log`); compiler
checks do not establish platform runtime parity.
Final full/lint/format results follow below.

The final six direct and three selected-sweep manifests have verified full
input/output file sizes and SHA-256 hashes. Official binary SHA-256:
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.
Invocation/hash evidence does not upgrade the manifests' unknown cache
producer provenance into an independent producer-identity guarantee.
First full workspace: 7424 PASS, 9 skipped, 263.909 s
(`/tmp/ares-preset-ids-workspace.log`); default/parity Clippy passes with
existing 248/28 warnings, fmt/diff checks pass. Re-review confirms the
missing-domain guard and reports no remaining required repair. Final exact-code
workspace: 7424 PASS, 9 skipped, 263.536 s
(`/tmp/ares-preset-ids-workspace-final.log`); default/parity Clippy
(248/28 existing warnings), fmt and diff checks all pass in the same
exit-0 chain. Final independent six-axis review approves the eight-file slice
with no remaining code repair items. Approval is limited to the single-filament
finite identity samples and excludes the global parity gate and old full scan.
