# Plan: PrintObject-centered fill reference

Source contract: [spec](../specs/2026-10-04-print-object-fill-reference.md),
[ARD](../../architecture/ard-print-object-fill-reference.md).

1. Establish RED at the existing public project/G-code seam on7ac55555.
   Focused A250 BKit0.4 already0/1; replay full68 frozen Snapmaker/Qidi residuals
   with unchanged references; save inventory and results outside the repository.
2. Replace the slice-contour center call only in `generate_layer` with the
   zero center of upstream `PrintObject::bounding_box`. Do not alter the retained
   bridge sparse-anchoring caller or any comparator/test expectations.
3. Repeat exact focused and68-case replay, plus previously PASS stock controls.
   Verify representative official2.4.2 CPU0 re-slices. Record remaining independent
   causes instead of bundling new work or lowering acceptance standards.
4. Run `cargo nextest run --workspace`, Clippy, fmt and WASM compile for core and
   adapter with isolated target directory. Inspect diff for unintended files,
   probes and stale review claims. Commit/push only when the bounded slice is green.
5. Print `REVIEW-REQUEST <hash> <summary>` and stop for coordinator Claude approval.
   After APPROVED fetch/rebase latest main, rerun all four gates and fast-forward
   push HEAD:main; repeat rebase/gates on race, never force push.

Commands use `CARGO_TARGET_DIR=/tmp/ares-pi-matrix-vendors-target`,
`CARGO_BUILD_JOBS=2`. Replay uses `ARES_PARITY_REPLAY` and
`ARES_PARITY_ARTIFACT_ROOT` (an early diagnostic invocation used incorrect names
and offline-skipped; `/tmp/ares-vendors-snapmaker-red.log` is NOT evidence).
The valid focused RED log is `/tmp/ares-vendors-snapmaker-red-actual.log`.
No test implementation is added after production code; the pre-existing strict
E2E replay test and frozen stock fixtures are the RED→GREEN regression contract.

## Coordinator-approved bounded slice and residual handoff

The coordinator explicitly approved the52-case independent fill-reference slice
and requires rebase onto latest origin/main (at least56e4da53) BEFORE commit/review.
The full68-case run remains52PASS/16DIVERGENT, never reported as all-green.
Frozen references/comparator and all failing assertions are unchanged.
823/823 pre-existing PASS stock controls stay green.
Official CPU0 representatives are6/8: the two0.2 residuals remain visible.

### pi-tower:14 Snapmaker Internal Bridge residuals

Each remaining Snapmaker case has just one non-generator differing G-code line,
under `;TYPE:Internal Bridge` at Z9.16. This is a role-localized symptom report,
not proof of the exact numerical owner; do not patch cooling or rounding blindly.
Original immutable case paths and candidate first differences:

- **stock/Snapmaker/Snapmaker A250 BKit (0.2 nozzle)** — frozen `/tmp/ares-pi-matrix-9a7b46fc.u9DXlq/stock/case-RfG1wX`; candidate `/tmp/ares-vendors-fill-green/case-UKwkcQ`.
  first difference at byte 126262 (line 5936, column 28; expected 173518 bytes, actual 173518 bytes)
    line 5936: expected "G1 X109.351 Y125.169 E.11807"; actual "G1 X109.351 Y125.169 E.11808"
- **stock/Snapmaker/Snapmaker A250 Dual (0.2 nozzle)** — frozen `/tmp/ares-pi-matrix-9a7b46fc.u9DXlq/stock/case-g14Xmz`; candidate `/tmp/ares-vendors-fill-green/case-dBtzvs`.
  first difference at byte 126816 (line 5940, column 28; expected 181188 bytes, actual 181189 bytes)
    line 5940: expected "G1 X110.765 Y126.583 E.0858"; actual "G1 X110.765 Y126.583 E.08581"
- **stock/Snapmaker/Snapmaker A250 Dual BKit (0.2 nozzle)** — frozen `/tmp/ares-pi-matrix-9a7b46fc.u9DXlq/stock/case-5SBGrh`; candidate `/tmp/ares-vendors-fill-green/case-Yl4yav`.
  first difference at byte 126820 (line 5940, column 28; expected 181252 bytes, actual 181253 bytes)
    line 5940: expected "G1 X110.765 Y126.583 E.0858"; actual "G1 X110.765 Y126.583 E.08581"
- **stock/Snapmaker/Snapmaker A250 Dual QS+B Kit (0.2 nozzle)** — frozen `/tmp/ares-pi-matrix-9a7b46fc.u9DXlq/stock/case-rVC2zy`; candidate `/tmp/ares-vendors-fill-green/case-QRzOOs`.
  first difference at byte 126824 (line 5940, column 28; expected 181312 bytes, actual 181313 bytes)
    line 5940: expected "G1 X110.765 Y126.583 E.0858"; actual "G1 X110.765 Y126.583 E.08581"
