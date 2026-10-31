# Seam overhang: actual current layer thickness

This implements slice D of the approved
[parameter-scope spec](gcode-seam-parameter-scope-and-visibility-order.md),
`4bea1d247505c6ca3c2ec1afcd34c6ad4a339adb`. A–C are separate future slices.
Status: complete and ready for exact-hash coordinator review; not pushed.

## Source boundary and implementation

Pinned upstream OrcaSlicer revision
`8500fcdccaa10b5099ac20d252af3a7c560046f1`:
`src/libslic3r/GCode/SeamPlacer.cpp:1058–1091`,
`SeamPlacer::calculate_overhangs_and_layer_embedding`, subtracts
`tan(overhang_angle_threshold) * po->layers()[layer_idx]->height`.
The threshold is `45.0f * float(PI) / 180.0f` (`SeamPlacer.hpp:122`).
`Layer.hpp:137` defines height as coordf_t; `libslic3r.h:46` defines that as
`double`. `Line.hpp:251–265` defines Linef on Vec2d;
`AABBTreeLines.hpp:300–304,348–351` therefore returns double distance.
The flow-width product is evaluated in float before addition to distance.

The Rust destination is only
`project_slice/seam_placement/alignment/{preparation,context}.rs`.
Preparation borrows the existing `PlannedPrintObject.layers`; context reads
`PlannedLayer.height` directly. It no longer subtracts adjacent candidate
midpoint Z values, which yield `(previous_height + current_height)/2`.
Candidate midpoint Z, distance geometry, visibility, candidate ordering,
comparison rules and layer planning remain unchanged. No new allocation,
fallback or adaptive-height feature is introduced.

The C++ compatibility header `<math.h>` exposes `using std::tan` after
including `<cmath>` (both included by `libslic3r.h:16,24`). On this libstdc++
header set, an external compile-only type check confirmed the unqualified
`tan(float)` return type is float, as is `std::tan(float)`; a double-return
assertion failed. Evidence: `/tmp/worker6-seam-D-tan-type.cpp`, its executable,
and `/tmp/worker6-seam-D-tan-headers.log`. This is type investigation, not RED.
Rust constructs the threshold in f32, evaluates tan on the widened input and
narrows once to f32, matching the existing seam transcendental strategy.
It promotes that rounded result for multiplication by the f64 actual height,
adds the existing distance and f32 width product in f64, then narrows the
result to f32 before the negative clamp. Existing f32 distance geometry is
retained; its broader precision parity is outside D.

## Reachability and actual Native RED

The witness is
`tests/parity/validation/seam-overhang-nonuniform-height.3mf`, SHA256
`699e9effa47a72c0de5292532aed8b1966813705f55fd0b4b22fdca661b04ee6`.
It derives from the existing Dumbbell fixture without editing that source:
set initial height0.3, regular height0.2 and nearest seam, then shear each
model-space vertex by `y += 0.7*z`. Other ZIP member contents are retained.
The E2E test verifies the stored options, pins fixture bytes and compares
the public slicing byte API to fresh real Native output. Only the generated-by
line is normalized; M73, wipes, ordering and all remaining bytes are compared.

Bounded exploration retained all attempted results:

| Probe | Domain | Result | Machine-readable record |
|---|---|---|---|
| v1 | .2/.2, .3/.2, .1/.2 heights; X shear0,.4,.8,1.2; nearest/aligned | 24/24 PASS | `/tmp/worker6-seam-D-probe-v1-record.json` |
| v2 | .3/.2,.1/.2; X shear±.5,±.6,±.7; nearest/aligned | 24/24 PASS | `/tmp/worker6-seam-D-probe-v2-record.json` |
| v3 | same height/slope/mode domain as v2, Y shear instead | 22 PASS,2 DIVERGENT | `/tmp/worker6-seam-D-probe-v3-record.json` |

