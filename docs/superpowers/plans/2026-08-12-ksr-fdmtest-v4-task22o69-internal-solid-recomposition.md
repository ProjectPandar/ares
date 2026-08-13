# Task 22O.69 implementation plan

1. Freeze pinned upstream closure (`PrintObject.cpp:3368-3374`, stable Surface
   filter/copy, no-safety difference, exact ExPolygon safety union) and
   the exact destination seam:

   ```rust
   pub(in crate::project_slice) fn recompose_internal_solids(
       fill_surfaces: &[RegionSurface],
       additional_ensuring: &[ExPolygon],
       cut_from_infill: &[Polygon],
   ) -> Result<Vec<RegionSurface>, ClipperError>;
   ```

2. Obtain independent pre-RED approval; add the ordinary module plus separate
   ordinary test module and preserve one behavioral RED log.
3. Implement stable InternalSolid selection, ordered ensuring append, exactly
   one no-safety difference, exact ExPolygon topology forwarding, exactly one
   safety union, and fresh default-metadata InternalSolid output—without early returns
   for empty operands.
4. Add injected operand/call-order fixtures and natural topology/empty/error/
   nonmutation tests. Kill compiling early-return, operand/order, per-item,
   repeated-call, bypass, ExPolygon topology/hole, error, metadata/kind, and sorting
   mutations; restore production byte-exactly.
5. Run focused/dependency/workspace Nextest, warning-denying Clippy/rustfmt,
   wasm32, x86_64/aarch64 Windows and macOS, and diff/LOC/static/include/
   pinned-Orca/no-staged gates.
6. Run independent read-only six-axis implementation review; fix only in the
   main thread and re-review until unconditional approval.

The module remains unwired and `ares-core`-private, trusts same-transaction
normalized Clipper-safe inputs, and adds no validation, option lookup, fallback,
I/O, threading, UI/OpenGL, unsafe, or platform behavior. Debug drawing at
`3376-3383`, region commit at `3385-3386`, composer, second pass, lifecycle, and
G-code/CLI/golden parity remain deferred.

## Completion evidence

Behavioral RED is preserved. Focused 6/6, dependency 794/794, workspace
6,454/6,454, strict/portability/static gates, and 26/26 compiling mutations pass
with byte-exact restoration SHA-256
`d170b25cb69d48a4befba3cb766eede5387109308ce0452961b3dc174f4bde3d`.
Independent six-axis implementation review approved with no blockers or major repairs.
