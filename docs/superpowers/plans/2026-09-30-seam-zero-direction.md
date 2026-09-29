# Zero seam direction plan

See the [spec](../specs/2026-09-30-seam-zero-direction.md) and
[ARD](../../architecture/ard-0031-seam-zero-direction.md) for the source-owned
boundary and exclusions.

1. Reproduce nozzle max versus official Orca; inspect movement, not only M73.
2. Lock the case at public project-to-G-code output, including nearest seams.
3. Match upstream 2D normalization, then rerun original and adjacent domains.
4. Run workspace/lint/platform checks, independent six-aspect review, repair
   and re-review as needed; commit and push code and docs together.

## Diagnosis

- `/tmp/ares-nozzle-diameter-red.log`: max fails in three completed official
  attempts; seeded interior passes. First full pair is `case-4Ostet` in
  `/tmp/ares-nozzle-diameter-red`. Inner wall start differs, not just timing.
- Ranked hypotheses: large-arm candidate angles; inner seam projection;
  final loop splitting. A diagnostic native score run reports concave
  -pi/2 candidates at 100 mm and matches the official inner corner. It is
  diagnostic only, not the acceptance oracle.
- The closed polygon repeats its first point. Aligned and nearest Rust
  direction normalization unconditionally divide by length, unlike Eigen's
  squared-norm-positive branch in `MatrixBase::normalized`. A zero previous
  direction therefore produces NaN before projection. No polygon dedup or
  timing change is part of this correction.

## Verification

- Focused regression first fails on aligned output:
  `/tmp/ares-nozzle-focused-red.log` (three official attempts).
- Changing only aligned's 2D normalization makes both aligned and nearest
  cases pass: `/tmp/ares-nozzle-aligned-fix.log`, with complete artifacts in
  its sibling root (`case-bnfFy1` aligned, `case-oCI75J` nearest). Both
  requested nozzle and seam values are checked in the exported project.
  Nearest's separate normalizer is deliberately unchanged, since this
  control does not reproduce a failure there.
- Original nozzle max/seeded plus all five seam positions, four seam-gap
  probes and both staggered values pass: 13/13 complete official outputs,
  `/tmp/ares-nozzle-zero-options.log` and sibling artifact root. The
  historical option matrix changes only nozzle FAIL to PASS and domain
  count 611 to 612; generated/compared totals are unchanged.
- Replay of all 973 PASS pairs from the immediately preceding official
  printer sweep passes 973/973, exit 0, without comparator changes:
  `/tmp/ares-nozzle-printer-replay.log`, paired outputs and JSON summary in
  `/tmp/ares-nozzle-printer-replay`. Input symlinks in
  `/tmp/ares-nozzle-printer-replay-input.H1GVyv` point to original full
  projects and official outputs. This is cached-reference regression
  evidence, not a new official sweep; its 18 oracle errors and 10 missing
  vendor profiles remain unverified and are not credited by this replay.
- `cargo fmt`, `cargo fmt --check`, `git diff --check` and `cargo clippy`
  pass. Core plus WASM bindings compile for `wasm32-unknown-unknown`;
  core compiles for Windows GNU and macOS. Logs:
  `/tmp/ares-nozzle-zero-{clippy,wasm,desktop}.log`. These are compilation
  checks, not native/browser runtime certification.
- Modified source files are 225, 357 and 44 lines; tests remain separate.
  No diagnostic probe remains. Checks include the pre-existing geometry
  worktree changes, which are excluded from this slice's commit.
- `cargo nextest run --workspace` exits 0: 7366 passed, 9 skipped,
  `/tmp/ares-nozzle-zero-workspace.log`. Offline environment-gated tests
  do not substitute for the explicitly executed official comparisons.
- Independent read-only six-aspect final review approves this slice with
  no remaining repair items, after independently checking all 15 fresh
  output pairs, the 973-case replay scope and terminal check results.
  Overall all-option/default-printer parity remains incomplete.