- **stock/Snapmaker/Snapmaker A250 Dual QSKit (0.2 nozzle)** — frozen `/tmp/ares-pi-matrix-9a7b46fc.u9DXlq/stock/case-vKr6n5`; candidate `/tmp/ares-vendors-fill-green/case-02g542`.
  first difference at byte 126822 (line 5940, column 28; expected 181254 bytes, actual 181255 bytes)
    line 5940: expected "G1 X110.765 Y126.583 E.0858"; actual "G1 X110.765 Y126.583 E.08581"
- **stock/Snapmaker/Snapmaker A250 QS+B Kit (0.2 nozzle)** — frozen `/tmp/ares-pi-matrix-9a7b46fc.u9DXlq/stock/case-iIv6gr`; candidate `/tmp/ares-vendors-fill-green/case-Am8f9y`.
  first difference at byte 126266 (line 5936, column 28; expected 173573 bytes, actual 173573 bytes)
    line 5936: expected "G1 X109.351 Y125.169 E.11807"; actual "G1 X109.351 Y125.169 E.11808"
- **stock/Snapmaker/Snapmaker A250 QSKit (0.2 nozzle)** — frozen `/tmp/ares-pi-matrix-9a7b46fc.u9DXlq/stock/case-FrjLnh`; candidate `/tmp/ares-vendors-fill-green/case-tLju5l`.
  first difference at byte 126240 (line 5934, column 28; expected 173499 bytes, actual 173499 bytes)
    line 5934: expected "G1 X109.351 Y125.169 E.11807"; actual "G1 X109.351 Y125.169 E.11808"
- **stock/Snapmaker/Snapmaker A350 BKit (0.2 nozzle)** — frozen `/tmp/ares-pi-matrix-9a7b46fc.u9DXlq/stock/case-WOebkp`; candidate `/tmp/ares-vendors-fill-green/case-aO5tMr`.
  first difference at byte 126233 (line 5933, column 28; expected 173479 bytes, actual 173479 bytes)
    line 5933: expected "G1 X154.351 Y175.169 E.11807"; actual "G1 X154.351 Y175.169 E.11808"
- **stock/Snapmaker/Snapmaker A350 Dual (0.2 nozzle)** — frozen `/tmp/ares-pi-matrix-9a7b46fc.u9DXlq/stock/case-ivfaze`; candidate `/tmp/ares-vendors-fill-green/case-gUiKvT`.
  first difference at byte 126820 (line 5940, column 28; expected 181173 bytes, actual 181174 bytes)
    line 5940: expected "G1 X155.765 Y176.583 E.0858"; actual "G1 X155.765 Y176.583 E.08581"
- **stock/Snapmaker/Snapmaker A350 Dual BKit (0.2 nozzle)** — frozen `/tmp/ares-pi-matrix-9a7b46fc.u9DXlq/stock/case-A0epFE`; candidate `/tmp/ares-vendors-fill-green/case-eucyCr`.
  first difference at byte 126803 (line 5938, column 28; expected 181216 bytes, actual 181217 bytes)
    line 5938: expected "G1 X155.765 Y176.583 E.0858"; actual "G1 X155.765 Y176.583 E.08581"
- **stock/Snapmaker/Snapmaker A350 Dual QS+B Kit (0.2 nozzle)** — frozen `/tmp/ares-pi-matrix-9a7b46fc.u9DXlq/stock/case-Ui6vXS`; candidate `/tmp/ares-vendors-fill-green/case-tHojta`.
  first difference at byte 126807 (line 5938, column 28; expected 181276 bytes, actual 181277 bytes)
    line 5938: expected "G1 X155.765 Y176.583 E.0858"; actual "G1 X155.765 Y176.583 E.08581"
- **stock/Snapmaker/Snapmaker A350 Dual QSKit (0.2 nozzle)** — frozen `/tmp/ares-pi-matrix-9a7b46fc.u9DXlq/stock/case-QBLpA2`; candidate `/tmp/ares-vendors-fill-green/case-DYTxVq`.
  first difference at byte 126826 (line 5940, column 28; expected 181239 bytes, actual 181240 bytes)
    line 5940: expected "G1 X155.765 Y176.583 E.0858"; actual "G1 X155.765 Y176.583 E.08581"
- **stock/Snapmaker/Snapmaker A350 QS+B Kit (0.2 nozzle)** — frozen `/tmp/ares-pi-matrix-9a7b46fc.u9DXlq/stock/case-dn6Fw5`; candidate `/tmp/ares-vendors-fill-green/case-LPpVTQ`.
  first difference at byte 126237 (line 5933, column 28; expected 173534 bytes, actual 173534 bytes)
    line 5933: expected "G1 X154.351 Y175.169 E.11807"; actual "G1 X154.351 Y175.169 E.11808"
