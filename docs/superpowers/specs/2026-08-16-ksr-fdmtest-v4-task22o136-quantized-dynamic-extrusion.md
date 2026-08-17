# Spec: Task 220.136 quantized dynamic-segment extrusion

## Observable contract

Variable-speed overhang extrusion length is measured between the same XYZF-quantized endpoints that are emitted to G-code. Unquantized state left by an adjacent materialized path must not perturb relative-E output when both paths meet at the same printed coordinate.

For the first KSR inner-wall segment after the first `Overhang wall`, the command is exactly `G1 X116.989 Y81.637 E.06303`.

## Upstream boundary

OrcaSlicer 2.4.2 `src/libslic3r/GCode.cpp:7123-7155` obtains every processed endpoint through `point_to_gcode_quantized` before calculating line length and extrusion. Ares mirrors that ordering in variable-speed emission while retaining its materialized-path seam.