Both v3 mismatches use .3/.2 and Y shear+.7 (nearest and aligned).
The nearest input was frozen as the committed fixture; equal-height siblings
in v1 passed. Probe sources, logs, fresh caches and full artifacts remain
under the corresponding `/tmp/worker6-seam-D-probe*` paths.

Original standalone RED commit:
`4ab2e3de45bb43bbf1ea30c9ddbdbb71677abe9e`.
Test:
`runner::seam_layer_height::second_layer_overhang_seam_uses_current_layer_height`.
Fresh seeded RED exits nextest100 and retains
`/tmp/worker6-seam-D-red-artifacts/case-sof17X`;
log `/tmp/worker6-seam-D-red.log`, evidence record
`/tmp/worker6-seam-D-red-record.json`.

The first normalized difference is byte2601, raw line124,column9:

```gcode
; Native
G1 X107.803 Y110.594 Z.9
; Ares
G1 X107.957 Y110.47 Z.9
```

Raw line106 is `;LAYER_CHANGE`,107 is `;Z:0.5`,108 is `;HEIGHT:0.2`.
The first layer and all earlier commands match; this is the second-layer
seam-entry travel, not a first-layer or M73 difference. Consecutive actual
heights are .3 then .2, whereas the old midpoint difference is .25.
Expected normalized output25710 bytes, actual25772 bytes.
Native raw SHA256:
`76d63c9e707cf9f70649757a9e126b18ba6f17ee31304498a01e682f343808d6`;
Ares raw SHA256:
`ab15ccbfbc29b3ca8939c9af02314a5530d2d20966282bd93890b23ae4b9415b`.

Fresh-cache Native producer receipts bind input/output and executable
identity, separately from generic comparison manifests that say `unknown`.
Seeded Native executable SHA256:
`59fb88a2cb441713c2f572cb06e30e4ab58c22a676cacca1812372b9575e73ff`;
resources SHA256:
`286fab66becc4c87ebb72dc8e66ec9d515e7f440d530bacebeb1f51407bb8a69`.

## Baseline advancement and remaining gates

An initial old-main seeded base completed:651 tests,650 PASS,1 FAIL (D RED),
128.669s. This preliminary run is retained under
`/tmp/worker6-seam-D-base-seeded*`; it does not replace the required new-main
controls after worker1's behavior change.

Coordinator advanced main to `3d65ec24e` (small-loop/scarf caller speed).
New base worktree `/tmp/ares-worker6-seam-D-base` is main plus the unchanged D
RED test, commit `c72ef9b1`. The only cherry-pick conflict was adjacent test
module registration in `runner.rs`; both small_loop_speed and
seam_layer_height modules were retained. No production conflict occurred.
The implementation branch rebased to the same main; spec is `ba03c1c10`,
RED is `07e6e9f5c`, GREEN is `189afcdfc493f9eba7f3fde39b101568630fe4d5`.
The branch encountered the same adjacent test-registration conflict; both
modules were retained. Base and candidate test/fixture content is identical.

The preliminary old-main GREEN narrow test passed (2.486s, nextest0),
artifacts `/tmp/worker6-seam-D-green-artifacts/case-FRJXNp`.
Native raw SHA256 `48f64abc146d6ff78a40c0473dbe001be345fb8f87556a19136e2557f7bbea36`;
Ares raw SHA256 `972581dcccc430b3e91ff07bc139ef35f16733a3fdf218686fa059877d09ac52`.
These raw hashes include differing permitted generated-by text; the complete
normalized output is equal. New-main rechecks are required separately.

New-main directed GREEN passed (4.738s, nextest0), artifacts
`/tmp/worker6-seam-D-main-green-artifacts/case-oYa4sL`, log
`/tmp/worker6-seam-D-main-green.log`. Ares raw SHA256 is the same as the
preliminary GREEN; Native raw SHA256 is
`5b654d256d73076670e0678ed850176adcfa5366010de62f203fbc5ff17be7b5`.
Complete normalized output is equal, including M73 and all seam/wipe lines.

