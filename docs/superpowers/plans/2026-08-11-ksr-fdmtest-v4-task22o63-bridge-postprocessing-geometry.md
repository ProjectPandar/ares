# Task 22O.63 bridge postprocessing geometry implementation plan

## Status

Implemented and unconditionally approved by independent six-axis implementation review.

## Objective

Port pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`OrcaSlicer/src/libslic3r/PrintObject.cpp:3290-3298`, into exact Rust destination
`crates/ares-core/src/project_slice/prepare_infill/bridge_over_infill/candidate_bridge_postprocessing.rs`.
Compose existing source-cited O48/O53/O62 and flat Clipper rewrite dependencies;
do not create an Ares-owned pipeline.

Included: active `0.75` opening, one-spacing closing, limiting and total-fill
intersections, total-top difference, and final expansion-area difference.
Direct dependencies are `Flow.hpp:62-69`, `libslic3r.h:38-43,60-94`,
`ClipperUtils.hpp:19-27,400-425,430-432,495-498`, and
`ClipperUtils.cpp:264-403,593-632,671-679,702-703`; plus O53 scaling, O62 output,
Ares `geometry/clipper/offset/opening.rs:4-12`,
`offset/execute.rs:54-64,162-184`, and `boolean_paths.rs:18-30,59-66`.

Deferred: debug drawing, candidate append `3304-3305`, layer swap/clear,
composer/history production, prepared successor/lifecycle, second bridge pass,
region-surface rewrite, extrusion, motion, G-code, CLI, and golden parity.

## Plan

1. **Approve boundary and seam**
   - Independently review ADR/spec/plan against all pinned source/dependencies.
   - Repair until unconditional approval for behavioral RED.

2. **Write vertical behavioral RED**
   - Register the ordinary private module and exact result/function seam.
   - Add one missing-behavior test and preserve RED output.
   - Add literal dual-scale arithmetic, active 0.75, operation/operand trace,
     ownership/nonmutation, empty, actual morphology/boolean, and error tests in
     vertical slices.

3. **Implement minimum behavior**
   - Reuse narrow O53 scaling; consume O62 and expansion state.
   - Call `opening_paths(..., d, d, Miter, 3)`, then compose flat closing
     explicitly as `offset_paths(..., +spacing, Miter, 3)` followed by
     `offset_paths(..., -spacing, Miter, 3)`, then call intersections and
     differences in exact source order.
   - Preserve boundaries/angle; return final bridge and expansion state.
   - Add no options, validation fallback, composer/commit/lifecycle, filesystem,
     platform branch, or sorting.

4. **Verify discrimination/restoration**
   - Run focused and exact dependency tests.
   - Reversibly mutate arithmetic, factor, morphology kernels/order,
     boolean operations/roles/order, final expansion clip, ownership/errors, and
     output ordering; require every mutation killed and byte-exact restoration.
   - Run workspace Nextest, strict Clippy, rustfmt, wasm32, four Windows/macOS
     targets, diff/LOC/static, clean pinned Orca, and no-staged checks.

5. **Independent final review loop**
   - Fresh read-only reviewer covers requirements, logic, edges, quality,
     coverage, and actual runtime.
   - Main thread repairs findings, reruns affected/full gates, and requests
     re-review until unconditional approval.

## Exit criteria

- Exact pinned postprocessing and expansion subtraction are implemented.
- Tests/mutations discriminate every required arithmetic, kernel, operand,
  ordering, ownership, and error invariant.
- Private/unwired ordinary-module architecture and 399-line cap hold.
- Runtime, mutation, portability, static, and final review gates pass.
