# Slow-layer count implementation plan

Source/destination and architecture decisions: ARD-0064. Base: e3534246.

1. Require scalar min/max/full-range seed and short ramp probes at the existing
   option-domain seam; observe failure before changing generation.
2. Generate the representable scalar range. Run each requested value through
   original export, exact application proof, final config and first full
   ordered output comparison. Verify real motion activation versus zero.
3. Selected sweep, full Nextest, lint/format and Tier-1 compiler checks.
4. Independent six-axis review and repair/re-review, then commit/push only
   this code+docs slice, preserving unrelated main-worktree changes.

Status: RED/GREEN, selected sweep, final full checks and six-axis re-review complete.
`/tmp/ares-slow-layer-red.log`: one FAIL (0.973s), old two local samples versus
five required cases. `/tmp/ares-slow-layer-direct.log`: two PASS (4.067s),
artifacts `/tmp/ares-slow-layer-direct.EhEEMN`; five first-reference complete
outputs for 0, 2147483647, seeded 774797067, 1 and 4. Exported exact values and
final config literals pass. Feed-bearing G1 lines equal the disabled baseline
for 1, and differ for every active count; the full parity comparator is unchanged.
No production speed change was needed.

Selected sweep: one PASS (6.373s), five generated/strict comparisons, zero
rejected, `/tmp/ares-slow-layer-sweep.log`; artifacts
`/tmp/ares-slow-layer-sweep.8NTNHr`. All ten successful direct/sweep manifests
have verified file sizes/SHA256. Manifest producer identity remains unknown;
recorded invocation and hashes are not independent producer provenance.

Initial full workspace attempt stopped at 1703 PASS / 1 FAIL, with 5736 tests
not run: the existing bounded-range label contract did not accept the two
additional slow-layer probes (`/tmp/ares-slow-layer-workspace.log`). Independent
review identified the same required repair. `tests.rs` now accepts only this
key's precise five-label sequence; no other domain contract is relaxed.
Focused domain/contract regression: 2 PASS (0.934s),
`/tmp/ares-slow-layer-regression.log`.

The review's additional no-raft assertion was adopted: exported `raft_layers`
must equal 0. Final direct/contract regression: 3 PASS (10.546s),
`/tmp/ares-slow-layer-final.log`, five full paired outputs under
`/tmp/ares-slow-layer-final.ApuZrG`, also verified against manifest sizes/hashes.
Final workspace/lint chain passes. Independent six-axis re-review approves the
eight-file slice after the exact-label and no-raft repairs, with no remaining
required fixes. Approval does not claim global parity completion.

`/tmp/ares-slow-layer-workspace-final.log`: 7440 PASS, 9 skipped (267.298s).
`cargo clippy`, parity-test Clippy, `cargo fmt --check` and `git diff --check`
pass, logs `/tmp/ares-slow-layer-{clippy,test-clippy,fmt}.log`; existing 248 core
and 28 parity warnings remain. Tier-1 compiler checks pass for ares-wasm on
wasm32-unknown-unknown and workspace on x86_64-pc-windows-gnu / aarch64-apple-darwin,
logs `/tmp/ares-slow-layer-{wasm,windows,macos}.log`. Cross-compilation does not
claim native runtime tests on those targets. The user requested a pause after
this slice's approved code+docs commit and push; no next slice is started.

The latest immutable
26dbef57 full scan lists slow_down_layers INCOMPLETE with two local comparisons;
its aggregate results must not be rewritten from future targeted evidence.
