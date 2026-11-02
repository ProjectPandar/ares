# Beta 3D honeycomb rewrite slice

Status: proposed for review; implementation requires approval of this boundary and its geometry-scoped oracle.

## Authority and acceptance boundary

Authority is OrcaSlicer 2.5.0-beta, commit `345efec64c7b95a05ce5cf96df0dde66a158835c`. The implementation base is `f4ea41332cdcb7eadf9adbc7f7c1936c0ad7e9c5`.

This is a Rust rewrite of `src/libslic3r/Fill/Fill3DHoneycomb.cpp`, not a new Ares fill pipeline. The existing `crates/ares-core/src/fill/three_d_honeycomb.rs` implements the older 2.4.2 construction and must be replaced, not retained behind a fallback or selector.

Acceptance is geometry-scoped. The original `plater_stl_session_centers_moves_and_slices_like_native` assertion and its full-stream comparator remain unchanged and failing. Independent Klipper timing and preset-footer discrepancies are outside this geometry slice; closing the geometry gap does not establish full plater parity.

## Qualified discrepancy

On `444364eaf2c82096cc00de06fa963a7ac40c6475`, the unchanged profile-entry test runs against official beta Native and fails at normalized byte 2157, line 91: Native places `M73 P1 R6` one motion record earlier.

External diagnostics over the complete captured streams on the original qualification base and `44fb3e6912efec80b2a12e01df6a44045daf3651` find:

- First non-M73 difference: line 549, Z=1.0, Native `G1 F938`, Rust `G1 F946`.
- First spatial difference: line 568, entering sparse infill at Z=1.0, Native `G1 X122.139 Y134.924 F24000`, Rust `G1 X122.139 Y134.046 F24000`.
- Both effective configurations select classic walls, 20% `3dhoneycomb`, 0.2 mm layers and four bottom shell layers.
- The complete Native/Rust streams contain 2871/2794 spatial motion records. This is not merely a misplaced progress line.

The unchanged test also fails on `44fb3e6912efec80b2a12e01df6a44045daf3651` at the same byte, line and context. That run selects one test, fails it, and filters out 2158 tests. Its captured API stream is byte-identical to the earlier capture. On the implementation base, the unchanged test again fails at byte 2157, line 91 with the same M73 context and expected/actual lengths of 137764/136217 bytes. This run selects one FAIL and filters out 2177 tests. The fixed authority files match their Git blobs. Temporary diagnostics are not a replacement acceptance comparator.

## Source-to-Rust boundary

Upstream paths in this table are under `src/libslic3r/`. Rust paths are under `crates/ares-core/src/`.

