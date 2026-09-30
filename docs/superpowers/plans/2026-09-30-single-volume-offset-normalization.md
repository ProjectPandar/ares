# Single-volume offset normalization plan

Base: a661945f. Decisions: ARD-0070.

1. Preserve the 2.572s complete-stream RED and debugger input evidence.
2. Add source-cited normalization at the existing project-load boundary;
   test public loaded transforms and all-instance world-placement preservation.
3. Re-run complete native prism parity, then revert only normalization to
   prove the test detects this behavior. Do not accept diagnostic injection.
4. Keep native controls and original KSR/concentric cases; record outcomes
   honestly even when adjacent differences prevent full GREEN.
5. Run full workspace, format/lint and Tier-1 compilation; request independent
   six-axis review, repair/re-review, then commit/push code and docs together.

Status: source normalization has sensitive full-output RED/GREEN; final
workspace/platform/lint gates passed; independent six-axis final review approved
the finite importer repair with no remaining required fixes. The temporary
concentric diagnostic entrypoint is isolated WIP and must not be shipped
with an absolute /tmp fixture path.

Evidence ledger:

- Original unnormalized three-field reference replay: process 77690 exit 100,
  2.572s, first difference line 273, width 0.394158 versus 0.394156;
  `/tmp/ares-concentric-prism-three-field-corrected.log` and full case
  `/tmp/ares-concentric-prism-three-field-corrected.OkE1nK/case-QlGglr`.
  The earlier command used wrong environment variable names and did not
  slice; `/tmp/ares-concentric-prism-three-field.log` is excluded.
- Native debugger captures show no small-line deletion, different Arachne
  input polygons and 110 different transformed vertices. Matrix probe was
  corrected for optimized argument ABI (rcx). Native capture logs
  `/tmp/ares-concentric-prism-native-trace.log`,
  `/tmp/ares-concentric-prism-native-matrix.log`; Rust capture logs
  `/tmp/ares-concentric-prism-rust-trace.log`,
  `/tmp/ares-concentric-prism-rust-matrix.log`. Vertex injection is diagnostic
  only and is not an accepted fix or comparison result.
- New normalization passes the original full-stream diagnostic (91480,
  2.584s) and the checked-in fixture test (83019, 2.098s). The final clean
  fixture test with explicit original component offset and instance rotation
  activation passes (85478, 2.249s), log
  `/tmp/ares-single-volume-offset-fixture-final.log`.
- Removing only the normalization call makes that fixture's complete
  comparison fail again (32620 exit 100, 2.098s), expected 32150 bytes,
  actual 32149, first difference line 273; full paired input/output
  `/tmp/ares-single-volume-offset-counterfactual.AXlTBK/case-bgTG3R`,
  log `/tmp/ares-single-volume-offset-counterfactual.log`. The call is restored.
  An earlier debugger force-return attempt failed because the function is
  inlined; it produced no G-code and is excluded as evidence.
- Independent review requested distinct instance rotations/scales and
  Eigen-relative-threshold evidence. Public load tests now include two
  different scaled/mirrored instances and all world vertices, exact/near
  offsets, threshold sides and zero-origin behavior, plus multi-volume
  exclusion. GCC 15.3.0 / Eigen 5.0.1 confirms 3e-11 is approximately equal
  at (10,20,30), 4e-11 is not, and a 1e-12 offset is not approximately zero
  at the world origin; `/tmp/ares-single-volume-isapprox.log`.
- Existing tests that assumed final volume-local centering were migrated
  individually to world composition or instance offset assertions, retaining
  their independently known world positions, mesh precision and provenance.
  Project runs exposed these old contracts; the latest interim run is
  292/293 PASS and one old volume-default assertion, compiled before its
  final world-coordinate migration. Full final workspace run is required;
  do not present that interim run as a completed gate.

Final gates: workspace 30500 exit 0, 7456 PASS / 9 skipped, 264.205s,
`/tmp/ares-single-volume-offset-workspace.log`. Clippy 70704 exit 0,
4m55s with existing warnings, `/tmp/ares-single-volume-offset-clippy.log`.
The earlier `fmt.log` contains a historical formatting diff, not a passed
check. After applying that formatting, final `cargo fmt --check` 44923 exits
0 with an empty `/tmp/ares-single-volume-offset-fmt-final.log`.
Tier-1 checks 82965 exit 0:
core/vgcode WASM 1m21s, Windows GNU 4m28s, macOS 49.71s, Linux 34.37s;
WASM adapter 22.21s, logs `/tmp/ares-single-volume-offset-<target>.log`
and `/tmp/ares-single-volume-offset-wasm-adapter.log`.
Original KSR replay 95608 exit 100, 45.010s: complete ordered output remains
DIVERGENT, first difference at line 3 (model printing time 1h43m53s versus
1h43m52s), expected 6321717 / actual 6322010 normalized bytes. Retained
full paired artifacts `/tmp/ares-single-volume-ksr-replay-output.FUXtP4/case-IUSCmV`
and log `/tmp/ares-single-volume-ksr-replay.log`; no additional normalization.
This remains a separate open global parity observation, not a finite importer
exit requirement or a claim of global completion.
