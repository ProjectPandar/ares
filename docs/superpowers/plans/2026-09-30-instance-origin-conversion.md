# Instance-origin conversion plan

The [spec](../specs/2026-09-30-instance-origin-conversion.md) defines the
upstream PrintInstance/GCode boundary and exact Rust destinations.

1. Reproduce Rolohaun complete-output failure and compare geometry,
   unscaled origin, and the final formatting input independently.
2. Preserve integer instance-shift construction when restoring emitter
   origin initially and per layer; reuse raw-center conversion.
3. Verify full official output and origin boundary regressions, existing
   multi-object/extruder tests, workspace tests and platform checks.
4. Independent read-only six-aspect review, repair/re-review as needed,
   and commit/push the code, tests and documentation together.

## Evidence and review repairs

- Official reference red: `/tmp/ares-rolohaun-solid-red.log`, three
  independent comparisons with the same Y6.014/Y6.013 difference. Input,
  complete streams and manifest: `/tmp/ares-rolohaun-solid-red/case-jpHneA`.
- Native path instrumentation (`/tmp/ares-rolohaun-native-paths.txt`) and
  rounding diagnostics establish identical integer coordinates and the
  differing origin arithmetic. Native instrumented output is diagnostic
  only, not the acceptance reference.
- Initial official green: `/tmp/ares-rolohaun-solid-green/case-XPJdDa`.
  Independent review then found that the initial origin incorrectly used
  the first source object, which may be nonprinting and unvalidated.
  The repair must select a resolved printable object and a printable
  instance; it does not introduce general multi-instance path expansion.
- The first nonprinting-prefix fixture hit a pre-existing start-template
  bounding-box dependency before origin initialization. Its start G-code
  is cleared to isolate origin selection; this is not a production fallback.
- Both review regressions have explicit red evidence:
  `/tmp/ares-instance-origin-prefix-red2.log` panics in `raw_center` on the
  nonprinting object's absent model part, and
  `/tmp/ares-instance-origin-instance-red.log` observes the wrong instance's
  zero translation. Initialization now uses a resolved source index, and
  translation uses the first printable instance.
- The repaired prefix fixture then reached an unrelated existing label
  mismatch: `gcode_emit/layers/schedule.rs` builds labels using output
  indices as source indices. The origin regression therefore checks
  successful complete slicing and the expected movement coordinate, not
  label identity. The existing nonfiltered multi-object label regression
  remains unchanged. Correct labeling after filtering nonprinting source
  objects is deferred to a separate upstream `GCode.cpp` object-label
  slice (`set_object_info` and per-instance start/stop labels);
  this result does not establish parity for that layout.

## Final verification

- All four origin boundary regressions pass after the review repairs:
  `/tmp/ares-instance-origin-boundaries-final.log`, including positive and
  negative rounding, separate center quantization, skipped nonprinting
  instance translation, and public slicing with a nonprinting prefix.
- After both repairs, official Rolohaun complete-output comparison passes:
  `/tmp/ares-instance-origin-final-rolohaun/case-He1Ybw` retains input,
  paired streams and manifest. Only generator identity/timestamp metadata
  is normalized; no movement/extrusion tolerance was added.
- Fresh related printer comparisons pass for Anycubic Kobra 3 0.2 nozzle,
  Sovol SV06 0.4 High-Speed nozzle, and RH3D E3NG v1.2S - 0.2 nozzle.
  Logs/artifacts: `/tmp/ares-instance-origin-final-kobra`,
  `/tmp/ares-instance-origin-final-sovol`, and
  `/tmp/ares-instance-origin-final-rh3d-corrected`. An earlier RH3D command
  used an unknown preset name and is not counted as a comparison.
- Both values of `print_sequence` and both authored `extruder_offset`
  cases pass complete official comparisons (four generated/four compared):
  `/tmp/ares-instance-origin-final-options.log` and sibling artifact root.
  No option domain or count changed in the historical option report.
- The historical modified-profile printer matrix is now 975/1001 after
  updating only Rolohaun's row. This is not a fresh full sweep or an
  untouched-default-profile acceptance result.
- WEMAKE3D TinyBotV1 was also rechecked and remains divergent at M73
  progress-command placement; no status change is claimed. Fresh complete
  pair: `/tmp/ares-instance-origin-final-wemake/case-PIzVq2`.
- fmt/diff checks and `cargo clippy` pass (227 existing core warnings).
  Core and WASM bindings compile for `wasm32-unknown-unknown`; core compiles
  for Windows GNU and macOS. Logs use `/tmp/ares-instance-origin-final-`
  with `clippy`, `wasm`, and `desktop` suffixes. These are compilation
  checks, not cross-platform runtime parity; live Orca comparisons ran on
  Linux. All changed Rust files remain under 400 lines.
- The pre-existing uncommitted geometry refactor was present during
  validation and remains outside this commit. Temporary Rust diagnostic
  probes were removed.
- Final `cargo nextest run --workspace`: 7364 passed, 9 skipped, exit 0;
  `/tmp/ares-instance-origin-final-workspace.log`. Offline oracle gates
  are not treated as live parity proof; the explicit official runs above
  supply that evidence for this slice.
- Final independent six-aspect review passed after the repair/re-review
  loop. No remaining repair items block this slice; the explicitly
  deferred neighboring limitations and overall parity gaps remain open.
