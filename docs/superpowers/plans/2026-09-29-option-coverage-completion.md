# Option coverage completion implementation plan

The source boundary and deferred upstream behaviors are defined in
[the spec](../specs/2026-09-29-option-coverage-completion.md).

1. Extract the existing completion check without changing behavior and add
   regressions in a separate test module. Verify they fail for the existing
   exemptions and empty input.
2. Require nonempty, all-PASS outcomes. Preserve report-before-failure order,
   case generation, and complete G-code artifacts.
3. Run focused tests, formatting, Clippy, workspace Nextest, and real Orca
   boolean/full versus filtered-domain sweeps.
4. Independently review requirements, logic, boundaries, quality, coverage,
   and actual results. Fix findings and re-review. Commit code and these
   documents together, then push.

## Verification record

- Red: all three new completion regressions failed against the extracted
  original gate (Nextest run `91ff0f32-0204-4c84-95b0-b5d2c27371b6`).
- Green: all 20 option-coverage tests passed. The default invocation leaves
  live-oracle cases offline; the following explicit runs supply that evidence.
- Real Orca 2.4.2, `ARES_OPTION_KEYS=accel_to_decel_enable`:
  both boolean values passed strict comparison; Nextest exited 0.
  Artifacts: `/tmp/ares-completion-gate-66BQcR/full`.
- The same key with `ARES_OPTION_VALUES=1`: the individual case passed,
  its domain reported INCOMPLETE, and Nextest exited 100 as required.
  Complete paired G-code and input remain under
  `/tmp/ares-completion-gate-66BQcR/filtered/case-criEpy`.
- `cargo fmt --check` passed; `cargo clippy` exited 0 with 227 existing
  core warnings.
- The first workspace build found missing test imports after the existing
  uncommitted EdgeGrid module split. Explicit test imports of EdgeGrid,
  Coord, and Point repair compilation without modifying geometry behavior.
- The repaired worktree passed `cargo nextest run --workspace`: 7353 passed,
  9 skipped.
  Log: `/tmp/ares-completion-gate-66BQcR/workspace.log`. Existing uncommitted
  geometry changes were present during validation but are not part of this
  slice's commit. The parity test target also passed Clippy (existing warnings).
- Independent review found a stale gate comment and a duplicate ARD number;
  both were corrected and re-reviewed.
- Fresh `raft_layers` min/max/seeded sweep compared all three cases and
  failed as expected. The max case still differs at line 4885:
  `E-.20595` in Orca versus `E-.20594` in Ares, across three reference
  attempts. Full paired outputs and report:
  `/tmp/ares-completion-gate-66BQcR/raft`. This remains a real slicing
  residual, not a completion-gate defect.

This slice does not close the full parity objective. The tracked summaries
still report 610/650 option domains and 971/1001 modified-profile printers;
they are historical evidence, not a fresh full sweep on this worktree.
