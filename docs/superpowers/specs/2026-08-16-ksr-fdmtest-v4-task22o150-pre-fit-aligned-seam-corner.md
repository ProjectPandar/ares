# Spec: Task 22O.150 pre-fit aligned seam corner

## Observable contract

On the first KSR layer, the second retract-and-wipe transition emits the exact source spiral lift `G3 Z.6 I1.188 J-.264 P1  F60000` and then travels to the outer-wall seam with `G1 X145.539 Y94.166 Z.6`. The target is selected from generated perimeter geometry and the loaded aligned-seam configuration; production code must not inspect fixture names or golden G-code.

## Upstream boundary

This slice ports the relevant behavior from OrcaSlicer 2.4.2 `src/libslic3r/ArcFitter.cpp:9-150`, `src/libslic3r/GCode/SeamPlacer.cpp:405-447,1014-1175`, and `src/libslic3r/ExtrusionEntity.cpp:182-315`.

Included behavior:

- seam candidate extraction retains the perimeter points that exist before Ares materializes reduced fitted ranges;
- sibling-perimeter locality remains anchored by the nearest source-acceptable candidate, while the best-scoring corner within one flow-width feature is retained;
- transient candidate geometry is moved rather than copied and is released immediately after seam placement;
- fitted-range projection, spiral-lift direction, and outer-wall travel consume the resulting exact seam point.

Deferred behavior: object identifier parity, printing-time processing, cooling, and later executable G-code differences.

## Acceptance

The focused `slice_project` contract asserts the exact lift arc and following outer-wall XYZ travel. Seam-placement and retained-arc tests remain green, the changed core crate passes Clippy and rustfmt, and every changed Rust source remains below 400 lines.
