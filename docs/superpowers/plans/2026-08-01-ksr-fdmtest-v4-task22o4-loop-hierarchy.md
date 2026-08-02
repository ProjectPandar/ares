# Task 22O.4 execution plan

1. Freeze Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1` and the
   bounded loop/hierarchy sources; preserve all dirty O1-O3 predecessors.
2. Port Clipper v6 `PointInPolygon` with its active f64 `CrossProductType`
   determinant and expose boundary-inclusive `Polygon::contains`, verified by
   transparent tests.
3. Define a boxed-resolved typed hierarchy successor nesting O3 and carrying
   exact loop trees plus aligned diagnostic leftovers.
4. Materialize O3 raw shells directly in depth/source order: normal before
   smaller-width, contour before holes, without geometry recomputation.
5. Port the destructive hole-first and contour parent searches in exact
   depth/index order, removing and retrying the same source index.
6. Build every O4 sidecar by borrowing before moving its O3 predecessor; retain
   all optional slots and source ordering transactionally.
7. Wire the public lifecycle through O4 while retaining
   `ProjectSlicingIncomplete` and the existing stack-safe boxing.
8. Add direct geometry/materialization/nesting tests and real KSR
   reachability/determinism/predecessor/lifecycle tests in separate modules.
9. Update the architecture ledger and roadmap without claiming traversal,
   extrusion, G-code, or complete Task 22O.
10. Run focused Nextest, strict Clippy, native/WASM checks, rustfmt, LOC and
    forbidden-pattern scans; review the complete bounded diff.
