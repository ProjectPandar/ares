# Active hot-plate parity plan

See the [spec](../specs/2026-09-30-hot-plate-activation.md) and
[ARD](../../architecture/ard-0034-hot-plate-activation.md).

1. Add a public-output test for effective plate selection and heating commands;
   run against official Orca to reproduce missing activation.
2. Add only the option-scoped plate dependency and correct probe labels.
3. Compare all complete outputs, run workspace Nextest, fmt and Clippy.
4. Independent read-only six-aspect review, repair/re-review, commit and push.

## Evidence

- `/tmp/ares-hot-plate-red.log`: old fixture fails because effective plate
  is Cool Plate, not High Temp Plate.
- `/tmp/ares-hot-plate-green.log`: three full-output comparisons pass for
  zero, 55 and 300; effective plate/value and first M140/M190 are checked.
  Full projects and paired outputs are retained in the sibling artifact root;
  `case-rjh81V` contains the maximum-temperature result.
- `/tmp/ares-hot-plate-options.log`: six bed-type values and three active
  temperature probes all compare successfully. The sweep exits 101 because
  the temperature domain correctly remains INCOMPLETE, not because an output
  differs. No lower-bound or random-coverage claim is made.
- Historical report updates only this row from two to three compared cases;
  complete-domain count stays 614/650. This is not a fresh whole-matrix run.
- fmt, diff checks and Clippy pass (227 existing core warnings). All changed
  Rust files are below 400 lines. No production code or dependencies change.
  Pre-existing geometry changes are included in checks but excluded from the
  commit. Overall all-option/all-printer parity remains incomplete.
- The first workspace run was interrupted without a terminal summary and is
  not counted as passing. After verifying its runner and children had exited,
  `cargo nextest run --workspace` was rerun: 7369 passed, 9 skipped, exit 0,
  255.301 seconds, `/tmp/ares-hot-plate-workspace-final.log`. Environment-gated
  offline tests do not replace the 12 explicit official output comparisons.
- Independent read-only six-aspect final review approves this slice with no
  repair items, after checking all 12 complete output pairs, effective plate
  selection, heating commands, report totals and terminal test results.
