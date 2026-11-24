# Organic branch circle sampling

OrcaSlicer `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`,
`src/libslic3r/Support/TreeSupport3D.cpp:2933–2955` (`discretize_circle`),
builds each ring vertex as `center + x * cos(angle) + y * sin(angle)`.
The angle starts at binary32 zero and is incremented in binary32 after each
sample. The pinned x86-64 reference calls binary32 `cosf` and `sinf` at
this boundary.

In `project_slice/support_material/tree/organic/branch_mesh/circle.rs`,
use the existing `scalar_math::sincosf::sin_cos` for those two results.
Keep the angle, its increment, the sample count, and the vertex arithmetic
unchanged. Do not change the scalar kernel, radius and angle-step derivation,
cap-local trig, or tree movement.

A one-ULP difference in cosine can move the unprinted endpoints of a support
extrusion without changing its three-decimal XY coordinates. On the synthetic
organic support ramp, that length difference changes the five-decimal E word.
A Chromium regression slices the ramp and compares its complete output with
runtime OrcaSlicer in the official and seeded lanes. Ignore exactly the one
slicer generator line; require positive extrusion in the support path and do
not filter, quantize, or reorder the remaining G-code.
