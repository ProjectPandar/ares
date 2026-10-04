# Thin-tip support fill connection plan

See the matching spec and `docs/architecture/ard-thin-tip-support-fill.md`.
Base: c3f6164530d6984aa1384c70ee47c7f7e3e553c6, branch
`pi/thin-tip-support-fill`, worktree `/tmp/ares-pi-tip`. All Cargo commands use
`CARGO_TARGET_DIR=/tmp/ares-pi-tip-target`; do not touch the main worktree.

1. Reproduce the unchanged public cached tip fixture RED; inspect upstream
   `FillBase.cpp::connect_base_support` and `base_support_extend_infill_lines`
   against Rust `fill/base_support`. Preserve output and first differences.
2. Rank falsifiable connection/extension/graph hypotheses; use a bounded
   upstream debugger observation where necessary. Repair only the proven
   mismatch and rerun the same full-output test, then both thin-tip contracts.
3. Run all five activation tests against official 2.4.2 CPU0 and seed-only
   CPU0/seed0; require 28/28 complete pairs each plus native rejection evidence.
4. Run `cargo nextest run --workspace`, `cargo clippy`, `cargo fmt`/check and
   `cargo check -p ares-core -p ares-wasm --target wasm32-unknown-unknown`.
   Inspect scope/debug residue, record exact results, commit and push the branch.
5. User follow-up requires main integration: after feature push, fetch and
   rebase latest origin/main in this isolated worktree, rerun all four quality
   gates, then `git push origin HEAD:main` (fast-forward only). On rejection,
   fetch/rebase and repeat verification; never force push or touch the main
   worktree.

Included/deferred boundaries are fixed by the spec; earlier ARD-0103 results
are prior evidence, not acceptance of this implementation.

## Diagnosis and red/green receipts

The existing public `replay::orca_parity_replay_sweep` with
`ARES_PARITY_REPLAY=/tmp/ares-tip-diagnostic-replay`, external artifact root,
`taskset -c 0` and the isolated Cargo target reproduced RED: exit100,
0/1, 15.248s (`/tmp/ares-pi-tip-red.log`). First strict difference remains
byte49575/line1902 (M73); movement-only diagnostic inspection locates the
actual support connection difference near Z9.36. No diagnostic filtering is
used by the acceptance comparator.

Three hypotheses: extension-expression semantics, contour trim bookkeeping,
then initial graph projection/order. An extension ternary precedence mismatch
was tested but left the complete output unchanged; that unrelated trial was
reverted. No extension change is included in this patch.

External gdb captures at the same offending contour show identical boundary,
three input lines, six endpoints and all initial arc lengths. Native snapshots
are `/tmp/ares-pi-tip-native-target.json`; Rust snapshots are
`/tmp/ares-pi-tip-rust-graphs.jsonl`. The first difference is after
`mark_boundary_segments_overlapping_infill`, before extension. A 92-scaled-unit
line is shorter than SCALED_EPSILON100. The shared collision helper preserves
the source's short-segment radius quirk and can return false. Upstream's
value-initialized `std::pair<double,double>` then remains `(0,0)` and the caller
still checks `len_out`; Rust incorrectly skipped that check on `None`, marking
both long contour arcs as entirely overlapped. Preserve the zero interval end
and evaluate the departure distance in both directions. Do not change the
collision helper or downstream extension/link phases.

Same public cached input, unchanged comparator after only `base_support/mark.rs`
changed: exit0, 1/1, 41.041s (`/tmp/ares-pi-tip-mark-green.log`, artifacts
`/tmp/ares-pi-tip-mark-green`). The existing live raw Grid/explicit Organic
assertions are the regression tests; no test body, fixture or comparator was
modified. Cached GREEN is not credited as a fresh native comparison.