New-main seeded base completed:660 tests,659 PASS,1 FAIL (only D RED),
145.587s, exit100. Base executable SHA256:
`cee95270fd16419a9e1f4093bfdea14ca7f4154b594b538607d4f4e5da150dc9`;
candidate executable SHA256:
`c072d094005ce6aad8d3677f03d4862b91e16366f4039519a05d1720c0edd31e`.
Passed: `cargo fmt --check`;
`cargo clippy --workspace --all-targets --all-features -- -D warnings`
(1m38s, exit0); `cargo check -p ares-wasm --target wasm32-unknown-unknown`
(35.65s, exit0). Logs: `/tmp/worker6-seam-D-{fmt,clippy,wasm}.log`.
New-main seeded candidate completed:660 tests,660 PASS,183.286s, exit0.
Its failure set is a strict subset of base; the only status change is D's
`runner::seam_layer_height::second_layer_overhang_seam_uses_current_layer_height`,
DIVERGENT/FAIL→PASS. No other seeded status changed.
New-main official completed:base649 PASS/11 FAIL (140.477s), candidate650
PASS/10 FAIL (108.800s),660 tests each, nextest100 on both because shared
baseline failures remain. No Native timeout occurred. Candidate failures are
a strict subset of base; the only status **and failure-reason** change is
DIVERGENT/FAIL→PASS for D. `compatible_regions` is PASS/PASS this round.
Workspace nextest completed:8050 PASS (14 slow),9 skipped,556.938s,
exit0. Compilation passed in3m55s. No workspace FAIL/TIMEOUT occurred.

| New-main D witness | Retained case | Result |
|---|---|---|
| Seeded base | `/tmp/worker6-seam-D-main-base-seeded-artifacts/case-r6t7eo` | First difference byte2601,line124, same second-layer seam travel as original RED |
| Seeded candidate | `/tmp/worker6-seam-D-main-candidate-seeded-artifacts/case-EYhMmx` | Whole output PASS |
| Official base | `/tmp/worker6-seam-D-main-base-official-artifacts/case-Owz7EO` | Same seam RED |
| Official candidate | `/tmp/worker6-seam-D-main-candidate-official-artifacts/case-BEE31p` | Whole output PASS |

Official witness raw Native SHA256:base
`4d7287d987c833426f456b01cdc1aee9bc59a3aa33572b7a90db0eab00a93813`,
candidate `02b168a3ec22415c56986c6e8d8e8aa071cef67bd44e7ce4815f0da0812fd8b1`.
Ares raw hashes match the original RED and GREEN respectively.
Official executable SHA256:
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.
The four fresh witness receipts are indexed with full producer identities at
`/tmp/worker6-seam-D-main-producer-receipts.json`; seeded witness manifests
are also indexed in `/tmp/worker6-seam-D-main-witness-record.json`.

Control logs/cache/artifact roots use
`/tmp/worker6-seam-D-main-{base,candidate}-{seeded,official}` followed by
`.log`, `-cache` or `-artifacts`. The machine-readable complete status/reason
comparison is `/tmp/worker6-seam-D-control-status-changes.json`; its extraction
script is `/tmp/worker6-seam-D-compare-controls.py`. It covers all660 cases and lists
both failure sets and the sole status/reason change, D.

## Every shared official failure

Eight failures remain missing initialized reference environment (NotPresent),
with the same status and reason on both sides. They remain FAIL:

- `option_coverage::bookkeeping_export::bookkeeping_matches_complete_configured_filament_output`
- `option_coverage::initialized_object_id::initialized_reference_repeats_complete_ksr_output`
- `option_coverage::initialized_short_line::initialized_short_line_matches_complete_internal_fill_output`
- `option_coverage::initialized_stitch_junction::initialized_stitch_junction_matches_complete_internal_fill`
- `option_coverage::initialized_transition_snap::initialized_transition_snap_matches_complete_internal_fill`
- `option_coverage::single_volume_offset::single_volume_offset_matches_complete_rotated_prism_output`
- `runner::initialized_loop_chain::initialized_loop_chain_matches_complete_output`
- `runner::initialized_overhang_intersections::initialized_overhang_intersections_match_complete_output`

