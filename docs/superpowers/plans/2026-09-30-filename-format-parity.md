# Filename-format parity implementation plan

Source/destination and non-negotiable decisions: ARD-0063.

1. Add a regression requiring three requested filename-format probes; observe
   failure while the domain is still baseline-only.
2. Add the minimal authored domain values. Exercise each through real original
   export, exact applied-value proof, original explicit output path and final
   config literal, followed by first-reference complete output comparison.
3. Run selected sweep, full Nextest, lint/format and platform compiler checks.
4. Independent six-axis review and repairs; commit/push only this code+docs
   slice, preserving unrelated dirty main changes.

Status: RED/GREEN, selected sweep, full checks and final six-axis review complete.
Based on 45e81266. `/tmp/ares-filename-red.log`: one behavioral FAIL (0.958s),
actual `[None]` versus the three required requested values. An initial test
compile error from accessing a private runner field was corrected before this
behavioral RED; no runner visibility or API expansion was introduced.

`/tmp/ares-filename-direct.log`: 2 PASS (2.914s), artifacts
`/tmp/ares-filename-direct.K5HutG`, three first-reference full comparisons.
The existing `slice_case` contract reads the explicit `plate_1.gcode`; successful
slice and comparison exercise that output path without private-field access.
Exact export proofs and final config literals pass for all three requested
values. Selected sweep artifacts: `/tmp/ares-filename-sweep.hFyvJS` (1/1 domain,
3 generated, 3 strictly compared, 0 rejected). All six successful manifests
have verified file sizes/SHA256. Manifest producer identity remains unknown;
the retained invocation and hashes are not independent producer provenance.

The immutable 26dbef57 full scan terminated independently: 376 PASS,
272 INCOMPLETE, 2 FAIL, 1653 generated, 1495 strictly compared, 1 rejected
(1362.889s), `/tmp/ares-options-26dbef57.tbYt1d`. It confirms this key was still
baseline-only before this change; its aggregate counts do not include this
slice or ARD-0062.

Full workspace Nextest: 7438 PASS, 9 skipped (261.478s),
`/tmp/ares-filename-workspace.log`. `cargo clippy`, parity-test Clippy,
`cargo fmt --check` and `git diff --check` pass; existing 248 core / 28 parity
warnings remain. Logs `/tmp/ares-filename-{clippy,test-clippy,fmt}.log`.
Compiler checks pass for ares-wasm/wasm32-unknown-unknown and workspace targets
x86_64-pc-windows-gnu / aarch64-apple-darwin, logs
`/tmp/ares-filename-{wasm,windows,macos}.log`. These do not claim native runtime
execution on those three targets. Independent final six-axis review approves
the seven-file slice with no required repairs. Approval covers explicit-path
configuration preservation, not recommended filename evaluation or global parity.
