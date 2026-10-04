# Printer/Option matrix continuation (2026-10-04)

## Scope and ownership

Continue the source-owned validation boundaries in ARD-0096 and ARD-0101–0103,
`2026-10-04-stock-printer-sweep.md`, `stock-preset-parity.md` and
`support-option-reporting.md`. Stock discovery/export remains
`stock_presets/{sweep,discovery}.rs`; Option application/domain reporting remains
`option_coverage/{sweep,domains,activation}.rs`. These CLI adapters validate the
OrcaSlicer v2.4.2 rewrite; they are not a new slicing pipeline.

Keep stock presets unmodified. The existing Option sweep uses a modified
Ender-3 Classic baseline and feature-specific activation: its results are not
stock-printer coverage or the Cartesian product of all printers and options.
Retain complete saved projects, native/Ares output and every non-PASS result.
No comparator widening, new unsupported guards, silent skips or ignored tests.

Coordinator's vendor split: after its tip merge, pi-tip owns stock Snapmaker,
Qidi, re3D, Raise3D, Flashforge, Anycubic, Prusa and Creality. This branch
owns BBL stock residuals and all Option-sweep triage. BBL post-timelapse
duplicate-Z residuals are explicitly assigned to pi-tower. Do not repair the
listed other vendors here, including the Kobra extra-perimeter candidate.
Stock evidence is shared read-only with pi-tip: never overwrite, merge fresh
runs into, or modify retained case files; the active original producer may
finish appending its remaining cases and terminal summary.

Other implementers own wipe tower/timelapse/AdaptivePA/bridge direction
(pi-tower), thin-tip support fill connection (`fill/base_support`, pi-tip),
and lint cleanup (w25:p3). Record and report matching failures without changing
those owners, including failures reached through the Option sweep. Other
production fixes need a specific upstream function, Rust destination,
included/deferred behavior and sensitive RED before edits.

## Execution plan

1. Fetch/rebase latest origin/main, then freeze the baseline source identity.
2. Run the unfiltered stock inventory and Option sweep, sequentially, with both
   Ares and the seeded reference restricted to CPU0 (ARD-0102). Limit Nextest
   to one test and Cargo to two build jobs; use the independent target directory.
3. Group actual failures by first divergent source-owned stage, not merely the
   first visible M73/header difference. Preserve native/application/I/O errors
   separately. Reproduce each selected case with the official CPU0 reference.
4. Author the bounded source-cited repair spec/plan; TDD through the unchanged
   public complete-output seam, retaining finite previous GREEN controls.
5. Run workspace Nextest, Clippy, rustfmt and core/adapter WASM compilation.
   Commit and push the feature branch only after green gates. Print exactly
   `REVIEW-REQUEST <commit-hash> <one-sentence summary>` in the terminal and
   stop for coordinator Claude review. External Codex/OpenCode dual review
   and external review-completion executables are no longer required and
   must not be invoked. Do not start new work modifying this branch while
   awaiting review. Only after receiving matching `APPROVED <hash>`, fetch/
   rebase latest origin/main, repeat all four gates, then fast-forward with
   `git push origin HEAD:main`. A rejection requires another rebase and full
   verification, never force push. Do not modify the main worktree.

## Superseded baseline (partial diagnostic, not acceptance)

Fetch/rebase leaves `c3f6164530d6984aa1384c70ee47c7f7e3e553c6` unchanged.
Worktree `/tmp/ares-pi-arachne`, branch renamed `pi/printer-option-matrix`;
`CARGO_TARGET_DIR=/tmp/ares-pi-arachne-target`. The ignored `OrcaSlicer` directory
is a read-only symlink to the existing seed-replay 2.4.2 source package,
not to the main worktree.

Session25 initially ran stock then queued options. Evidence root:
`/tmp/ares-pi-matrix-c3f61645.4vmEqO`, with `stock.log`, `options.log`, separate
artifact subdirectories, `provenance.txt` and eventual `exits.txt`.
Reference is the existing
`/nix/store/jqfsg42f6xzq3krnf5scm6w4g1z4kz4c-orca-slicer-seed-replay-2.4.2/bin/orca-slicer`,
`ORCA_REPLAY_SEED=0`; executable hashes are retained in provenance.txt.
`ARES_PARITY_BUILD_REVISION` is a compile-time claim, not a replacement for
binary identity. No Option key/value filters are set. The second sweep has
not started at this record; no final denominator or completion is claimed.

