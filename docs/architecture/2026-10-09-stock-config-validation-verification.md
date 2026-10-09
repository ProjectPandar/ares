# #7 stock validation: current runtime conclusion

Status: REVIEW, no production change; spec ffe9db96d approved. Base8a0116f54.
Owners and per-stock exact error texts/line numbers remain in
[the approved spec](../superpowers/specs/2026-10-09-stock-config-validation-parity.md).

## Conclusion

**All28 admitted current stock selections have matching rejection payloads.**
No new validator discrepancy or runtime RED is established, so no production
validator port follows. Twenty loaded-config rows independently exit238
(Unix -18) with exact full stderr frame; eight width rows exit205 (-51) with
`Too small line width\n`. Public Core validation and validated slicing return
matching detailed payloads. Core is a byte API, not a process: its result is
compared to Native's phase-owned message, while Native exit status is asserted
separately. This does not modify CLI exit mapping or claim Core has a process
exit code.

All inputs in this round are explicitly
**RECONSTRUCTED_STOCK_SELECTION_NOT_ORIGINAL_PRESETS**, including width rows.
The unchanged stock select_printer resolver selects process and compatible
filament; no preset repairs, hand-selected filament, invented defaults or tuned
parameters. Catalog/selection/flattened presets/model/Native commands and full
captures are retained externally. The failed-Export historical presets are not
misrepresented as original byte replays.

For20 loaded errors, public validate_with_profiles and
slice_with_profiles_validated use the same catalog/selection/STL as Native's
Export request. For8 width errors, public validate_with_profiles checks the
selected config, and slice_project_plate_validated uses **the exact3MF bytes
exported for the subsequent Native Slice**. This keeps scene placement equal.
Every rejected input produces no successful Native or Ares G-code. This is
rejection-payload parity, not complete-output parity for successful slicing.
The historical raw ORCA_ERROR stock statuses remain ORCA_ERROR; no1001-row
sweep or valid-stream PASS is claimed.

## Exact run bindings and non-PASS history

- Test-only5847e1024: helper import compilation failure101 before Native;
  `/tmp/worker3-stock-validation-compile-error-v1.*` retained, not runtime RED.
- Test-onlyb12df8c677d95178d4e083e515d9d6b26d3b994b, run
  `1f1fff07-3bd1-46f9-bfd7-0c3470d4ec39`:20PASS/8FAIL, exit100.
  All28 validation-only payloads matched. Eight profile-slicing calls used
  Placement::None on the origin STL while Native arranged it; Core rejected
  scene eligibility first. This unequal-scene adapter failure is retained at
  `/tmp/worker3-stock-validation-current-v2*`, not a missing-validator RED.
- Adapter correction414c4d1c10525cdf74c55af602d333f0f3557669, run
  `30c73832-acb7-4b9d-bee8-806a8f5b70b4`:28PASS, exit0. Assertions remain
  exact; the width slicing adapter now supplies the identical exported scene.
  No production/fixture changes or altered expected error text.
- Rebase to8a0116f546be8930a6cc6275b443104803c1f4d3 produced
  `2996314c49e9be50f5b4dcf4564512da286d3376`. Fresh rebuild/current Native
  directed run `3716044a-0739-4a75-b405-4c4e0fb14f45`:28PASS, exit0.
  All28 retained receipts independently show both Core payloads equal; code
  counts20×238 and8×205. This is actual rebased execution, not carry-forward.

## Coordinator-defined docs/test-only gates

No seeded or official pair required because this slice changes no production.
No new WASM gate required. At exact2996314: workspace/all-target/all-feature
clippy `-D warnings` and fmt check both exit0.

Default workspace actually ran at exact414c4d1c1, with **all ARES_* removed**
(receipt records ARES_keys=[]),16threads/build8, success-output immediate:
run `8ffe8038-0064-46d9-b731-6a83acfadb71`, exit100,
**8176PASS / 1 known-main FAIL / 6 existing skips**. All28 new tests explicitly
print `SKIPPED: ARES_ORCA_BIN is unset` before accessing catalog/artifact paths;
none panic. Nextest records their early returns as PASS, but Native comparisons
are visibly SKIPPED and never counted as Native evidence.

The sole failure is worker6-owned
object_layer_height_ranges::runtime::optional_missing_native_never_consumes_frozen_range_reference,
artifact-root unwrap at object_layer_height_ranges_runtime.rs:7. Coordinator
confirmed this known-main failure introduced by bf0dd12b8, directed worker6's
hotfix, and authorized reporting the remaining workspace results normally.
This report does **not** claim global workspace GREEN or edit that adapter.
Coordinator later authorized workspace carry-forward across8a's PrintStatistics
production addition (new field, zero G-code byte change) while requiring fresh
rebuild/directed tests; the latter completed above. Default workspace is not
attributed to299 or8a. The conclusion is the only later change.

Evidence: `/tmp/worker3-stock-validation-{current-v3,rebased,rebased-record.json,
default-workspace,default-environment.json,clippy,fmt}`; artifact roots retain
full inputs, selection, receipts, command/status/stdout/stderr and SHA256s.
Seeded Native SHA256 remains
`59fb88a2cb441713c2f572cb06e30e4ab58c22a676cacca1812372b9575e73ff`.
No official-binary run claimed. Crashes/UB/abort/read/vendor failures, eligibility,
postprocess/templates, other option families and INPUT_REJECTED remain excluded.
Native products remain/tmp. Await sole-coordinator approval before push.
