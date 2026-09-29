# Seam sample scalar precision plan

The [spec](../specs/2026-09-30-seam-sample-precision.md) defines the source
owner, Rust destination, included arithmetic and deferred adjacent behavior.

1. Add and run WEMAKE's complete-output regression; inspect the complete
   diff, not just its first differing line.
2. Isolate the first-layer seam selection. Compare upstream sampled
   position construction and test its scalar conversion boundary.
3. Rerun the original strict comparison, related seam values and printers,
   workspace Nextest, formatting, Clippy and cross-platform compilation.
4. Independent read-only six-aspect review, repair/re-review, then commit
   and push implementation and documentation together.

## Diagnosis

- `/tmp/ares-wemake-progress-red.log`: strict printer regression fails.
- Ignoring M73 only for diagnosis isolates the first-layer wall/travel
  order; acceptance still compares complete output with only generator
  identity/timestamp normalization.
- Direct official invocation output is retained at
  `/tmp/ares-wemake-official-direct/plate_1.gcode` with invocation log.
- `/tmp/ares-wemake-native-score.txt` reports first-layer candidates with
  near-equal visibility and selects the lower-right corner. This native
  instrument is diagnostic only; its different speeds are not accepted.
- Changing only weight grouping did not resolve the failure
  (`/tmp/ares-wemake-weight-green.log`, despite its provisional name);
  that isolated experiment was reverted before testing vector reduction.
- `/tmp/ares-wemake-native-precise3.txt` exposes native visibility values
  near 1.6: the lower-right candidate is 1.5999999, while the upper-right
  is 1.60000014. Ares's initial candidate scores instead select the upper
  right (`/tmp/ares-wemake-score-probe.log`). This rules out an initial
  cursor explanation for the selected seam.
- Vector reduction and combined weight-grouping experiments also failed
  and were reverted. A standalone Eigen multiplication experiment
  (`/tmp/ares-wemake-eigen-probe.cpp`) establishes that weight
  0.6856440703409491 times vertex coordinate 5 produces 3.42822051 after
  scalar conversion, versus 3.42822027 when the product is narrowed.
- Sample conversion alone still failed (`/tmp/ares-wemake-sample-probe.log`);
  combining it with upstream visibility-weight grouping passes the complete
  comparison (`/tmp/ares-wemake-sample-weight-probe.log`). Both changes are
  necessary to this observed output regression. The timing processor is
  unchanged; its differences disappear with the corrected wall/travel path.

## Final verification

- Clean official WEMAKE smoke passes after removing all temporary Rust
  probes: `/tmp/ares-wemake-sample-final.log`. Complete input, paired G-code
  and manifest: `/tmp/ares-wemake-sample-final/case-zlV48D`.
- Vector reduction experiments are absent from the final diff. Production
  changes are confined to sample scalar conversion and visibility weights;
  all affected source files are below 400 lines.
- All five `seam_position` values, both `staggered_inner_seams` values,
  and four generated `seam_gap` probes pass complete official comparisons:
  `/tmp/ares-seam-sample-options.log` and its sibling artifact root. Only
  the freshly verified seam-gap case count changes in the historical
  option report (three to four); domain coverage remains 611/650.
- fmt/diff checks and Clippy pass (227 existing core warnings). Core and
  WASM bindings compile for `wasm32-unknown-unknown`; core compiles for
  Windows GNU and macOS. Logs: `/tmp/ares-seam-sample-{clippy,wasm,desktop}.log`.
  These are compile checks, not native/browser runtime certification.
- Validation includes the pre-existing uncommitted geometry refactor,
  which is not part of this slice. No diagnostic probes remain in Rust.
- `cargo nextest run --workspace`: 7365 passed, 9 skipped, exit 0
  (`/tmp/ares-seam-sample-workspace.log`). Offline oracle gates do not
  replace the explicit live official comparisons above.
- Fresh 1001-printer sweep completes with 973 PASS, 18 ORCA_ERROR and
  10 VENDOR_INCOMPLETE, with no Ares errors or output divergences:
  `/tmp/ares-seam-sample-printer-sweep.log` and sibling artifact root.
  WEMAKE now passes. Oracle crashes changed which profiles could be
  compared (including new Custom/MyToolChanger failures); historical
  passes are not substituted for this run's missing comparisons.
  The committed printer matrix contains this fresh run, not a merged
  best-of result. The harness exits 0 despite these explicit unverified
  categories; that exit is not proof of the user's overall goal.
- Printer results use the documented modified classic-wall cube baseline,
  not untouched vendor defaults. All-printer/default parity and complete
  option coverage remain open; this slice only fixes seam arithmetic.
- Independent read-only six-aspect final review approves this slice with
  no remaining repair items. The reviewer independently compared all 12
  focused output pairs and checked the fresh sweep's counts, 973 paired
  artifact sets and official command evidence. This approval is scoped
  to the seam fix, not overall printer/option parity completion.
