# Timelapse state / Type2 implementation plan

1. Establish the BBL stock complete-output RED using untouched vendor presets.
   Inspect the upstream owner and repair only the incorrect state transition.
2. Extend the stock regression to the handed-off printer/nozzle families and
   adjacent controls. Obtain original and separate seeded full-output GREEN.
3. Review the historical Type2 source slice against current preparation,
   footprint, layer and tool interfaces. Port source-owned behavior via TDD;
   do not replay historical guards, region-zero assumptions or lint suppressions.
4. Run workspace Nextest, Clippy (zero new warnings), fmt and WASM. All Cargo
   commands use `CARGO_TARGET_DIR=/tmp/ares-pi-tower-target`.
5. Commit/push each independently green slice, print `REVIEW-REQUEST <hash>
   <summary>`, and stop for coordinator review. After `APPROVED`, fetch/rebase
   current main, rerun gates and ff-push main. Repeat on advancement; no force.

## Provenance and execution

Adaptive PA was approved at `91cae8cf`; rebase onto `0d6dfe1f` preserved all
three patches (`git range-diff` equality). Workspace 7592 PASS, both live
references PASS, Clippy 249 baseline warnings, fmt and WASM passed again.
Fast-forward integration: `0d6dfe1f -> 56e4da53`.
Evidence: `/tmp/ares-pi-tower-pa-integration-*.log` and
`/tmp/pi-unified-exec-69-55e90c70.log`.

Coordinator's BBL handoff: expected two `G1 Z.1` lines before unretraction,
Ares emitted one after the first timelapse / object travel. The retained path
was `/tmp/ares-pi-matrix-9a7b46fc.u9DXlq/stock/case-AHpvpi`, byte 44919 / line
1090. That directory is no longer available in this worktree environment;
generate a fresh original-reference pair rather than claiming its inspection.

The first tracer uses stock `Bambu Lab P1P 0.2 nozzle` with `cube10.stl`, no
smoke overrides, no geometry rewrites and no disabled timelapse dependency.
## Fresh RED and source diagnosis

Original CPU0 run `b3c18bd2-66ae-4e4c-9324-2bf7799c59e4` confirms the
handoff: all three completed oracle attempts are divergent. The inspected
final pair differs only by one missing `G1 Z.1` (7 bytes):
`/tmp/ares-pi-tower-bbl-z-red-official/case-pQ3q43`; log
`/tmp/ares-pi-tower-bbl-z-red-official.log`, first difference line 966.

An additional full-output fixture enables the upstream `gcode_comments`
option without altering the stock baseline. Its original comments identify
**two distinct source operations**: travel's `move to first perimeter point`
Z line followed by `_extrude`'s `ensure Z matches planned layer height`.
Ares emitted only the latter and dropped the travel XY comment. Evidence:
`/tmp/ares-pi-tower-bbl-z-comments-red.log`, run
`ef3f0fcd-c365-4437-b4d1-fcf661f0893a`, pair `case-ILF3QY`.

The quantization hypothesis is disproven: upstream's post-contour reset uses
`quantize_xyzf` too (`GCode.cpp:6396-6400`). The owning difference is
`GCode::travel_to` passing `m_need_change_layer_lift_z` as `force_z`
(`GCode.cpp:7480-7482`) into `GCodeWriter::travel_to_xyz`: its lifted dead-band
shortcut requires **both** `!force_z` and `!will_move_z` (line 771). The Rust
lift-cancellation predicate tested only `will_move_z`, missing forced travel
after an unclear-position timelapse. Preserve the force flag independently
of the approach-shape flag; let the existing XY/Z split and first-extrusion
synchronization emit their own commands. Do not add a literal duplicate Z.

## GREEN prerequisite slice and gates

The source predicate repair passes both P1P 0.2 cases (stock + source comments):
`/tmp/ares-pi-tower-bbl-z-force-fix.log`, run
`59cf49a5-100b-4bc0-9683-c813e5859e75`, 6.868s. Extend the unchanged-stock test
to H2S, P1P and P1S at 0.2/0.4/0.6/0.8 mm. All 12 presets pass, including
0.4-mm controls, as does the additional comment-enabled P1P 0.2 case.

| Gate | Result | Log |
| --- | --- | --- |
| Original CPU0 | 5 tests / 29 complete-output cases PASS, 33.752s | `/tmp/ares-pi-tower-bbl-z-matrix-official.log` |
| Seeded, seed 0 / CPU0 | 5 tests / 29 complete-output cases PASS, 40.550s | `/tmp/ares-pi-tower-bbl-z-final-seeded.log` |
| Workspace Nextest | 7594 PASS, 9 existing skips, 373.293s | `/tmp/ares-pi-tower-bbl-z-final-workspace.log` |
| Clippy | exit 0, same 249 baseline warnings, no added warnings | `/tmp/ares-pi-tower-bbl-z-final-clippy.log` |
| rustfmt | exit 0 | `/tmp/ares-pi-tower-bbl-z-final-fmt.log` |
| Core + adapter WASM build | exit 0, 16.14s | `/tmp/ares-pi-tower-bbl-z-final-wasm.log` |

The 29 cases consist of 13 BBL cases and 16 Adaptive PA regressions, not 29
new printer presets. Original run: `e239d0a8-bb20-4230-b773-d0cbf1eae4a9`;
seeded run: `9eae1e45-62ef-4927-883e-4cde2651bf99`. Separate artifact roots:
`/tmp/ares-pi-tower-bbl-z-matrix-official/` and
`/tmp/ares-pi-tower-bbl-z-final-seeded/`. Gate chain session 78 exits 0.
Only documentation changes after these code checks to record their results.

This independently green prerequisite is ready for feature-branch push and
coordinator review. Stop for matching `APPROVED` before rebasing/revalidating
and integrating. **Type2 tower production code is still untouched**; do not
claim completion of that dependent port or start it on this branch while
awaiting review.
