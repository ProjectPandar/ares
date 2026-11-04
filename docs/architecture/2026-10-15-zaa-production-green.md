# ZAA production contour integration

Boundaries: SPEC06a8, RED6d2b6f37a (carried as503dd63f7),
repair PLAN90381524 with its deterministic-tree supplement. Builds on main
29f4a49c18139925fec238d31d2fdb6b790fbc30, including the separate
Polyline3 mechanical migration. This feature chain keeps RED with its
production repair; it is not a standalone unused-kernel submission.

## Source ownership and production calls

PrintObject.cpp::need_z_contouring/contour_z:748-804 owns preparation after
fill/ironing, once per resolved object and excluding the first object layer.
project_slice::slice_source calls contour_z::prepare; that stage builds the
IndexedMesh/Tree3f once, then visits each physical layer's canonical region
ranges. Each region uses its own effective RegionOptions. There is no first-
region shortcut, shape whitelist or source-option rejection replacement.

ContourZ.cpp:162-243 owns the path/multipath/loop/collection traversal.
contour_z/entities.rs visits perimeter collections and region-owned fill
collections, including their ordinary/oriented and variable-width paths.
contour_z/path.rs applies the source eligible-role predicate,0.1mm samples,
IndexedMesh queries, face normals, height clamps and collinear reduction.
mesh::query calls ray::Query::first_hit; Tree::build calls partition selection.
All production kernels have this live caller chain. Tree construction and
ray traversal are serial, do not inspect available_parallelism or CPU count,
and do not use floating-point parallel reductions.

SLA/IndexedMesh.cpp:17-43/139-171 and AABBTreeIndirect.hpp:121-212/396-440
remain the indexed-mesh/query boundary, with f32 source vertices and hit
carrier, f64 query arithmetic, original face indices and source child order.
Existing Ares preparation wrappers are temporary compatibility carriers for
these upstream stages, not an independently designed contour pipeline.

## XYZ carrier and G-code boundary

ExtrusionEntity.cpp:33-47 bypasses ordinary simplification for contoured
paths. Both perimeter and fill paths now retain their Polyline3 XYZ and
z_contoured flag through simplify, reverse, seam split and clipping.
Polyline.cpp:843-888 preserves the prefix's preceding Z and suffix's next
Z when splitting at an interior XY projection. Polyline.cpp:683-705 clips
in scaled XYZ with truncating Eigen cast; loop length budgets use Line3.

GCode.cpp:6374-6400 sets the first contour Z and restores nominal Z for a
following ordinary path. The shared travel writer follows its actual Z-
change branch (GCodeWriter.cpp:771-806), instead of losing the requested
contour start during a same-layer approach. The existing retraction, origin,
extruder offset, role markers, speed, fan and writer state remain authoritative.

motion/path/contour.rs and the processed-point branch in path/variable.rs
port GCode.cpp:6992-7030/7220-7237: 3D length, nominal-plus-offset Z and
end-point height-scaled E; ironing retains its original E ratio. Contoured
paths bypass arc fitting. GCode.cpp:7025/7233's "GCode: very low z" export
exception is returned through the existing SliceError::Slicing boundary;
no partial G-code is returned as success. The bounded writer exception flag
is a compatibility carrier for C++ unwinding, not a new admission policy.

ExtrusionProcessor.hpp:85-180/366/457 retains Z in extended/processed points,
3D distance and segmentation, with the previous boundary lines at Z=0.
The original ordinary-Z=0 geometry arithmetic is retained for controls.

## Constructor correction and retained evidence

The first two-mode public replay retained disabled PASS but enabled DIVERGENT.
Assertions/comparator were not changed. A Native ABI probe and Ares debugger observation isolated a one-unit
coordinate error: Native(-2031758,-6352), Ares(-2031757,-6351). Native E was
0.0031749992803961334, just below the five-digit rounding boundary.

ContourZ.cpp:155 calls Point3(double,double,double), whose Point.hpp:280
constructor uses std::round, rather than the truncating Point3::new_scale
or Eigen cast. Polyline.cpp:783-790 drops adjacent identical Point3 values
on append. The repair uses rounded sampled coordinates and adjacent dedup;
this also preserves corners and removes the unintended duplicate wipe point.
This is source constructor behavior, not a tolerance or comparator change.

Only the superseded capabilities.rs zaa guard and compiler-proven local
helpers/imports are removed. The exact retirement patch SHA256
is a4df93e58be222f236201e3947b17a0a1bb8ea618e2002f74f82cd8a7d0419af;
its obsolete private assertion/helpers are retired in a separate test-only
commit after the production GREEN. The public RED stays unchanged.

Finite coverage is the admitted classic sloped-roof public API/CLI witness,
plus ordinary fill/perimeter directed controls and source kernel/motion tests.
Broader fixtures remain deferred as listed in the SPEC: Arachne,
modifier boundaries, multipart/instance transforms, nondefault minZ/angle,
fill-direction, ironing-specific and scarf/arc interactions. No generic
coverage claim replaces these missing witnesses. Native products are never
committed.
