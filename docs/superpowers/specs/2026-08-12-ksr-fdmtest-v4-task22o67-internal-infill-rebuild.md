# Task 22O.67 — internal infill rebuild

## Status

Implemented after approved behavioral RED; final independent review pending.

## Goal

Port pinned Orca `PrintObject.cpp:3345-3350` into private ordinary module
`prepare_infill/bridge_over_infill/internal_infill_rebuild.rs` without wiring
the transaction lifecycle.

## Contract

```rust
pub(in crate::project_slice) fn rebuild_internal_infills(
    fill_surfaces: &[RegionSurface],
    cut_from_infill: &[Polygon],
    additional_ensuring: &[ExPolygon],
) -> Result<Vec<RegionSurface>, ClipperError>;
```

## Behavior

1. Select exact Internal surfaces stably; preserve component/contour/hole order.
2. Run one default no-safety selected-internals-minus-cut difference.
3. Run one default no-safety first-result-minus-ensuring difference.
4. Build fresh Internal surfaces in engine order with default metadata
   `(-1.0, 1, -1.0, 0)`.
5. Empty inputs still execute both calls. First error wins; inputs and
   allocations remain unchanged.
6. Add no safety, union, batching, sorting, validation, fallback, option lookup,
   region mutation, or lifecycle.

Direct source closure is `SurfaceCollection.cpp:45-52`,
`Surface.hpp:9-77,116-157`, `ClipperUtils.hpp:183-220,265-307,442-455`, and
`ClipperUtils.cpp:734-769`.

## Included/deferred

Included only `3345-3350`. Deferred: bridge conversion `3352+`, solids,
replacement, second pass, composer/lifecycle, extrusion, G-code, CLI, golden.

## Acceptance

Tests and compile-valid mutations must discriminate exact filtering/order,
holes, both call operands/cardinality/no-safety, second-result dependency,
metadata reset, empty traversal, first/natural errors, output engine order,
repeatability and input allocation preservation. Require byte-exact restoration,
focused/dependency/workspace Nextest, strict lint/format, WASM/four desktop
targets, LOC/static/pinned/no-staged, and independent six-axis approval. Every
Rust source is at most 399 LOC; ordinary modules only; no include macros for
splitting.
