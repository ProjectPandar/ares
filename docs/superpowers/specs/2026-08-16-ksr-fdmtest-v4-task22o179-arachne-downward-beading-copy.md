# Spec: KSR FDM Test V4 task179 Arachne downward beading copy

## Observable contract

For an upward skeletal edge whose peak node has a final beading and whose lower node has none, downward propagation copies the peak beading to the lower node and adds the edge length to `dist_from_top_source`. The copied state is retained by the trapezoidation and remains non-upward-only.

A focused graph test observes the copied beading and propagation distance. Existing Arachne tests, workspace formatting, and Clippy remain clean.

## Upstream boundary

This slice ports the empty-lower-node branch of OrcaSlicer 2.4.2 `Arachne/SkeletalTrapezoidation.cpp:1614-1636`, `propagateBeadingsDownward(edge_t*, ...)`. Upward/downward merge, lazy source beading, radius-aware unequal-count interpolation, junction generation and connection, local maxima, Arachne concentric internal fill, cooling, timing, and remaining exact G-code differences are deferred.
