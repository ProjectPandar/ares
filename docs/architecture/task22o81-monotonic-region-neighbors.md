# Task 22O.81 architecture decision record

## Status

Accepted for implementation. Decision date: 2026-08-13.

## Decision

Port pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/Fill/FillRectilinear.cpp:2079-2179`, into
`fill::rectilinear::neighbors`.

Populate O80 monotonic regions' ordered left/right neighbor indices from the
linked boundary overlaps. Preserve region order, source boundary scan order,
duplicate suppression, sorted neighbor identity, and final bidirectional
symmetry repair.

Deferred: region path lengths, ant matrix/chaining, polyline output, entities,
lifecycle, and G-code. No legacy fallback, fixture branch, or public API.

Two focused tests pass for sorted unique symmetric adjacency and disconnected
identity. All 1,179 task22o core regressions pass. Strict core Clippy, rustfmt,
diff, and sub-400-LOC checks pass; the neighbor shard is 55 LOC.
