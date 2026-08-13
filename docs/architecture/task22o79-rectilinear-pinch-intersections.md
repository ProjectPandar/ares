# Task 22O.79 architecture decision record

## Status

Accepted for implementation. Decision date: 2026-08-13.

## Decision

Port pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/Fill/FillRectilinear.cpp:1216-1312`, into
`fill::rectilinear::pinch`.

Before monotonic-region generation, scan each noninitial vertical section for
an `InnerHigh` followed by `InnerLow` whose contour links do not connect the
pair. Insert a phony `OuterHigh` / `OuterLow` pair at the source midpoint and
reindex current, previous, and next section links exactly in source order.
Phony intersections own no contour or segment and invalid links.

Included: source scan/order, midpoint integer arithmetic, pair insertion,
current/neighbor reindexing, deterministic mutation, and no-op identity.
Deferred: monotonic region generation/chaining, complete filler/entity output,
lifecycle, and G-code. No legacy fallback or fixture branch.

Two O79 focused tests pass for no-op identity and ordered phony insertion; all
five O77/O78 regressions remain green. Strict core Clippy, rustfmt, diff, and
sub-400-LOC gates pass. The pinch shard is 114 LOC.