First observed non-PASS cases: stock Anycubic Kobra 3 Max 0.6 and 0.8 nozzle
return the existing `extra_perimeters_on_overhangs` rejection. Candidate owner:
`PerimeterGenerator.cpp::apply_extra_perimeters` (1087–1113) and
`generate_extra_perimeters_over_overhangs` (885–1085). Current Rust rejection
is in `project_slice/perimeters/classic/preflight.rs::validate_record`; its
infill-boundary shell has no equivalent full consumer. These are retained
ARES_ERROR evidence, not a proven geometry fix or permission to remove the
guard alone. The helper also consumes `detect_bridging_direction`; coordinate
that dependency with pi-tower rather than changing its bridge-direction owner.
Anycubic Kobra 3 Max 0.4 now reports PASS in this run; older bounding-box RED
must not be relabeled as current evidence. Final triage awaits the full scans.

### Routed evidence: timelapse / pi-tower

At stock row163 the scan is still live. BBL H2S and P1P/P1S non-0.4 nozzle
cases show a missing duplicate `G1 Z` immediately after the first timelapse
block and initial object travel. Example retained H2S0.2 pair:
`stock/case-AHpvpi` under the evidence root, byte44919/line1090,
374423 expected versus374416 actual normalized bytes. Native emits two
`G1 Z.1` lines before unretract; Ares emits one. All three native retry
manifests report the same first difference. The adjacent timelapse block is
identical. This is routed to pi-tower's timelapse/motion-state investigation,
not a diagnosis proving the exact cause; no motion or timelapse edits here.
Other native export errors remain in stock.log and await final summary.

## Latest-main stock terminal receipt; Option still live

The frozen9a7b46fc stock phase has finished with exit100, recorded in
`/tmp/ares-pi-matrix-9a7b46fc.u9DXlq/exits.txt`. Its unchanged
`stock/stock-printer-summary.json` reports1001 outcome rows:823 PASS,
100 DIVERGENT,2 ARES_ERROR,66 ORCA_ERROR,10 VENDOR_INCOMPLETE;
`complete=false`. BBL accounts for48 rows:27 PASS and21 DIVERGENT.
These are9a7b46fc results, not acceptance of later main revisions. The stock
subdirectory is now retained read-only for pi-tip; all future replay outputs
must use a new external directory. Session35 has advanced to the Option
sweep; it remains live, with no final Option denominator or verdict yet.

## Latest-main refresh

Coordinator requested latest-main baseline after the test-cleanup merge.
Fetch/rebase moved this branch to `9a7b46fc`. Session25 was explicitly stopped
with SIGTERM after224 completed stock rows; no final stock summary or Option
sweep was produced. Its artifacts remain historical partial diagnostics.
Stopping also prevented the queued Option command from compiling new source
while carrying the old compile-time revision claim.

Replacement session35 uses `/tmp/ares-pi-matrix-9a7b46fc.u9DXlq` and first
builds `baseline.tar.zst` with `cargo nextest archive` at9a7b46fc. Both stock
and Option phases execute that same frozen archive, sequentially, CPU0 and
one test at a time. Archive/reference hashes and source identity are retained
in provenance.txt. Runtime profiles/fixtures still resolve through this
worktree; do not modify these inputs during the scan. This prevents a later
source rebase from silently recompiling the queued second sweep. No final
result is available yet. Before any production fix, fetch/rebase again and
reproduce its focused RED on the then-current main; these running results
must never be relabeled as a newer revision.

Coordinator announced the thin-tip merge; fetch/rebase advanced the feature
branch to `7ac55555` before implementation. Session35 continues to execute the
unchanged9a7b46fc archive, including its old thin-tip behavior; any related
Option RED must be replayed on7ac55555 or later before being treated as an
open defect. No `fill/base_support` edits are made here. All21 BBL stock
DIVERGENT rows have the duplicate-Z symptom assigned to pi-tower (H2S,
P1P/P1S/P2S, X1/X1 Carbon/X1E, nozzle0.2/0.6/0.8). This classifies the first
visible differences, not proof that no later differences exist.

The Option phase later terminated exit100 (3277.527s):421 PASS,
225 INCOMPLETE,4 FAIL of650 domains;1661 generated,1659 strictly compared,
1 rejected. FAIL owners: printer_technology/SLA application readback becomes
FFF; support_material_interface_fan_speed max/seeded lacks fan commands;
tree_support_branch_diameter_angle/min and tree_support_tip_diameter/min
have complete-output differences on the old archive. Timelapse_type/1 remains
an explicit rejected input, not successful coverage. Its source owner stays
with pi-tower. The two Tree cases must be replayed after the tip merge.

Fetch/rebase subsequently advanced this feature branch to56e4da53 (AdaptivePA
merge). The first selected repair is the support-interface fan request slice,
specified in `docs/architecture/support-interface-fan.md` and planned in
`2026-10-04-support-interface-fan.md`. No original stock/Option artifact is
changed or relabeled; fresh focused roots belong to the source snapshots
recorded in that plan.
