# Spec: Task 22O.148 retained pre-seam fitted arcs

## Observable contract

After the first KSR inter-path travel, the inner wall begins with the exact source command `G2 X145.766 Y96.281 I3.394 J-1.502 E.01821`. Subsequent fitted ranges continue from the same 3MF-derived perimeter without refitting already-simplified endpoint pairs.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `src/libslic3r/ArcFitter.cpp:9-150`, `src/libslic3r/Circle.cpp:276-488`, `src/libslic3r/Polyline.cpp:34-101,268-297,939-1005`, and `src/libslic3r/GCode.cpp:6991-7110`.

Included behavior:

- `Polyline3` retains line/arc fitting ranges produced before seam placement;
- path reversal, seam splitting, inserted split points, and terminal seam-gap clipping update that metadata;
- the emitter consumes retained arc centers, directions, and analytic lengths instead of attempting to infer arcs from two simplified endpoints;
- source tail-slice validation semantics are retained in the arc fitter.

All values come from fitted project geometry and effective options. Deferred behavior: exact positive near-zero I/J formatting, later perimeter ordering, cooling, timing, and remaining byte differences.

## Acceptance

The focused KSR test asserts the exact first retained G2 command. Arc fitter, seam placement, earlier range-boundary, and next-layer travel tests remain green; changed Rust files remain below 400 lines and pass Clippy and rustfmt.
