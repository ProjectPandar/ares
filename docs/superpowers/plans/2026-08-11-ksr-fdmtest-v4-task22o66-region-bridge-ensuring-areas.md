# Task 22O.66 region bridge ensuring-area implementation plan

## Status

Implemented after approved behavioral RED; final independent implementation review pending.

## Objective

Port pinned Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`OrcaSlicer/src/libslic3r/PrintObject.cpp:3341-3343`, into exact Rust destination
`crates/ares-core/src/project_slice/prepare_infill/bridge_over_infill/region_bridge_ensuring_areas.rs`.
Compose Task 22N/O53/O65 and existing Clipper dependencies only; do not create an
Ares-owned pipeline.

Included: one region's all-surface near-perimeter ring and clipped O65 ensuring
areas. Deferred: enclosing region/new-surface context `3338-3339`, lines
`3345+`, layer/map/composer projection, second pass,
lifecycle, extrusion, G-code, CLI, and golden parity.

## Plan

1. **Approve exact source closure and seam**
   - Independently review ADR/spec/plan against pinned PrintObject, Surface,
     ExPolygon, Flow, scaling, and Clipper sources.
   - Repair until unconditional approval for RED.

2. **Write vertical behavioral RED**
   - Register one ordinary private module and exact typed seam with a deliberate
     missing-behavior stub.
   - Add exact private geometry overload
     `intersection_polygons_polygons_ex(&[Polygon], &[Polygon])`, re-export it
     through both geometry module roots, and directly test insertion order,
     NonZero topology, empty behavior, and subject/clip range errors.
   - Add focused ordinary child modules covering global flatten/union, exact
     arithmetic and operation order, topology/output, empty/error/nonmutation.
   - Preserve the failing RED command output.

3. **Implement minimum source behavior**
   - Flatten all region surfaces contour-before-holes in source order.
   - Run one safety union, flatten once, exact O53 negative Miter/3 shrink, one
     default flat difference, and one ensuring-subject intersection.
   - Return owned intermediates without sorting or mutating inputs.
   - Add no filters, validation, fallback, batching, rewrite, composer, or
     lifecycle behavior.

4. **Verify discrimination and restoration**
   - Run focused and exact dependency tests.
   - Reversibly mutate flattening/order/topology, union cardinality, Flow and
     scale/cast/sign/join/miter, difference cardinality/roles/safety,
     intersection cardinality/roles/safety, empty traversal, errors, and output
     order. Reject compile failures as evidence; require every compiling
     behavioral mutation killed and byte-exact restoration.
   - Run workspace Nextest, strict Clippy, rustfmt, wasm32, x86_64/aarch64
     Windows and macOS checks, diff/LOC/static, clean pinned Orca, and no-staged
     checks.

5. **Independent final review loop**
   - Fresh read-only reviewer covers requirements, logic, boundaries, quality,
     coverage, and actual runtime evidence.
   - Main thread repairs findings, reruns affected/full gates, and requests
     re-review until unconditional approval.

## Exit criteria

- Exact pinned `3341-3343` behavior and direct dependency semantics are ported.
- Tests and valid behavioral mutations discriminate all order, arithmetic,
  topology, cardinality, ownership, and error invariants.
- Private/unwired ordinary-module architecture and 399-line cap hold.
- Runtime, portability, static, restoration, and independent review gates pass.