| Upstream boundary | Rust destination | Included behavior |
| --- | --- | --- |
| `Fill/Fill3DHoneycomb.cpp:41–87`, `triWave`, `troctWave`, `getCriticalPoints` | private `fill/three_d_honeycomb/wave.rs` | Native f32 phase storage and integer extraction, signed wave branches and the ordered four critical points. |
| `Fill/Fill3DHoneycomb.cpp:229–287`, `patternPoints`, `makeZigZag` | private `fill/three_d_honeycomb/grid.rs` | Native path direction, alternating cells, native multiline offsets and endpoint order. |
| `Fill/Fill3DHoneycomb.cpp:174–224`, `makeEndPoints` | private `fill/three_d_honeycomb/grid.rs` | Boundary templates, parity, extents and translated end connectors. |
| `Fill/Fill3DHoneycomb.cpp:92–168`, `addTops` | private `fill/three_d_honeycomb/tops.rs` | Top-cover extents, adjusted spacing, direction and multiline transitions. |
| `Fill/Fill3DHoneycomb.cpp:296–324`, `makeGrid` | private `fill/three_d_honeycomb/grid.rs` | Ordered zigzags, end connectors and conditional top covers using current/previous Z. |
| `Fill/Fill3DHoneycomb.cpp:336–398`, `_fill_surface_single` setup | `fill/three_d_honeycomb.rs` | Actual physical layer height, thickness-layer count, density branch, `2*gridSize` alignment, `startOffset` and both Z arguments. |
| `Fill/Fill3DHoneycomb.cpp:400–436`, output processing | `fill/three_d_honeycomb.rs` | Translation, source corner smoothing, clipping, strict minimum length, existing chain/connect handoff and final rotation. |
| `Fill/FillCornerSmoothing.hpp:18–137`; `.cpp:17–299`, polyline branch | private `fill/three_d_honeycomb/smoothing.rs` and small private Bezier/cache helpers if required by size limits | The dependency called at `Fill3DHoneycomb.cpp:405`: source curve construction, tolerance, bounded shape caches, streaming endpoints, integer emission and duplicate removal. No smoothing rollout to other patterns. |
| `Fill/Fill.cpp:308–402,961,1044,1432,1440,1476`; `Fill/FillBase.hpp:91,101–105,130`; `PrintConfig.hpp:1477,1482` | `options/region_fields.rs`; field-only `project_slice/group_fills/{types,coalesce,params/projection}.rs` changes | Existing `infill_complete_top` and `sparse_infill_smooth_factor` binding, grouping identity and source parameter forwarding. No registry redesign or unrelated option work. |
| `Fill/Fill.cpp:1432` and `_fill_surface_single` thickness argument | `project_slice/fill_entities/{three_d_honeycomb,generate,surface}.rs`, only 3D calls | Carry physical layer height separately from extrusion-flow height or combined-surface thickness. Forward source smoothing resolution without changing other pattern calls. |
| `Fill/Fill.cpp:1561,1647–1667`, `Layer::generate_sparse_infill_polylines_for_anchoring` | only the 3D-honeycomb parameter fields in `project_slice/prepare_infill/bridge_over_infill/sparse_anchoring/arms.rs` | Forward physical height, thickness count, resolution and smooth factor. Keep complete-top false as the upstream anchoring caller does. No bridge algorithm, dispatch-signature, or surrounding arm rewrite. |

Private file splits follow these upstream functions and the 400-line limit, not a new geometry API. Source scalar types, cast positions, operation order, loop order and branch inequalities remain authoritative. The sole safe Rust Clipper2 adapter remains the clipping backend.

### Remove the old construction

Delete the old `make_grid`, `collinear_points`, `perpendicular_points` and their 2.4.2-only request types when the beta functions replace them. Remove the synthetic 1 mm layer height, repeated layer-module resnapping, rounded `4*gridSize` alignment and old external multiline/large Douglas–Peucker treatment from this generator. Native beta owns multiline generation and smoothing here.

Keep only existing wrapper operations that correspond to the cited beta fill entry/output boundary. Do not preserve an old path for untested settings, introduce a compatibility mode, hardcode the observed coordinate, or translate/canonicalize output to force agreement.

## Included settings and deferred work

Included: the ordinary-density and dense/small-module branches; physical layer height and combined-layer count; native multiline paths; complete-top false/true behavior; and the polyline corner-smoothing dependency, including its disabled path. Existing recognized configuration keys are forwarded rather than replaced with constants.

Deferred: all other fill-pattern rewrites; generic connection, Clipper2, polygon smoothing and shortest-path changes; bridge-generation logic; Klipper timing; M73 placement; preset loading/refresh and footer serialization. If the first remaining bad operation belongs to one of these boundaries, report it instead of expanding this implementation.

The core stays rendering-neutral and browser-WASM capable, without filesystem, terminal, native runtime, FFI or viewer dependencies. Native execution belongs only in the CLI parity harness. No new public geometry API is needed.

## RED plan

