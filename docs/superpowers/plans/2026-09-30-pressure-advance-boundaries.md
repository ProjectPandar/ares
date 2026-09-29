# Pressure advance boundary plan

The [spec](../specs/2026-09-30-pressure-advance-boundaries.md) defines upstream
ownership, Rust boundary and included/deferred behavior.

1. Add a public slicing regression requiring the true ceiling and active
   emitted PA commands, including zero and negative controls; run red.
2. Remove the old authored interior-only probes and inject filament enablers.
3. Compare complete official outputs and rerun workspace Nextest/fmt/Clippy.
4. Independent read-only six-aspect review, repair/re-review, then commit and
   push code and documentation together.

Initial audit also ran hot-plate/skirt/long-cut probes:
`/tmp/ares-option-bounds-red.log`, seven output pairs match but all three
domains remain INCOMPLETE. They are not changed or credited by this slice.

## Evidence

- `/tmp/ares-pressure-boundaries-red.log`: the focused test fails because
  the plan has no value 2.
- After removing only the authored probes,
  `/tmp/ares-pressure-activation-red.log` fails with effective
  `enable_pressure_advance=[0]`, not the required `[1]`.
- After filament-scoped activation, the public regression passes all four
  complete outputs: max, seeded interior, zero and negative. Log:
  `/tmp/ares-pressure-boundaries-green.log`; complete paired artifacts in
  the sibling root. `case-8f092W` holds the ceiling output.
  The test verifies effective values and distinguishes `M900 K0`, `M900 K2`
  and negative suppression in official output before full comparison.
- `/tmp/ares-pressure-boundaries-options.log`: both enable booleans and
  generated max/seeded PA values pass, 4/4 official comparisons, exit 0.
  The historical matrix updates only PA to PASS and 612 to 613 domains;
  case totals stay unchanged. The separate zero/negative regressions are
  not included in that generated-domain count.
- fmt, diff checks and `cargo clippy` pass (227 pre-existing core warnings).
  No production core/adapter code or dependencies change in this slice.
  Edited/new Rust files are all below 400 lines, and tests stay separate.
  Validation includes existing geometry worktree changes, excluded from
  this commit. Full all-option/default-printer parity remains incomplete.
- `cargo nextest run --workspace` exits 0: 7367 passed, 9 skipped,
  `/tmp/ares-pressure-boundaries-workspace.log`. Environment-gated offline
  passes are not substituted for the eight explicit official comparisons.
- Independent read-only six-aspect final review approves this slice with
  no remaining repair items, after checking all eight complete output pairs,
  exported configuration, PA commands and terminal verification results.
