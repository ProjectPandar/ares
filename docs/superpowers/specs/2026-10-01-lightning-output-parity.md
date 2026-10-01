# Lightning complete-output parity

The identified production owner is
`Fill/Lightning/TreeNode.cpp:154-180::lineSegmentPolygonsIntersection`
and its edge-first call to `Geometry.hpp:92-120::segment_segment_intersection`.
Destination: `fill/lightning/tree_geom.rs::line_segment_polygons_intersection`.
Include restoring that argument order and a public full-stream regression,
without changing the existing Rust helper arithmetic, RNG or reference output.
This repairs argument order only, not the complete native call contract:
native passes endpoints where the helper accepts directions and measures
the previous intersection; the Rust shell passes direction differences
and measures the new intersection. These existing differences are deferred,
and the cube's matching output does not establish general equivalence.
The investigation also covered the existing upstream rewrite boundary
`Fill/Lightning/TreeNode.cpp::Node::convertToPolylines`,
`Fill/Lightning/Layer.cpp::Layer::convertToLines`,
`FillLightning.cpp::Filler::_fill_surface_single`, and
`PrintObject.cpp::bridge_over_infill` lightning anchoring. Destinations are
the existing `fill/lightning/` and `project_slice/fill_entities/lightning.rs`
and bridge-over-infill transaction modules, not a new Ares pipeline.

Include identifying and correcting the first deterministic lightning
motion divergence on the exported Ender-3 cube input. Preserve upstream
tree geometry, branch selection and stream-consumption behavior. Narrow
the owning source boundary from evidence before a production repair.
Existing scaffolding is a temporary shell around those upstream concepts.
Defer adjacent support-material and SLA ports, unproved numeric domains,
distance-field default join type and non-transitive stable-sort traversal.
Require active positive sparse-infill extrusion, not only an option string.

The approved test seam is public 3MF input and complete ordered output.
No semantic acceptance, tolerance, source-structure pinning or blanket
nondeterminism exemption may make a differing output pass. Retain original
RED input and both outputs; retain fresh native producer evidence.
Require witnessed RED/GREEN, separate test modules, files below400LOC,
workspace Nextest, fmt/Clippy, Tier-1 evidence and independent six-axis
review before committing code and documentation together.
