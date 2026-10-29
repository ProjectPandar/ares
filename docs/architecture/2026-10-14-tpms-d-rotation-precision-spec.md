# TPMS-D corrected-angle float boundary

Status: SPEC REVIEW REQUEST; implementation and E2E RED require approval.
Baseline: `29f4a49c18139925fec238d31d2fdb6b790fbc30` (stack-9).
Upstream: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Owner: worker6. This is non-ironing fill; SupportCubic Slice B remains worker3's, and support-ironing plane paths/CrossHatch remain worker5's.

## Owning upstream boundary

`src/libslic3r/Fill/FillTpmsD.cpp:101-110`, `FillTpmsD::_fill_surface_single`, computes:

```cpp
auto infill_angle = float(this->angle + (CorrectionAngle * 2*M_PI) / 360.);
if (std::abs(infill_angle) >= EPSILON)
    expolygon.rotate(-infill_angle);
```

`FillBase.hpp:124` stores `Fill::angle` as float. `FillTpmsD.hpp:24-26` defines the exactly representable float correction `-45`; multiplication by `M_PI` promotes the correction expression to double. The addition to the float base angle therefore occurs in double, **then narrows once to float**. The float is used for the rotation predicate and widened when passed to the polygon rotation. `FillTpmsD.cpp:155-160` uses that same corrected float to rotate connected paths back.

The intervening operations are already represented in Ares: rotated bounds/density/grid origin (`:112-127`), translation/multiline/clipping (`:129-136`), minimum-length filtering (`:138-144`), and connection (`:147-153`). They are included only as unchanged consumers of the corrected rotation, not new algorithm ports.

## Rust destination and proposed minimal change

`crates/ares-core/src/fill/tpms_d.rs:34-45` currently keeps `f64::from(params.angle) + CORRECTION_ANGLE` in double; `:99-100` rotates the output with the same unrounded double. Preserve the double expression but narrow its result to f32 before widening for existing double geometry operations. Add the owning `FillTpmsD.cpp:108-110,155-160` citation at that boundary.

`project_slice/fill_entities/tpms_d.rs:23-33` already supplies the owning surface's f32 angle, f64 spacing/Z, f32 density, multiline count and anchors; it needs no behavior change. The existing fill module is a temporary compatibility shell around `FillTpmsD`, and this slice corrects its missing upstream numeric boundary.

Only this TPMS-D angle boundary is proposed. No shared rotation/Clipper/math primitive, source-angle conversion, density, wave equation, simplifier, connector, materialization, or processor change is included. No region indexing shortcut, new API, guard, fallback, ordering postprocess or external crate is needed. Existing files/new test helpers must remain at most 400 LOC.

## Native-first survey and exact differences

Survey receipts are outside the repository:

- `/tmp/worker6-tpms-angle-survey.xa3wp1lb/report.json` and `exact-analysis.json`.
- `/tmp/worker6-tpms-repeat.7fyn8ln0/report.json`: exact-input 30-degree repeat, fresh root/cache in each mode.
- Commands: official `/tmp/orca-wrapper-inherit-cpu.sh` or seeded `scripts/orca-seeded-parity.sh`, `--slice 0 --outputdir <fresh/native> <input.3mf>`; current-main FULL9 CLI `ares slice -o <fresh/ares.gcode> <same input.3mf>`.
- Existing CLI SHA256 `d28f18e0b88f46e0dbf47023ad93f9d19d01d87dfc051f09922348f491a2f41c`. Official wrapper SHA256 `1689a2ff46564cf723ad90c663051b58115920c3cce639bd2961340955cebbf3`. Seeded producer `/nix/store/1k0p5nlbs3zsfdcy08g3y5fb7bcm1qrv-orca-slicer-seed-replay-2.4.2/bin/orca-slicer`, seed 0.

All 12 survey executions and both repeats admitted the input (Native/Ares exit 0). Outer taskset CPU11; 107 actual Native affinity samples across survey/repeat, all CPU11. No compilation, intervention, full, workspace, timeout or retry occurred. Both modes produced the same generator-only-normalized Native bytes for all six survey inputs. The 30-degree repeat reproduced both complete normalized streams with identical raw input SHA. This is evidence of a reproducible parity gap; the proposed narrowing has not yet been executed or proven sufficient.

Inputs come from the existing Adaptive precision runtime export of the authored `tests/parity/multiline-union/prism.stl`, SHA256 `1d50f639d940e2a95c324b8f7ef1a33ba9bcf7e83706ba3cf0669f67f6feae7e`. Export construction is documented by `runner/adaptive_cubic_precision_inputs.rs:6-80`: Ender-3, classic walls, 0.2 mm layers, 3 top/bottom shells, no support/raft/brim/skirt/tower/ironing/arc fitting, 30% sparse fill, 0.45 mm sparse width, multiline 1. The survey changes only project settings for sparse pattern and angle (explicitly preserving multiline 1); mesh and all other ZIP members remain unchanged. The runtime-exported 3MF is evidence only and will not be committed. The survey receipt's `count` field denotes angle, not multiline; `exact-analysis.json` records these separately.

