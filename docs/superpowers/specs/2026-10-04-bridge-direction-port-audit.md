# Bridge direction WIP port audit

The upstream boundary is OrcaSlicer 2.4.2
`BridgeDetector.hpp::detect_bridging_direction(Lines, Polygons)`, called by
`LayerRegion.cpp::detect_bridge_directions`. Rust owns it in
`geometry/bridge_direction.rs` and the temporary external-surface adapter
`project_slice/prepare_infill/external_surfaces/detect_bridge_directions.rs`.
The adjacent slicing boundary is
`TriangleMeshSlicer.cpp::transform_mesh_vertices_for_slicing`, mapped to
`project::Transform3d::transform_point_f32`.

Include candidate hash/bucket iteration, rehash group reversal, strict minimum
selection and non-fused f32 slicing transforms from WIP `a53bc00c` only where
they are not already present. Exclude unrelated AGG/support-grid WIP changes,
Arachne outline changes and historical diagnostic patches. No fixture-specific
tie break, host-dependent Rust hash order or comparator relaxation is allowed.

Current main already contains these runtime changes (ARD-0075/0076). The
remaining task is an equivalence audit and fresh full-output regression, not
reapplying stale files or writing new source-pinning tests. Preserve existing
multi-region consumers. Prior historical RED/GREEN evidence belongs to the
original slice; fresh execution must be distinguished from that evidence.

Exit requires both equal-cost and rehash public complete-output cases plus
tilted/affine consumers, explicit native reference identity and full workspace,
lint/format/WASM gates. Other bridge geometry and global inventory completion
remain deferred.
