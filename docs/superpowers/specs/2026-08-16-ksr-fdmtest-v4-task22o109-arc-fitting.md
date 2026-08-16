# Spec: KSR FDM v4 option-driven arc fitting

## Observable contract

When the project effective option `enable_arc_fitting` is true and `spiral_mode` is false, the project G-code seam emits circular path portions as Orca-compatible `G2`/`G3` moves. Linear portions remain `G1` moves. Arc extrusion uses the analytic arc length, while endpoint coordinates and I/J offsets use the same print-space formatter as linear motion. When the option is false, output remains linear.

The implementation must consume only typed 3MF options and generated path geometry. It must not inspect the fixture name, reference G-code, or expected counts.

## Upstream boundary

Port the behavior selected by `OrcaSlicer/src/libslic3r/Layer.cpp:348-390`, `ArcFitter.cpp:27-95`, `Circle.hpp:69-140`, and `GCode.cpp:6990-7109`. The Rust destination is the private project G-code emission module below `slice_project`.

Included: greedy arc fitting with the Orca 0.0125 mm output tolerance, 2000 mm maximum radius, 5% arc-length tolerance, and G2/G3 output for fitted arc segments. Deferred: z-contoured/sloped paths and DP simplification of non-arc portions until their source path metadata is carried through the Ares materialization seam.

## Acceptance

Focused tests prove option gating, clockwise/counter-clockwise selection, analytic arc length, and line fallback. The KSR CLI output contains source-derived G2/G3 moves without fixture-specific branches.
