# Skirt height boundary plan

See the [spec](../specs/2026-09-30-skirt-height-boundaries.md) and
[ARD](../../architecture/ard-0033-skirt-height-activation.md) for source
ownership and the narrow harness destination.

1. Reproduce the old incomplete domain and add public skirt-layer checks.
2. Replace fixed authored probes with source minimum/maximum/random generation;
   activate finite-skirt dependencies.
3. Compare full outputs for generated and one/two-layer cases, test negative rejection;
   run workspace Nextest, formatting and Clippy.
4. Independent read-only six-aspect review, repair/re-review, then commit
   and push this slice's code and docs together.

## Evidence

- `/tmp/ares-skirt-height-red.log`: original authored domain compares its
  three cases but fails the completion gate as INCOMPLETE. Constant 2 is
  not a seeded random interior probe.
- `/tmp/ares-skirt-activation-red.log`: focused test rejects the baseline's
  two loops when one is required for this finite-skirt context.
- `/tmp/ares-skirt-height-green.log` is an unsuccessful intermediate run:
  max, seeded and zero match, but Orca rejects negative height with exit
  238 and a `skirt_height: invalid value -1` diagnostic. Source inspection
  locates the actual lower bound at `PrintConfig.cpp:10422-10424`; do not
  infer accepted input from `Print::has_skirt`'s downstream branch alone.
  The final plan therefore includes effective minimum zero and tests
  negative rejection separately, never crediting it as a G-code match.
- `/tmp/ares-skirt-height-final.log`: five complete output comparisons pass
  (min/max/seeded/one/two), alongside the attributable negative rejection,
  exit 0. All inputs and paired outputs are in the sibling artifact root;
  `case-Us7eHt` contains the ceiling result. The public test verifies height,
  loop count and shield in the exported project, and counts actual skirt
  layers in official output before the complete byte comparison.
- `/tmp/ares-skirt-height-options.log`: height min/max/seeded, loop-count
  min/max/seeded and both draft-shield values pass, 8/8 comparisons, exit 0.
  The historical report updates only skirt height (one old recorded case
  to three newly compared cases), domain count 613 to 614 and totals +2.
  Negative rejection in the separate focused test is not counted as a
  generated legal-domain case, a byte match, or an excused requirement.
- fmt, diff checks and `cargo clippy` pass (227 existing core warnings).
  No production core/adapter code or dependencies change. Modified Rust
  files stay below 400 lines. Checks include pre-existing geometry changes,
  which are excluded from this commit. All-option/default-printer parity
  remains incomplete.
- `cargo nextest run --workspace` exits 0: 7368 passed, 9 skipped,
  `/tmp/ares-skirt-height-workspace.log`. Offline environment-gated passes
  do not substitute for the 13 explicit official output comparisons.
- Independent read-only six-aspect final review approves with no remaining
  repair items, after checking all 13 complete output pairs, negative
  rejection, effective configuration, skirt counts and terminal results.
  The reviewer also recomputed all report totals from its rows.
