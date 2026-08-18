# Spec: KSR FDM Test V4 task208 rectilinear stick and small-contour cleanup

## Observable contract

Rectilinear source and offset contours remove zero-area turn-back sticks using OrcaSlicer's dot/cross predicate. Outer and inner offset paths with absolute area below `0.01 * inner_offset²` are discarded before scanline construction. Surviving path order is stable under in-place compaction.

A focused turn-back polygon test pins source point compaction. The KSR fixture retains its normalized one-micron arc endpoint after cleanup, excluding these branches as that mismatch's cause. Files remain below 400 LOC; rectilinear tests, formatting, and Clippy remain clean.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Polygon.cpp:532-594` and `Fill/FillRectilinear.cpp:416-429` into rectilinear contour preparation. It includes stick removal before and after offsets and source small-area filtering; arc input geometry, exact E, infill counts, timing, and remaining differences are deferred.