The other two are known official Native object-id UB, with the same fixture
SHA256 `22d3813ddbbfbc40bd3955fb35d5f6d19b7d8ee23472e7aa2e354e50a23cfe82`
and the boundary established by the accepted
[X2D classification](2026-10-08-x2d-official-first-difference-classification.md):
`Print.hpp:582` has uninitialized m_id, read by get_id at468–469. This is not
O8-X2D-SCALAR and no garbage ID is ported into Ares.

| Case | First assertion difference base→candidate | Ares first differing line |
|---|---|---|
| `bridge_over_infill_filament::bridge_over_infill_multi_filament_x2d` | byte57060,line1096 (raw1097): Native id4575657223555907584→2147483648 | `; printing object ksr_fdmtest_v4.drc id:0 copy 0` |
| `narrowed_x2d_whole_file::narrowed_x2d_whole_file` | byte57075,line1097: Native id4564685423565588874→4575657221408423936 | Same Ares id0 line |

These facts come from parsing the actual full byte arrays retained in the
four test failure assertions, not selecting a nearby unrelated temp directory.
Evidence: `/tmp/worker6-seam-D-official-x2d-assertion-record.json`.
Bridge's asserted Ares SHA256 is identical on both sides:
`331827159281658d6baac34427836ee19fe89242c123fcb32ae3757ccd4ed79d`;
narrowed's asserted Ares SHA256 is likewise identical:
`ff469e046fbd6f49e361bed645f184e5daed08366a9ce02d0f48a0dc26574543`.
Both remain DIVERGENT/FAIL, not PASS.

## Resource policy and final gate

The official wrapper fix at16:03 Asia/Singapore (08:03UTC) changed CPU1
forcing into inheritance of the pinned runner CPU. All D official runs use
the fixed wrapper, SHA256
`391081d7f971e703efb6a72fe9ad7a7c406a11e220d8bf4bc9474e01437a9333`.
Control script SHA256
`a8ee8f379012d374eb05e63bf786c34957ad137501cde6a32c6de53fcd0e2fb6`.
Each native producer remains single-CPU; no old timeout is counted as PASS.

Ordinary workspace execution uses
`CARGO_TARGET_DIR=/tmp/ares-worker6-green-target`, `CARGO_BUILD_JOBS=8`,
`CARGO_INCREMENTAL=0`, and
`cargo nextest run --workspace --test-threads 16 --no-fail-fast`.
Log: `/tmp/worker6-seam-D-workspace.log`. Its optional-reference tests returning
without Native environment are not substituted for the separately recorded
fresh Native control comparisons. The official pair completed before the
coordinator's new BROWSER-START; already-running workspace continued, and
no new heavy task was started during that browser hold.

All paired control rounds use independent base/candidate target directories,
fresh cache roots and16 nextest threads, one heavy task at a time. Native
timeout is INCONCLUSIVE; Native nondeterminism is recorded separately and
never promoted to PASS. All status/reason changes are enumerated above. Every required gate is
complete; the shared official failures remain non-PASS. No comparator or
assertion was relaxed and no retry replaced any original result.

After the coordinator's docs-only main advancements, the branch rebased onto
`d9b642191` with zero conflicts. Final spec/RED/GREEN commits are respectively
`518d771a89019fce6440478c620452f20e804137`, `89c4154117b314736e0e5406c4e6c74531d53843`, `5b54fd9caa1b59d1cd7cb123871b01342895a772`.
`git diff --exit-code 189afcdfc493f9eba7f3fde39b101568630fe4d5 HEAD -- crates tests scripts Cargo.toml Cargo.lock .config`
returned0 with empty output, confirming runtime/test/fixture content equals
the gated candidate. Evidence is
`/tmp/worker6-seam-D-post-rebase-runtime-diff.log`. Rust evidence is reused
under the coordinator's explicit docs-only ruling. This document is the
review report; push still waits for exact-hash coordinator approval.
