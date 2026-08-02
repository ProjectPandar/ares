# Task 22O.12 implementation plan

1. Pin Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1` and the exact `ThickLine` / `ThickPolyline` boundaries listed in the paired spec.
2. Add literal tests for source constructors, defaults, reversal, clear, thick-line projection, closed rotation, and fixed-width conversion.
3. Port safe crate-private `ThickLine` and flattened `ThickPolyline` data with exact width and endpoint ordering; add no validation, fallback, dependency, or public API.
4. Record O12 as a prerequisite only. Do not advance the O11 lifecycle or claim medial-axis execution.
5. Run focused O12/O11 tests, workspace Nextest/Clippy/check, both WASM checks, rustfmt, diff, LOC, and static-policy audits.
6. Independently review source fidelity, scope, portability, tests, ownership, and documentation before approval.

The next milestone must cite and port the actual Voronoi implementation boundary needed by `Geometry::MedialAxis`; it may not replace it with a simplistic skeleton algorithm or runtime Orca oracle.
