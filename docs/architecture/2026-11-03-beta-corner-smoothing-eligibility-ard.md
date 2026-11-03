# Beta corner-smoothing eligibility and planar honeycomb handoff

Status: source-bounded correction approved; paired verification gates passed.

## Authority and prior evidence

Authority is OrcaSlicer beta `345efec64c7b95a05ce5cf96df0dde66a158835c`.
This supplements the approved planar honeycomb ARD `d511fa2b24a5bbd213c1e08b37f51ab30bb384a1` and RED `6b1ed79fb33ec215b6f87ffa7de99495ab18f46c` on integrated base `8612802fec12baeaa32600edb8e0f3795d3eeb95`.

The first bounded port attempt passed sharp and seven 3D witnesses but retained three rounded planar geometry failures. Its caller forwarding missed the existing 3D-only regional-factor projection gate. That attempt is not GREEN. An initially proposed two-pattern guard was rejected because it was not the upstream predicate.

## Source-owned correction

| Upstream owner | Rust destination | Included behavior |
| --- | --- | --- |
| `src/libslic3r/PrintConfig.hpp:181–199`, `is_smoothable_infill_pattern` | `ProcessInfillPattern::is_smoothable`, beside the enum in `options/process_options/object_source/enums.rs` | Exact predicate, expressed as an exhaustive match over the Rust enum. |
| `src/libslic3r/Fill/Fill.cpp:1043–1044` | `project_slice/group_fills/params/projection.rs::project_surface` | Retain `InternalInfill`; use the predicate with the effective pattern and multiline before assigning `0.01 * regional percentage`. |
| `src/libslic3r/Fill/FillHoneycomb.cpp:81–92`, `FillHoneycomb::_fill_surface_single` | `fill/honeycomb.rs::fill_honeycomb` and normal/anchor parameter callers | Rotate → shared polyline smoother → multiline → clip → connect; remove the replaced DP stage. |
| `src/libslic3r/Fill/FillCornerSmoothing.hpp/.cpp`, `CornerSmoother`, `smooth_polyline_corners` | Private `fill/corner_smoothing` | Relocate/reuse the existing implementation with unchanged arithmetic, caches, emission rounding and EPSILON value. |

The exact eligibility mapping is:

- Always true: HilbertCurve, OctagramSpiral, Lightning, Honeycomb, ThreeDHoneycomb, Concentric, CrossHatch.
- `multiline > 1`: Grid, Triangles, TriHexagon (`ipStars` upstream), Cubic.
- False: every other represented upstream pattern, listed explicitly in the Rust match. No wildcard or supported-kernel subset may replace this mapping.

`SurfaceFillPattern::ConcentricInternal` is not a configured process pattern and is not admitted through this guard. Print resolution is already projected independently. No registry entry, serialization change, new public API or additional curve engine is required.

## Non-negotiable scope

This is a `libslic3r` predicate/handoff rewrite slice, not an Ares-owned feature design. The old 3D-only guard is temporary scaffold and is replaced, not retained as a fallback. Eligibility must not be conflated with which Rust kernels currently consume the field.

Other kernels must not start smoothing as a side effect of this change. In particular, `FillPlanePath.cpp::generate_path` and its streaming `SmoothingPolylineOutput`, polygon smoothing and other pattern integrations remain deferred. Existing unconsuming kernels must retain identical byte output under the approved breadth comparison. Do not alter their algorithms, ordering, coalescing, clipping, timing or footer handling to accommodate a result.

The three support/raft honeycomb initializers keep Native default smoothing zero and resolution .0125. Normal and anchor callers forward the regional factor and print resolution. Shared smoother relocation must preserve the existing 3D results. No compatibility selector, new clipping backend, global rounding change or suppression is permitted.

## Verification contract

- Use the unchanged four planar public witnesses plus the seven existing 3D witnesses. Sharp stays PASS; the three qualified planar geometry REDs must flip without weakening the oracle.
- Execute the same 132 IDs once on RED `6b1ed79fb33ec215b6f87ffa7de99495ab18f46c`, then once on corrected source: eleven geometry witnesses plus 121 rectilinear/grid/cubic/lightning/concentric breadth IDs. Disable both test-runner and Native-helper retries. Require P→F zero over all 132 IDs. Compare the same persisted API evidence per ID for hash equality, recording the three intended rounded planar output changes separately and accounting explicitly for IDs without emitted API artifacts.
- The accepted baseline has 33 PASS / 99 FAIL. For the coplanar outer-wall witness, compare its persisted `api-outer-wall.raw` projection: the full stream is not persisted, and the test's own Native outer-wall comparison establishes PASS. Do not relabel the projection as a complete stream or infer full-stream parity. No replacement baseline or additional replay is authorized.
- Preserve the first failed attempt and the new exact source/input/settings/binary/command/result artifacts separately. No diagnostic replay or additional test-body run is inherited from the earlier budget.
- If the new attempt fails, stop and report; do not start another fix/run loop.
- Keep all F/M73/footer differences separate from geometry. No full strict parity is inferred from geometry PASS.
- Formatting, strict all-target Clippy, core/WASM wasm32 builds and the 400-line/no-new-suppression limits remain required.
