# Slicing mesh affine rounding rewrite slice

Port source-owned `TriangleMeshSlicer.cpp::transform_mesh_vertices_for_slicing`
Eigen affine arithmetic into `project::Transform3d::transform_point_f32`, consumed
by the existing raw-intersection adapter shell. Keep f64 import/composition,
options, surface processing and G-code comparison unchanged. No Ares pipeline.

Use the agreed project-byte to complete-G-code seam and approved initialized
Orca reference. Start with the already validated stretched polygon fixture,
change only the model's X scale to0.99, and retain full paired streams. Verify
applied options. Minimize further if this does not reach mesh-transform drift.

Also compare a tilted 3MF through the same public seam: retain nonzero X/Z
and Y/Z mixing and XY translation in the exported project, then compare the
whole G-code. Ordinary walls keep its first layer printable; verify their
applied value rather than treating this as the wall-free bridge fixture.

Exit requires native/Rust producer evidence, a sensitive public RED/GREEN,
original KSR revalidation, consumer regression tests, fmt/clippy/Tier-1 gates,
and independent six-axis review. No source-layout pinning, output filtering or
new tolerance. The initialized and untouched native controls remain separate.
