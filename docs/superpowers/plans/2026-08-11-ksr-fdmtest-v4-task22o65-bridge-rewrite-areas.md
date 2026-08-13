# Task 22O.65 bridge rewrite-area implementation plan

## Status

Implemented after approved behavioral RED; final independent implementation review pending.

## Objective

Port pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`OrcaSlicer/src/libslic3r/PrintObject.cpp:3318-3319,3322-3336`, into exact Rust destination
`crates/ares-core/src/project_slice/prepare_infill/bridge_over_infill/bridge_rewrite_areas.rs`.
Compose O43/O53/O64 and existing flat Clipper dependencies only; do not create
an Ares-owned pipeline.

Included: key-presence gate, current cut flattening, and sequential upper
ensuring-ring collection using already-resolved Task 22N Flow. Deferred:
traversal/timeouts `3315-3317`, layer retrieval `3320`, source-to-record
projection and upstream Flow errors, per-region
rewrite `3338+`, second pass, composer/lifecycle, extrusion, G-code, CLI, and
golden parity.

## Plan

1. **Approve boundary and seam**
   - Independently review ADR/spec/plan against pinned source and dependencies.
   - Repair until unconditional approval for behavioral RED.

2. **Write vertical behavioral RED**
   - Register the ordinary private module and exact typed seam.
   - Add one missing-behavior test and preserve RED output.
   - Add all four presence gates, current-only range cloning,
     flatten/clone/order, Flow/cast, operation/operand, empty, natural
     offset/injected difference error-order, repeatability, and nonmutation tests
     in ordinary child modules.

3. **Implement minimum behavior**
   - Preserve Option key presence and clone current paths in source order.
   - For each upper candidate, reuse O53 scaling, run one negative Miter/3
     offset and one default difference, then append output.
   - Add no batching, union, safety, sort, validation, option inference, map,
     region rewrite, composer, successor, or lifecycle.

4. **Verify discrimination/restoration**
   - Run focused and exact dependency tests.
   - Reversibly mutate gate, flatten source/order, Flow/scale/cast, shrink
     sign/join/miter/cardinality, difference roles/safety/cardinality, batching,
     errors, and output ordering; require every compiling behavioral mutation
     killed and byte-exact restoration. Audit clone ownership structurally and
     with distinct-allocation/input-preservation assertions because a borrowed
     alias cannot satisfy the owned result type in safe Rust.
   - Run workspace Nextest, strict Clippy, rustfmt, wasm32, four Windows/macOS
     targets, diff/LOC/static, clean pinned Orca, and no-staged checks.

5. **Independent final review loop**
   - Fresh read-only reviewer covers requirements, logic, edges, quality,
     coverage, and actual runtime.
   - Main thread repairs findings, reruns affected/full gates, and requests
     re-review until unconditional approval.

## Exit criteria

- Exact pinned gate and rewrite-area collection semantics are implemented.
- Tests/mutations discriminate all order, arithmetic, kernel, ownership, and
  error invariants.
- Private/unwired ordinary-module architecture and 399-line cap hold.
- Runtime, mutation, portability, static, and final review gates pass.
