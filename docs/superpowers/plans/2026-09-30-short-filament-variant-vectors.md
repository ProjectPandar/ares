# Short filament variant vectors plan

Base: 541370f1. Decisions: ARD-0072.

1. Preserve the ARD-0071 full failures. Add one minimized public-byte native
   fixture isolating short filament payloads and run complete output RED.
2. Distinguish raw zero-fill from wrong indices or premature normalization;
   preserve printer/process selection. Implement the cited filament branch.
3. Require complete GREEN and reversion RED; retain original positive-density
   and Auto fixtures as open separate behavior, not excluded output lines.
4. Run workspace, fmt/Clippy/Tier-1 checks; independent six-axis read-only
   review, repair/re-review, then code/docs commit/push together.

Status: source-owned filament zero selector implemented, with complete native
GREEN and reversion RED; final gates passed and independent six-axis final
review approved this finite slice. Global parity remains incomplete.

Evidence:

- Initial reduced RED 69633 exit 100, 2.339s: first difference line 17,
  air-filtration `1,0` versus `1,1`, full case
  `/tmp/ares-filament-variant-zero-red.tLSWF5/case-3vQez6`.
- Changing only the filament selector resolves all scalar/nullable
  differences; 24564 exit 100, 2.348s retains a separate map-mode mismatch.
  Merely setting Manual in project JSON did not activate plate ownership.
  Reusing the existing `bind_filament_map_mode` owner and activation check
  gives full GREEN 31295 exit 0, 2.367s, case
  `/tmp/ares-filament-variant-zero-manual.JhoYR5/case-E4miNb`.
- Strengthened nullable bool/enum short-vector case gives full GREEN
  51580 exit 0, 2.112s, paired artifacts
  `/tmp/ares-filament-variant-zero-enums.RmUcNz/case-sCVQ1y`.
  In-range Spiral Lift is preserved, missing enum is Auto Lift (discriminant
  zero, not the Rust enum default Slope); missing nullable bool is false.
- Restoring only first-value broadcast, 20633 exit 100 / 2.282s, makes the
  complete comparison fail at line 17 again (161970 versus 161839 bytes);
  `/tmp/ares-filament-variant-zero-counterfactual.T8QO6u/case-7Qm13U`.
  Zero selection is restored before final gates.
- The prior short-filament volumetric-speed test expected a copied first
  value. Its assertion now preserves the independently known first value
  and requires numeric zero in the missing slot; printer/process broadcast
  tests remain unchanged.
- Fixture strengthening with active Top Only lift exposed an adjacent
  runtime lift/timing divergence, retained complete in
  `/tmp/ares-filament-variant-zero-two-types.79ZL4S/case-FbOH6T`.
  This behavior is not repaired or filtered by this slice. The final candidate
  fixture preserves original active lift behavior while independently
  activating missing nullable bool, enum, percent and string slots.

The first final fixture attempt 97310 failed its activation assertion because
native serializes an empty string as a trailing semicolon, not quoted `""`.
The independently observed native representation is now asserted exactly;
34190 exit 0 / 3.313s is full ordered GREEN with empty string, bool and enum
slots, `/tmp/ares-filament-variant-zero-string.tpLPgr/case-1FaR3a`.

First workspace 85131 exit 100: 7456 PASS / 1 FAIL / 9 skipped, 269.418s.
The old public flush-placeholder test expected short payloads to broadcast.
The first material selects the first payload value; the second material's
selected variant exceeds the two-entry payload (the original KSR ZIP has
eight identity slots, with the second material starting at raw index 4).
Thus `[18,0]`, `[15,20]` and `[230,0]` produce `[18,0]`, `[15,0]` and
`[230,0]`. `GCode.cpp:2876-2887` retains the
zero speed fallback and replaces zero temperature with the ordinary material
range-high vector `[250,260]`; the corrected literal is `18,0 230,260`.
No production placeholder code changed. Final full rerun 35727 exits 0:
7457 PASS / 9 skipped, 257.158s,
`/tmp/ares-filament-variant-zero-workspace-final.log`.
Clippy 60485 and Tier-1 checks 1008 terminated successfully; final test-only
assertion changes are checked by fmt/workspace 35727 and Clippy 73665 (exit 0,
47.87s); final fmt log is empty, and Clippy retains existing warnings only.
Tier-1 core/vgcode: WASM 2m07s, Windows GNU 3m05s, macOS 12.24s, Linux
14.72s; WASM adapter 8.60s. Logs are
`/tmp/ares-filament-variant-zero-<target>.log` and `-wasm-adapter.log`.

KSR replay 71188 exit 100 / 35.900s was ARTIFACT_ERROR, not an ordered
comparison: no comparison completed. The error is consistent with concurrent
rebuilding removing the executable pathname before its build-identity read;
this is an inference, not a validated harness root cause. Retained directory
`/tmp/ares-filament-variant-zero-ksr-replay.40Sn3P`. Re-run after the final
workspace compilation settles; do not claim this run as KSR parity evidence.

Serialized KSR replay 96835 exit 100 / 42.681s completed one strict comparison:
DIVERGENT at line 3 (1h43m53s versus 1h43m52s model time), expected 6321717
/ actual 6322010 normalized bytes. Full paired artifacts
`/tmp/ares-filament-variant-zero-ksr-serialized.MS7WCL/case-gUR4zR`.
The Ares file is byte-identical to the prior ARD-0070 Ares KSR result
(`/tmp/ares-single-volume-ksr-replay-output.FUXtP4/case-IUSCmV/ares.gcode`);
this is a retained unchanged failing baseline, not a KSR GREEN claim.

Source revision
541370f1 full option sweep 38766 terminated exit 100 / 2410.824s, with
386 PASS / 262 INCOMPLETE / 2 FAIL, 1661 generated / 1517 strict comparisons
/ 1 rejected. It used its already built immutable binary and the three-field
initialized reference; it is not the new patch's acceptance gate. Full report
`/tmp/ares-options-541370f1-initialized.5K1EdV/option-coverage-summary.md`.
