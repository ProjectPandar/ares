# Bottom-shell count implementation plan

Source/destination and architecture decisions: ARD-0065. Base: 3a98d03d.

1. Require scalar min/max/full-range seed at the existing option-domain seam;
   observe failure before changing generation.
2. Generate the representable range and exercise exact requested/effective
   export values, final config and first-reference complete output. Require
   zero thickness and actual motion changes for active counts.
3. Selected sweep, full Nextest, lint/format and Tier-1 compiler checks.
4. Independent six-axis review and repairs, then one code+docs commit/push.

Status: RED/GREEN, selected sweep, full checks and final six-axis review complete.
`/tmp/ares-bottom-shell-red.log`: one FAIL (0.957s),
old two samples versus required three. Direct regression:
`/tmp/ares-bottom-shell-direct.log`, three PASS (3.035s); three first-reference
complete paired outputs for 0, INT_MAX and seeded 1706227297 under
`/tmp/ares-bottom-shell-direct.3grNa2`. Export exact values, final config,
zero-thickness precondition and active motion differences pass. The existing
range-label contract also passes; no comparator or production change needed.

Selected sweep: one PASS (10.252s), three generated/strict comparisons and
zero rejected, `/tmp/ares-bottom-shell-sweep.log`; artifacts
`/tmp/ares-bottom-shell-sweep.5v34xJ`. All six successful manifests have verified
file sizes/SHA256. Producer identity remains unknown in the manifests; retained
invocation and hashes are not independent producer provenance.

Full workspace Nextest: 7442 PASS, 9 skipped (263.211s),
`/tmp/ares-bottom-shell-workspace.log`. `cargo clippy`, parity-test Clippy,
`cargo fmt --check` and `git diff --check` pass, logs
`/tmp/ares-bottom-shell-{clippy,test-clippy,fmt}.log`; existing 248 core / 28
parity warnings remain. Compiler checks pass for ares-wasm/wasm32-unknown-unknown
and workspace x86_64-pc-windows-gnu / aarch64-apple-darwin, logs
`/tmp/ares-bottom-shell-{wasm,windows,macos}.log`. These are cross-compilation
checks, not runtime tests on those targets. Independent pre-review reports
no required repairs. Final independent six-axis review approves the seven-file
bottom-shell range slice, without claiming 650-domain or global parity completion.

Work is resumed after
the completed ARD-0064 pause. The 26dbef57 immutable scan reports this key
INCOMPLETE; its aggregate results remain a separate historical snapshot.
