# Seam overhang: actual current layer thickness

This implements slice D of the
[parameter-scope spec](gcode-seam-parameter-scope-and-visibility-order.md),
`4bea1d247505c6ca3c2ec1afcd34c6ad4a339adb`. A–C are separate future slices.

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
assertion failed. This is type investigation, not RED.
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

| Probe | Domain | Result |
|---|---|---|
| v1 | .2/.2, .3/.2, .1/.2 heights; X shear0,.4,.8,1.2; nearest/aligned | 24/24 PASS |
| v2 | .3/.2,.1/.2; X shear±.5,±.6,±.7; nearest/aligned | 24/24 PASS |
| v3 | same height/slope/mode domain as v2, Y shear instead | 22 PASS,2 DIVERGENT |

Both v3 mismatches use .3/.2 and Y shear+.7 (nearest and aligned).
The nearest input was frozen as the committed fixture; equal-height siblings
in v1 passed.

Original standalone RED commit:
`4ab2e3de45bb43bbf1ea30c9ddbdbb71677abe9e`.
Test:
`runner::seam_layer_height::second_layer_overhang_seam_uses_current_layer_height`.
Fresh seeded RED exits nextest100.

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
128.669s. This preliminary run does not replace the required new-main
controls after an intervening main behavior change.

Main advanced to `3d65ec24e` (small-loop/scarf caller speed).
The new base is main plus the unchanged D
RED test, commit `c72ef9b1`. The only cherry-pick conflict was adjacent test
module registration in `runner.rs`; both small_loop_speed and
seam_layer_height modules were retained. No production conflict occurred.
The implementation branch rebased to the same main; spec is `ba03c1c10`,
RED is `07e6e9f5c`, GREEN is `189afcdfc493f9eba7f3fde39b101568630fe4d5`.
The branch encountered the same adjacent test-registration conflict; both
modules were retained. Base and candidate test/fixture content is identical.

The preliminary old-main GREEN narrow test passed (2.486s, nextest0).
Native raw SHA256 `48f64abc146d6ff78a40c0473dbe001be345fb8f87556a19136e2557f7bbea36`;
Ares raw SHA256 `972581dcccc430b3e91ff07bc139ef35f16733a3fdf218686fa059877d09ac52`.
These raw hashes include differing permitted generated-by text; the complete
normalized output is equal. New-main rechecks are required separately.

New-main directed GREEN passed (4.738s, nextest0). Ares raw SHA256 is the same as the
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
(35.65s, exit0).
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

| New-main D witness | Result |
|---|---|
| Seeded base | First difference byte2601,line124, same second-layer seam travel as original RED |
| Seeded candidate | Whole output PASS |
| Official base | Same seam RED |
| Official candidate | Whole output PASS |

Official witness raw Native SHA256:base
`4d7287d987c833426f456b01cdc1aee9bc59a3aa33572b7a90db0eab00a93813`,
candidate `02b168a3ec22415c56986c6e8d8e8aa071cef67bd44e7ce4815f0da0812fd8b1`.
Ares raw hashes match the original RED and GREEN respectively.
Official executable SHA256:
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.

The complete status/reason comparison covers all660 cases and lists
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
`Print.hpp:582` has uninitialized m_id, read by get_id at468–469. No garbage ID is ported into Ares.

| Case | First assertion difference base→candidate | Ares first differing line |
|---|---|---|
| `bridge_over_infill_filament::bridge_over_infill_multi_filament_x2d` | byte57060,line1096 (raw1097): Native id4575657223555907584→2147483648 | `; printing object ksr_fdmtest_v4.drc id:0 copy 0` |
| `narrowed_x2d_whole_file::narrowed_x2d_whole_file` | byte57075,line1097: Native id4564685423565588874→4575657221408423936 | Same Ares id0 line |

These facts come from parsing the actual full byte arrays retained in the
four test failure assertions, not selecting a nearby unrelated temp directory.
Bridge's asserted Ares SHA256 is identical on both sides:
`331827159281658d6baac34427836ee19fe89242c123fcb32ae3757ccd4ed79d`;
narrowed's asserted Ares SHA256 is likewise identical:
`ff469e046fbd6f49e361bed645f184e5daed08366a9ce02d0f48a0dc26574543`.
Both remain DIVERGENT/FAIL, not PASS.