- **stock/Snapmaker/Snapmaker A350 QSKit (0.2 nozzle)** — frozen `/tmp/ares-pi-matrix-9a7b46fc.u9DXlq/stock/case-oXBz2q`; candidate `/tmp/ares-vendors-fill-green/case-81Uurf`.
  first difference at byte 126223 (line 5932, column 28; expected 173472 bytes, actual 173472 bytes)
    line 5932: expected "G1 X154.351 Y175.169 E.11807"; actual "G1 X154.351 Y175.169 E.11808"

### Retained for this worker:2 Qidi0.2 cases

- **stock/Qidi/Qidi Q1 Pro 0.2 nozzle** — frozen `/tmp/ares-pi-matrix-9a7b46fc.u9DXlq/stock/case-WE5l76`; candidate `/tmp/ares-vendors-fill-green/case-DZf9vs`.
  first difference at byte 14856 (line 590, column 1; expected 274464 bytes, actual 274512 bytes)
    line 590: expected "M73 P5 R13"; actual "G1 X122.072 Y127.907 E.06764"
    line 591: expected "G1 X122.072 Y127.907 E.06764"; actual "G1 X121.93 Y127.765 E.00165"
- **stock/Qidi/Qidi X-Max 4 0.2 nozzle** — frozen `/tmp/ares-pi-matrix-9a7b46fc.u9DXlq/stock/case-b8PGl3`; candidate `/tmp/ares-vendors-fill-green/case-TAfI7N`.
  first difference at byte 117325 (line 4720, column 1; expected 279213 bytes, actual 279263 bytes)
    line 4720: expected "M73 P73 R4"; actual "G1 X197.917 Y192.083 E.02141"
    line 4721: expected "G1 X197.917 Y192.083 E.02141"; actual "M73 P73 R4"

Qidi residuals include late-layer grid geometry/travel/cooling changes and M73
placement; exact cause remains unproven. They are explicitly next-slice work,
not claimed resolved or folded into the bridge handoff.

Full machine-readable handoff `/tmp/ares-vendors-fill-residual16.json`; approved
52-input inventory `/tmp/ares-vendors-fill-reference-52/inventory.json`.

## Verified receipt before coordinator review

Rebased cleanly onto `56e4da53ecae0e51d728457978b63a4b7b1466e8` before commit.
The coordinator subsequently announced main `b1a5a84d`; integrate only after
approval plus another latest-main rebase/revalidation (never force push).
Production delta is one reference selection and its comment; no comparator,
test, rounding, bridge, motion, support or vendor-preset code changed.

| Evidence | Result | Artifact |
| --- | --- | --- |
| Original main7ac55555 full68 RED | 0/68 PASS | `/tmp/ares-vendors-fill-red/replay-summary.json` |
| Approved52 on rebased main | 52/52 PASS | `/tmp/ares-vendors-rebased-reference52/replay-summary.json` |
| Full68 on rebased main | 52 PASS /16 DIVERGENT | `/tmp/ares-vendors-rebased-full68/replay-summary.json` |
| All frozen prior-PASS stock controls | 823/823 PASS | `/tmp/ares-vendors-rebased-controls823/replay-summary.json` |
| Fresh official CPU0 representatives | 6 PASS /2 DIVERGENT | `/tmp/ares-vendors-rebased-official8/replay-summary.json` |
| `cargo nextest run --workspace` | 7592/7592 PASS,9 existing skips | `/tmp/ares-vendors-rebased-workspace.log` |
| `cargo fmt --all -- --check` | exit0 | `/tmp/ares-vendors-rebased-fmt.log` |
| `cargo clippy` | exit0,249 existing core-library warnings | `/tmp/ares-vendors-rebased-clippy.log` |
| `cargo build --target wasm32-unknown-unknown -p ares-core -p ares-wasm` | exit0 | `/tmp/ares-vendors-rebased-wasm.log` |

All68 statuses and first-difference details are identical before/after the
56e4da53 rebase. The official8 references independently match their frozen
counterparts after generator-only normalization. The strict full68 and official8
runs still exit100; these failures are preserved, not converted to successes.
Only the coordinator-approved52 inventory is claimed as this slice's green gate.
The52 labels all occur in the original68 RED inventory; original evidence remains
read-only. Receipts: `/tmp/ares-vendors-rebased-exits.txt`.

### Lint-owner handoff

An extra, broader pre-rebase `cargo clippy --workspace --all-targets -- -D warnings`
failed on existing lint diagnostics, including269 lib-test errors (log
`/tmp/ares-vendors-clippy.log`). The required default `cargo clippy` exits0 on the
rebased candidate but reports249 core-library warnings. This is NOT a claim of
warning-free Clippy or of a passing all-targets/-Dwarnings gate. Reported to the
coordinator for w25:p3; no unrelated lint fixes or suppressions were introduced.

