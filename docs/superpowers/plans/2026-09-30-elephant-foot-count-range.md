# Elephant-foot count range implementation plan

Base: 92207777. Source/destination and architecture decisions: ARD-0067.

1. Require min/max/full-range seed at the agreed native domain seam; observe RED.
2. Generate the scalar range and activate geometric compensation. Verify exact
   export, final config, full ordered output and changed motion versus control.
3. Diagnose and repair only evidenced discrepancies in the cited upstream slice.
4. Selected sweep, full checks, independent six-axis review/repair; commit/push.

RED: one FAIL (0.942s), old two samples versus required three,
`/tmp/ares-elephant-count-red.log`. Native direct GREEN: three tests PASS
(3.584s), including the generic range-label contract,
`/tmp/ares-elephant-count-direct.log`. Four complete first-reference outputs
(zero-compensation control plus 1 / INT_MAX / seeded 1407454340) are retained under
`/tmp/ares-elephant-count-direct.42Mmrh`. Exact exported and final counts,
positive compensation, 100% density and zero raft are asserted. Original
motion differs from control and multi-layer counts differ from single-layer
compensation; these diagnostics do not replace full-output comparison.
All four manifest file sizes/SHA256 are verified. Reference producer identity
remains unknown in the manifests, not established by file hashes.
Existing Rust allocation/ramp behavior needs no production change.

Selected sweep: one PASS (10.082s), three generated/strict comparisons and
zero rejected, `/tmp/ares-elephant-count-sweep.log`; retained artifacts
`/tmp/ares-elephant-count-sweep.rDhpUT`. All three additional manifest file
sizes/SHA256 are verified. Tier-1 compiler checks pass for ares-core,
ares-vgcode and ares-wasm on wasm32-unknown-unknown and the workspace on
x86_64-pc-windows-gnu / aarch64-apple-darwin; logs
`/tmp/ares-elephant-count-{wasm,windows,macos}.log`. These are compilation
checks, not runtime tests on those targets.

Full workspace Nextest: 7446 PASS, 9 skipped (268.990s),
`/tmp/ares-elephant-count-workspace.log`. `cargo clippy`, parity-test Clippy,
`cargo fmt`, `cargo fmt --check` and `git diff --check` pass, logs
`/tmp/ares-elephant-count-{clippy,test-clippy,fmt}.log`; existing 248 core /
28 parity warnings remain. Changed Rust modules stay below 400 lines.

Final independent six-axis review approves the eight-file slice, with no
remaining required repairs. This does not claim global option/printer parity.
Independent pre-review finds no required repairs. The earlier pause is revoked by the explicit
request to continue all tasks; the complete objective remains unchanged.