Full ordered bytes use only the existing generated-by normalization. Offsets below are zero-based in that normalized stream; line numbers are one-based. Both modes have identical first differences:

| TPMS-D angle | First byte | First line | Native line | Ares line |
| --- | ---: | ---: | --- | --- |
| 0° | 45055 | 1725 | `M73 P20 R41` | `G1 X137.853 Y96.663 E.02605` |
| 15° | 15677 | 604 | `M73 P9 R47` | `G1 X126.084 Y133.02 E.70064` |
| 30° | 25087 | 970 | `G1 E-2.8 F3600` | `M73 P15 R45` |
| 90° | 1941 | 93 | `M73 P2 R51` | `G1 X141.087 Y87.873 E.04184` |

These earliest whole-stream differences are M73 placement/timing differences, including the 30° retract versus M73 position. They do not by themselves locate the fill defect. Comparing ordered sparse extrusion records separately gives the first geometry/segmentation differences below; this diagnostic does **not** replace the full-byte comparator:

| Angle | Sparse record index (zero-based) | Layer (one-based) | Native line/command | Ares line/command | Sparse extrusion count N/A |
| --- | ---: | ---: | --- | --- | --- |
| 0° | 17260 | 6 | 18775: `G1 X138.349 Y101.924 E.00628` | 18775: `G1 X138.349 Y101.925 E.00628` | 74261 / 74280 |
| 15° | 150 | 4 | 1568: `G1 X137.088 Y128.813 E.00615` | 1568: `G1 X137.066 Y128.622 E.01253` | 72104 / 72120 |
| 30° | 4035 | 4 | 5457: `G1 X82.091 Y125.7 E.02602` | 5457: `G1 X82.091 Y125.701 E.02602` | 75223 / 75222 |
| 90° | 681 | 4 | 2098: `G1 X134.82 Y128.08 E.02605` | 2099: `G1 X135.358 Y127.831 E.00637` | 74317 / 74336 |

TPMS-D 45° is whole-byte equal (80123 lines, 75436 sparse extrusions); TPMS-FK 0° is whole-byte equal (53036 lines, 47473 sparse extrusions). The 45° correction stays below EPSILON in both implementations; it is a useful nonrotating control. TPMS-FK already narrows its corrected angle before geometry (`fill/tpms_fk.rs`, upstream `FillTpmsFK.cpp:123`). No TPMS-FK production edit is proposed.

An earlier 3D Honeycomb hypothesis was rejected: `Point.hpp:197` already rounds doubles, and Honeycomb 17/30/43% × multiline 1/2, regular Honeycomb and Rectilinear controls were whole-byte equal in both modes. Evidence `/tmp/worker6-nonironing-survey.4kl2xb10/report.json`. The TPMS-D narrowing is the smallest source discrepancy with a reproduced non-ironing witness in this bounded survey.

## Included and deferred behavior

Include TPMS-D sparse corrected-angle narrowing and its existing forward/back rotations, initially single-region/by-layer/count1/no-support/no-ironing. The shared TPMS-D kernel also serves other consumers: record any observed non-RED raw change and report it before broadening acceptance. Adjacent consumers get controls, not speculative edits.

Defer Adaptive/SupportCubic, every ironing pattern/dispatch, TPMS-FK numeric changes, multiline expansion changes, new rotation/connection algorithms, heterogeneous-region behavior not exercised here, and unrelated fill ordering. W2's `anchor_projection.rs` remains untouched. The four witnessed TPMS-D legs cannot be silently removed if this narrow fix fails to close them; report the residual and request a new boundary ruling.

## RED, GREEN and review gates after approval

1. Commit a separate public CLI E2E RED using the existing OrcaRunner export/slice cache from authored STL and settings. Test helpers live in a dedicated `runner/tpms_d_rotation_precision*` module; do not edit worker5's support-ironing test/domain files. Export and verify pattern/angle/width/density/count settings, then pass the same runtime 3MF to the validated core API and CLI. Native dependencies follow shared required FAIL / visible optional SKIPPED policy. Commit no Native stream, exported project, receipt, map or fixed output hash.
2. Establish four RED legs (0/15/30/90) and equal 45°/TPMS-FK controls in official and seeded fresh runs. Assert public admission, exact full ordered generator-only bytes, line count and layer schedule. Preserve sparse diagnostic records only to identify geometry; no tolerances or M73 filtering. Report the separate RED hash before production.
3. GREEN makes only the corrected-angle type boundary described above. Require all four legs to close completely, including M73 and sparse output. If not, STOP and report the exact residual; do not widen production scope or weaken comparisons. Controls' complete raw Ares output must remain unchanged. Add a multiline TPMS-D control and existing support-ironing TPMS-D witness to observe shared-kernel consumers; any newly changed raw output outside approved RED is reported separately.
4. At the exact clean GREEN hash, run directed official/seeded fresh CPU11 jobs1/threads2, option_coverage whole group in both modes, strict clippy, fmt and wasm32 within coordinator windows. Broad workspace/full/CI gates are coordinator-assigned only. No full/workspace is run for this SPEC survey.

No production or test implementation is part of this document-only proposal.
