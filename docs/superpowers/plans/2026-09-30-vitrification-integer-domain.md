# Vitrification integer-domain implementation plan

Boundary and decisions: ARD-0058; `PrintConfig.cpp:2917-2924`,
`Config.hpp:995-1085,2475-2479`, `GCode.cpp:3017-3040`. Existing Rust project
wire and start-placeholder code remain the rewrite destination. No new pipeline.

1. Add a failing generated-domain behavior test for signed endpoints and an
   interior sample (INT_MIN through INT_MAX-1, with INT_MAX rejection). Add original-binary first-reference output tests, including
   explicit start-template activation. Keep tests in a separate module.
2. Replace only this numeric authored-probe override with storage-bounded
   generation. Preserve 110. Fix production only against demonstrated output
   mismatch; defer multi-filament selection and generic SliceOptions behavior.
3. Run targeted live comparison and selected sweep, full workspace Nextest,
   rustfmt, Clippy and Tier 1 compiler checks. Retain complete output artifacts.
4. Independent six-axis read-only review, repair/re-review, then commit and push
   code and documentation together. Do not count this slice in a concurrently
   running scan of the preceding immutable commit.

Initial RED: generated-domain test found one case rather than four
(`/tmp/ares-vitrification-domain-red.log`, 0.497s). Direct original-binary run
(`/tmp/ares-vitrification-direct.log`, 2.234s) compared INT_MIN completely,
then INT_MAX aborted export with `ConfigurationError: Serializing NaN`.
Artifacts: `/tmp/ares-vitrification-direct.v0HLTH`, including INT_MIN paired
complete output at `case-3WGlHT` and rejected export at
`orca-runner-CTIg5i/command-D2TIOH`. This matches the upstream nil sentinel
serialization rule; no claim that INT_MAX generates comparable G-code.

Pre-review correction: live tests must execute the actual generated seeded
value, not substitute an unrelated hard-coded interior. Updated to iterate
generated cases plus supplemental negative and rejected-sentinel probes.

Targeted GREEN: `/tmp/ares-vitrification-green.log`, 2 PASS in 4.408s;
`/tmp/ares-vitrification-green.7hxR2j` retains five first-reference complete
outputs (INT_MIN, INT_MAX-1, seeded -837694717, 110 and -123456789), plus the
explicit INT_MAX rejected export. Selected sweep:
`/tmp/ares-vitrification-sweep.log`, 1 PASS in 3.976s;
`/tmp/ares-vitrification-sweep.Gpgmsy` reports 4 generated, 4 strictly compared,
0 rejected. The rejection is a separate direct negative, not a sweep pass.
All nine successful manifests' input/output lengths and SHA256 hashes were
independently checked. Manifest producer identity remains unknown; invocation
of the original binary is separate evidence, not an independent identity proof.

Independent implementation review accepts the wire-domain explanation and
generated-seed coverage; its orphan-comment cleanup was applied. Final full
validation and completion review results follow below.

First workspace run stopped after 1087 passes and one failure: the generic
range-label contract did not admit this key's retained `probe` after
`min/max/seeded` (`/tmp/ares-vitrification-workspace.log`, 21.336s). Extended
only that key's permitted label sequence, leaving all other domain contracts
and completion flags unchanged. Full validation is rerun after this repair.

Final validation: range-contract regression PASS (0.474s), workspace
7426 PASS / 9 skipped (268.488s), default Clippy and parity-test Clippy
successful with the existing 248 / 28 warnings, rustfmt and diff checks clean.
Logs: `/tmp/ares-vitrification-range-contract.log`,
`/tmp/ares-vitrification-{workspace-final,clippy-final,test-clippy-final,fmt-final}.log`.
WASM, Windows GNU and macOS ARM compiler checks passed, recorded in
`/tmp/ares-vitrification-{wasm,windows,macos}.log`; these are compiler checks,
not cross-platform runtime parity. Final independent six-axis review approves
this nine-file slice with no remaining required fixes. Approval is limited to
single-filament configuration/template behavior, not whole-program parity.