1. Preserve the original cube STL, selected LH presets, profile-entry gate and strict plater test. Record its failure on the implementation base before product changes; do not relabel it PASS or suppress its timing/footer residuals.
2. Add separate centered and moved plater geometry witnesses using the same public session/profile APIs and official Native commands. Each keeps its own input, effective settings, raw Native/API output, command receipts and hashes.
3. The new scoped oracle compares complete ordered motion records: exact lexical `G0/G1/G2/G3` commands and their non-feedrate motion operands, including XYZE and arc operands when present. It excludes M73, pure feedrate-setting records and non-geometric metadata only in this explicitly named geometry witness. Keep `G20/G21/G90/G91/G92/M82/M83` coordinate/extrusion mode records unchanged in that ordered witness. It does not sort, parse/round coordinates, translate paths, reverse rings or use tolerances. The original byte comparator is untouched.
4. The centered witness must reproduce the first Z=1.0 XY gap above. Assert the whole ordered geometry stream, not only that coordinate or a selected matching window. Keep layer/role context in failure artifacts.
5. Run the existing 3D-honeycomb option/domain parity cases with their actual reference gates enabled. Do not confuse validation cases with successful sparse-fill Native witnesses.
6. Before implementation, add Native E2E geometry cases for density/module branches, multiline, complete-top and nonzero smoothing, reusing existing immutable fixtures/harnesses where possible. Each case must expose the beta behavior it protects and record RED before product code.

No committed diagnostic hooks or general comparator framework are proposed. The scoped geometry assertion is new and explicit; it neither weakens nor replaces an existing assertion.

## GREEN and regression exit criteria

- The new centered and moved geometry witnesses pass against official beta Native, including closure of the first Z=1.0 XY gap and all later ordered motion geometry.
- The beta-specific density, multiline, cover and smoothing witnesses pass. Unexercised branches are not described as qualified.
- The frozen implementation-base selector `test(honeycomb) | test(fill) | test(plater_stl_session)` selects 117 existing tests and filters out 2061. The sorted test-name list has SHA256 `9e62b5743e48a36560e7a7ea87c1de4f048bf13e93b27344fab77f6b52096070`. Include any additional existing 3D-honeycomb cases found by source registration. Inventory is not a test result. Record required oracle switches and actual command receipts for oracle-bearing tests.
- Compare that exact family before/after: **zero PASS-to-FAIL**. Report unchanged failures, improvements, selected tests and filtered tests separately. Do not count an inactive reference gate as a Native PASS.
- Rerun the unchanged strict plater test after geometry GREEN and report its first non-M73 difference from the complete streams. Explicitly report whether the initial Z=1.0 `F938`/`F946` gap closes, including any cooling slowdown change derived from corrected path length. Excluding feedrates from the geometry oracle does not exclude this check from the slice. If it remains, identify the source-owned cause and ownership before proposing a separate residual. Do not silently omit or automatically defer it.
- Keep the original strict plater assertion and report every remaining failure, including timing re-measured after the separate Klipper TimeMachine port lands and preset-footer differences. Geometry-scoped GREEN is not full-output GREEN.
- Run Rust tests with nextest, plus rustfmt and strict workspace/all-target clippy. Compile `ares-core` and `ares-wasm` for `wasm32-unknown-unknown`. New or changed implementation modules stay within the size/lint limits without new suppression attributes.
- Keep separate specification, RED and GREEN commits with reproducible external artifacts. Remove temporary product diagnostics before submission; do not amend accepted history.

## Independent timing handoff

The input's first five G1 commands are byte-identical before any honeycomb path. Captured Native planned fields reconstruct 0.776319453865 s; Rust's captured schedule totals 0.741911402903 s. The first block is 0.015491933693 s versus 0.010625000112 s. The totals agree with the respective public first-layer-time footer values at emitted precision.

These measurements describe the original qualification base. The paired Native diagnostic produces exactly the official Native stream after generator normalization and selects Klipper planning (`minimum_cruise_ratio=0.5`). The Klipper TimeMachine Rust port belongs to a separate slice and is already in integration verification. Re-measure timing and the strict-stream residual after that port lands. Do not carry the historical planner-dispatch diagnosis forward as a claim about the current implementation. Its upstream boundary is `GCode/GCodeProcessor.cpp:505::TimeMachine::plan_klipper`, `:576::TimeMachine::calculate_time`, and flavor dispatch `:8058`.

That is an independent timing slice, not a consequence to repair by changing honeycomb coordinates or progress-line placement. Detailed timing captures remain external for its owner. No timing implementation is included in this specification.
