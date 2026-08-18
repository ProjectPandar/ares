# Spec: KSR FDM Test V4 task205 source circle-fit arithmetic

## Observable contract

Three-point arc fitting evaluates OrcaSlicer's `Circle::try_create_circle` expressions in source order on scaled integer-valued coordinates. Algebraically equivalent determinant formulas are not interchangeable because global coordinates exceed the exact-square range of `f64`; changed operation order moves fitted centers by microns, changing `I/J` words and fitted segment lengths.

A focused high-coordinate counterexample pins the source center `(151.545998, 102.773998)` where the rearranged formula produces `(151.545999, 102.773998)`. Fixture comparison must move the first non-metadata structural divergence beyond the prior `I-3.392` versus `I-3.393` arc. No debug instrumentation remains. Files stay below 400 LOC; focused arc tests, formatting, and Clippy remain clean.

## Upstream boundary

This slice ports literal arithmetic order from OrcaSlicer 2.4.2 `libslic3r/Circle.cpp:16-55` into `gcode_emit::motion::arc::circle_from_three`. It does not redesign fitting, tolerance, or emission behavior. Remaining outline nudging, infill count/order, timing, and G-code differences are deferred.
