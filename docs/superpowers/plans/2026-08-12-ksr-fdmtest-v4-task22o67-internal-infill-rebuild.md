# Task 22O.67 internal infill rebuild implementation plan

## Status

Implemented after approved behavioral RED; final independent review pending.

## Objective

Port pinned Orca `PrintObject.cpp:3345-3350` to private ordinary module
`bridge_over_infill/internal_infill_rebuild.rs`, composing O65/O66 geometry and
existing no-safety Clipper APIs only.

## Plan

1. Independently approve source closure, seam, trusted domain, and deferrals.
2. Register exact stub seam and ordinary test children; preserve behavioral RED.
3. Add direct default `difference_ex_polygons` dependency coverage if needed.
4. Implement stable exact-Internal selection, two sequential default no-safety
   differences, and fresh default-metadata Internal materialization.
5. Add no batching/safety/sort/validation/composer/lifecycle behavior.
6. Run focused/dependency/workspace, compile-valid mutation/restoration,
   Clippy/rustfmt, WASM/four desktop, LOC/static/pinned/no-staged gates.
7. Run independent read-only six-axis review; repair and re-review to approval.

## Exit criteria

Exact `3345-3350` behavior, ownership, metadata, order and error semantics are
discriminated; all valid mutants die; files stay below 400 LOC with ordinary
modules/no include splitting; all runtime/static/review gates pass.
