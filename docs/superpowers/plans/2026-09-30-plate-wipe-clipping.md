# Plate-frame wipe implementation plan

The [spec](../specs/2026-09-30-plate-wipe-clipping.md) defines the upstream
wipe/origin boundary and Rust destination.

1. Reproduce the complete Kobra 3 0.2 nozzle comparison failure.
2. Compare wipe integer points and clipping arithmetic; minimize the
   observed case into output-behavior regression coverage.
3. Carry the active upstream emission frame into wipe clipping without
   changing the stored local geometry.
4. Verify the original full output, related wipe option cases, Rust checks
   and cross-platform compilation. Independently review and fix/re-review.
5. Commit and push code, coverage evidence and documentation together.

## Verification

- Official reference red: `/tmp/ares-kobra-wipe-red.log`, three independent
  attempts fail at the same E distribution. Minimized output regression
  red: `/tmp/ares-wipe-frame-unit-red.log`.
- The native instrumented build was used only to inspect integer points
  and arithmetic (`/tmp/ares-kobra-native-wipe.txt`), not for acceptance.
  The acceptance smoke uses the selected Orca 2.4.2 AppImage extraction.
- Official Kobra green retains complete paired output, input and manifest
  at `/tmp/ares-kobra-wipe-green/case-iuz36W`. Independent review confirmed
  the full diff contains only generator metadata.
- Sovol and RH3D live rechecks remain PASS; Rolohaun remains DIVERGENT.
  Logs/artifacts use `/tmp/ares-wipe-frame-{sovol,rh3d,rolohaun}`.
  The historical modified-profile matrix is 974/1001 after this one-row
  update, not a fresh all-printer or untouched-default-profile sweep.
- All changed source files stay under 400 lines. Temporary probes were
  removed; the pre-existing geometry worktree changes remain out of scope.
- All 14 generated cases for `wipe`, `wipe_distance`, `wipe_speed`,
  `role_based_wipe_speed`, `retract_before_wipe`, and `retraction_length`
  match complete official output. Logs and raw paired artifacts:
  `/tmp/ares-wipe-frame-options.log`, `/tmp/ares-wipe-frame-options`.
  The historical option matrix keeps 611/650 domains; only the freshly
  verified `wipe_speed` case count changes from three to four.
- The minimized regression passes (`/tmp/ares-wipe-frame-unit-green.log`).
  fmt/diff checks and Clippy pass, with 227 existing core warnings.
- Core/WASM binding checks for `wasm32-unknown-unknown` and core checks for
  Windows GNU/macOS pass. These are compilation checks, not runtime parity;
  real-Orca comparisons ran on Linux.
- `cargo nextest run --workspace`: 7359 passed, 9 skipped, exit 0;
  `/tmp/ares-wipe-frame-workspace.log`. Validation includes the existing
  uncommitted geometry refactor, which is not part of this commit.
- Final independent six-aspect review passed with no remaining repair
  items; all complete-output comparison evidence was independently checked.
