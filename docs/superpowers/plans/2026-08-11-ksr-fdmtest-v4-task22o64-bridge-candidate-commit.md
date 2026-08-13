# Task 22O.64 bridge candidate commit implementation plan

## Status

Implemented and unconditionally approved by independent six-axis implementation review.

## Objective

Port pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`OrcaSlicer/src/libslic3r/PrintObject.cpp:3304-3310`, into exact Rust destination
`crates/ares-core/src/project_slice/prepare_infill/bridge_over_infill/candidate_bridge_commit.rs`.
Compose existing O43 identity and O63 output only; do not create an Ares-owned
pipeline.

Included: one-candidate conversion/append, expansion ownership continuation,
and current-layer vector swap/clear. Deferred: reserve/orchestration/debug,
BTreeMap and cluster traversal, second pass `3315+`, region rewrite, lifecycle,
extrusion, motion, G-code, CLI, and golden parity.

## Plan

1. **Approve boundary and seam**
   - Independently review ADR/spec/plan against pinned source and O43/O63 types.
   - Repair until unconditional approval for behavioral RED.

2. **Write vertical behavioral RED**
   - Register the ordinary private module and exact two-function seam.
   - Add one missing-behavior test and preserve RED output.
   - Add order, source/angle bits, move/allocation, empty, replacement/drop, and
     repeatability tests in ordinary child modules.

3. **Implement minimum behavior**
   - Destructure O63 state, push one exact `CandidateSurface`, and return the
     owned expansion vector.
   - Swap completed into current and clear the swapped-out original vector.
   - Add no geometry, sorting, validation, map traversal, option lookup,
     fallback, error channel, composer, commit successor, or lifecycle.

4. **Verify discrimination/restoration**
   - Run focused and exact dependency tests.
   - Reversibly mutate append cardinality/order, source/angle, cloned polygon
     and expansion state, empty handling, returned expansion, missing/cleared or
     inverted replacement, and allocation reconstruction; require every
     behavioral mutation killed and byte-exact restoration.
   - Structurally audit explicit swap-then-clear, boundary non-leak, absence of
     raw geometry operands, and absence of map/cross-layer access. Do not claim
     observationally equivalent omitted-clear or assignment variants as kills.
   - Run workspace Nextest, strict Clippy, rustfmt, wasm32, four Windows/macOS
     targets, diff/LOC/static, clean pinned Orca, and no-staged checks.

5. **Independent final review loop**
   - Fresh read-only reviewer covers requirements, logic, edges, quality,
     coverage, and actual runtime.
   - Main thread repairs findings, reruns affected/full gates, and requests
     re-review until unconditional approval.

## Exit criteria

- Exact pinned append and layer replacement semantics are implemented.
- Tests/mutations discriminate every required order, identity, ownership,
  cardinality, and replacement invariant.
- Private/unwired ordinary-module architecture and 399-line cap hold.
- Runtime, mutation, portability, static, and final review gates pass.