Fresh full activation and quality receipts will be recorded below after
terminal completion. Native official AppImage SHA256
`d12fb8c8eac1aecd2dfb6377acd48f994f8fa439ed5292fa532dd82880f029fd`;
executable `64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.
Seed-only executable SHA256
`8f0cf69dd9494ccfe53bf6749d6b685ccb5a0dd0df206694ce50d421007cc03d`.
Source `8500fcdccaa10b5099ac20d252af3a7c560046f1`, with no FillBase.cpp diff;
reference paths/hashes recorded in `/tmp/ares-pi-tip-reference-sha256.txt`.

## Retained official first-run outlier

First fresh official suite: exit100, 4/5 functions, 27/28 complete pairs,
231.529s (`/tmp/ares-pi-tip-official.log`). Both thin-tip minimum assertions
PASS. The sole failure is Tree Slim style's model solid-infill order at
byte25982/line1032, not a support path; preserve `case-9eGkoF` and its exports.

This is not waived as timing noise. Re-slicing that exact saved 3MF with the
same official executable and CPU0 produces a different native stream matching
Ares (diagnostic `/tmp/ares-pi-tip-slim-official-repeat`). A clean archive of
base c3f61645 in `/tmp/ares-pi-tip-baseline`, built only in the diagnostic child
`/tmp/ares-pi-tip-target/baseline` of this worker's isolated target root,
reproduces precisely the same strict RED against the first reference (exit100,
10.355s, `/tmp/ares-pi-tip-slim-baseline.log`). Baseline and candidate Ares
complete bytes are identical, SHA256
`7e4571a048dd9d3e51e1b56a4508516b039241ddff038b55c2927a32bd104be1`.
This establishes a pre-existing official-reference instability, not a regression
from the mark fix. Do not splice a replacement pair into the first suite's
27/28 result. Run a new complete official suite and report both receipts.

## Feature-branch terminal gates

- Fresh official full repeat: exit0, 5/5 functions, **28/28** complete pairs,
  509.549s; one separate expected native-only explicit Organic100 rejection.
  `/tmp/ares-pi-tip-official-repeat.log` and matching artifact root.
- Fresh seed-only CPU0/seed0: exit0, 5/5 functions, **28/28** complete pairs,
  451.574s; one separate expected native-only rejection.
  `/tmp/ares-pi-tip-seeded.log` and matching artifact root.
- All case manifests in those two complete runs report PASS, with the same
  candidate test executable SHA256
  `35c78384630516eace5b26cdfa07f105c54200ce6fcc32dc5c4f08fb3879592b`.
  No failed pair was replaced inside a run. Runtime affinity was CPU0 for
  both Ares and each native process; compilation used the worker's other CPUs.
- Workspace Nextest: exit0, **7618/7618**, 71 slow, 9 existing skipped,
  625.975s test runtime (`/tmp/ares-pi-tip-workspace.log`). No ignores added.
- `cargo clippy`: exit0, pre-existing core253 warnings; no new mark.rs warning.
  `cargo fmt --all` and `--check`, `git diff --check`: exit0.
- `cargo check -p ares-core -p ares-wasm --target wasm32-unknown-unknown`:
  exit0 (compile-only). Quality logs: `/tmp/ares-pi-tip-{clippy,wasm,fmt}.log`;
  terminal receipt `/tmp/ares-pi-tip-quality-exits.txt` records all four zero.

Codex read-only review returned implementation `VERDICT: APPROVE`, without
attesting then-pending execution gates; parent verification above supplies
those terminal receipts. Review reports `/tmp/ares-pi-tip-codex-review*.md`
retain their tooling limitations (first attempted internal delegation failed;
single-reviewer retry stopped after trying to read not-yet-created gate logs).
These are not claimed as an unrestricted dual-review completion.

Production scope is only `fill/base_support/mark.rs`; no probe, guard, test
ignore, fixture change, comparison waiver or unrelated extension fix remains.
The changed Rust source remains below400 LOC. Main integration follows the user's
mandatory fetch/rebase + four-gate + fast-forward sequence; integration
receipts are reported externally on the actual resulting commit, without a
further documentation-only commit that would invalidate the verified tip.
