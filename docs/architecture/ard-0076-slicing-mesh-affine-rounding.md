# ARD-0076: Slicing mesh affine rounding

Upstream boundary: `TriangleMeshSlicer.cpp::transform_mesh_vertices_for_slicing`
and its Eigen `Transform3f * Vec3f` arithmetic. Rust destinations are
`project::Transform3d::transform_point_f32` and its existing
`project_slice/raw_intersections.rs::transformed_vertices` consumer. The
project-slice adapter is an integration shell around the upstream mesh boundary,
not an independent pipeline. Do not change f64 transform composition or import.

At investigation entry the committed Rust f32 helper explicitly fused
multiply-adds. Initialized-reference mesh producer capture and disassembly
confirmed separately rounded X/Y/Z products and sums, with translation last.
Preserve native arithmetic independent of host FMA support; do not add a fixture
branch, alternate precision path or a tolerance to hide one-scaled-unit drift.

Include only demonstrated f32 mesh-transform arithmetic and directly affected
consumer expectations. Defer unrelated Arachne short-path rounding and M73
insertion. A compressed polygon public complete-output witness and the original
KSR diagnostic must show what changes; local success does not close global parity.
