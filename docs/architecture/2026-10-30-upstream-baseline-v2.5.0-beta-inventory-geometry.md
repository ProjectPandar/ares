# Upstream v2.5.0-beta inventory — Geometry, mesh and model import

Part 2 of the [baseline migration ARD](2026-10-30-upstream-baseline-v2.5.0-beta.md).
Planning base: Ares `e4a12268782154587e164a11bb67f09701599d3b`. Target: upstream
`345efec64c7b95a05ce5cf96df0dde66a158835c`; source baseline `v2.4.2` -> `v2.5.0-beta`.

All upstream paths below have prefix `src/libslic3r/`; statuses are Git `M`/`A`/`D`.
`C:` = `crates/ares-core/src/`; `V:` = `crates/ares-vgcode/src/`.
`partial port` identifies an existing bounded 2.4.2 rewrite/counterpart, NOT full-file
coverage or 2.5.0 qualification. `not ported` includes new files, native-only APIs
and helpers without an identified source-port owner; a similarly named type or
comment alone is not evidence of a port. Deleted files retain their old owner
when present. The geometry owner is the proposed sole upstream-file owner; deferred rows
are tracking ownership, not approval to implement those features.

Rows: **269**. Every listed Rust path was checked at the planning base.

| Status | Upstream file | Current Ares port/counterpart | Scope / gap |
|---|---|---|---|
| M | `AABBMesh.cpp` | **not ported** | no identified current port |
| M | `AABBMesh.hpp` | **not ported** | no identified current port |
| M | `AABBTreeIndirect.hpp` | partial port — `C:project_slice/seam_placement/spatial/bvh.rs`, `C:project_slice/contour_z/tree.rs` | bounded ray/nearest traversal only |
| M | `AABBTreeLines.hpp` | partial port — `C:geometry/line_distance_tree.rs` | bounded segment-distance queries |
| M | `AStar.hpp` | **not ported** | no identified current port |
| M | `Algorithm/LineSplit.cpp` | partial port — `C:geometry/line_split.rs` | bounded 2.4.2 slice |
| M | `Algorithm/LineSplit.hpp` | partial port — `C:geometry/line_split.rs` | bounded 2.4.2 slice |
| M | `Algorithm/RegionExpansion.cpp` | partial port — `C:geometry/region_expansion.rs` | bounded 2.4.2 slice |
| M | `Algorithm/RegionExpansion.hpp` | partial port — `C:geometry/region_expansion.rs` | bounded 2.4.2 slice |
| M | `Arrange.cpp` | **not ported** | no identified current port |
| M | `Arrange.hpp` | **not ported** | no identified current port |
| A | `BeltSliceStrategy.cpp` | **not ported** | deferred belt execution; no current port |
| A | `BeltSliceStrategy.hpp` | **not ported** | deferred belt execution; no current port |
| A | `BeltTransform.cpp` | **not ported** | deferred belt execution; no current port |
| A | `BeltTransform.hpp` | **not ported** | deferred belt execution; no current port |
| M | `BoundingBox.cpp` | partial port — `C:geometry/bounding_box.rs` | bounded 2.4.2 slice |
| M | `BoundingBox.hpp` | partial port — `C:geometry/bounding_box.rs` | bounded 2.4.2 slice |
| M | `BuildVolume.cpp` | partial port — `C:project_slice/state/print_volume.rs` | bounded 2.4.2 slice |
| M | `BuildVolume.hpp` | partial port — `C:project_slice/state/print_volume.rs` | bounded 2.4.2 slice |
| A | `CAD/CadDocument.cpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `CAD/CadDocument.hpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `CAD/GeometryEngine.cpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `CAD/GeometryEngine.hpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `CAD/SketchConstraints.cpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `CAD/SketchConstraints.hpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `CAD/SketchEngine.cpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `CAD/SketchEngine.hpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `CAD/SketchImport.cpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `CAD/SketchImport.hpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `CAD/SketchInference.cpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `CAD/SketchInference.hpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `CAD/SketchSolver.cpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `CAD/SketchSolver.hpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `CAD/ThreadStandards.cpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `CAD/ThreadStandards.hpp` | **not ported** | deferred CAD/texture execution; no current port |
| M | `CSGMesh/CSGMesh.hpp` | **not ported** | no identified current port |
| M | `CSGMesh/CSGMeshCopy.hpp` | **not ported** | no identified current port |
| M | `CSGMesh/ModelToCSGMesh.hpp` | **not ported** | no identified current port |
| M | `CSGMesh/PerformCSGMeshBooleans.hpp` | **not ported** | no identified current port |
| M | `CSGMesh/SliceCSGMesh.hpp` | **not ported** | no identified current port |
| M | `CSGMesh/TriangleMeshAdapter.hpp` | **not ported** | no identified current port |
| D | `CSGMesh/VoxelizeCSGMesh.hpp` | **not ported** | deleted upstream; no existing port to retire |
| D | `Clipper2Utils.cpp` | **not ported** | deleted upstream; no existing port to retire |
| D | `Clipper2Utils.hpp` | **not ported** | deleted upstream; no existing port to retire |
| D | `Clipper2ZUtils.hpp` | **not ported** | deleted upstream; no existing port to retire |
| M | `ClipperUtils.cpp` | partial port — `C:geometry/clipper.rs`, `C:geometry/bbox_clip.rs` | Clipper1-based wrapper counterpart; NOT a Clipper2 port |
| M | `ClipperUtils.hpp` | partial port — `C:geometry/clipper.rs`, `C:geometry/bbox_clip.rs` | Clipper1-based wrapper counterpart; NOT a Clipper2 port |
| A | `ClipperZUtils.cpp` | **not ported** | new beta file |
| M | `ClipperZUtils.hpp` | partial port — `C:geometry/clipper/z_clip.rs` | legacy Clipper1 Z-clipping counterpart; new beta .cpp/backend NOT ported |
| A | `ColorDecomposeRecipe.cpp` | **not ported** | new beta file |
| A | `ColorDecomposeRecipe.hpp` | **not ported** | new beta file |
| A | `ConnectedBodies.cpp` | **not ported** | new beta file |
| A | `ConnectedBodies.hpp` | **not ported** | new beta file |
| M | `ContourZ.cpp` | partial port — `C:project_slice/contour_z.rs` | bounded 2.4.2 slice |
| M | `CutSurface.cpp` | **not ported** | no identified current port |
| M | `CutSurface.hpp` | **not ported** | no identified current port |
| M | `CutUtils.cpp` | **not ported** | no identified current port |
| M | `CutUtils.hpp` | **not ported** | no identified current port |
| M | `EdgeGrid.cpp` | partial port — `C:geometry/edge_grid.rs` | bounded 2.4.2 slice |
| M | `EdgeGrid.hpp` | partial port — `C:geometry/edge_grid.rs` | bounded 2.4.2 slice |
| M | `ElephantFootCompensation.cpp` | partial port — `C:project_slice/elephant_foot.rs` | bounded 2.4.2 slice |
| M | `ElephantFootCompensation.hpp` | partial port — `C:project_slice/elephant_foot.rs` | bounded 2.4.2 slice |
| M | `Emboss.cpp` | **not ported** | no identified current port |
| M | `Emboss.hpp` | **not ported** | no identified current port |
| M | `EmbossShape.hpp` | **not ported** | no identified current port |
| M | `ExPolygon.cpp` | partial port — `C:geometry/expolygon.rs` | bounded geometry/model operations |
| M | `ExPolygon.hpp` | partial port — `C:geometry/expolygon.rs` | bounded geometry/model operations |
| M | `ExPolygonCollection.hpp` | partial port — `C:geometry/expolygon.rs` | bounded geometry/model operations |
| M | `ExPolygonSerialize.hpp` | **not ported** | no identified current port |
| M | `ExPolygonsIndex.cpp` | **not ported** | no identified current port |
| M | `ExPolygonsIndex.hpp` | **not ported** | no identified current port |
| M | `Exception.hpp` | partial port — `C:lib.rs` | SliceError public-boundary compatibility shell, not C++ exception ABI |
| A | `ExclusionVolumeGeometry.cpp` | **not ported** | new beta file |
| A | `ExclusionVolumeGeometry.hpp` | **not ported** | new beta file |
| M | `Execution/Execution.hpp` | **not ported** | no identified current port |
| M | `Execution/ExecutionSeq.hpp` | **not ported** | no identified current port |
| M | `Execution/ExecutionTBB.hpp` | **not ported** | no identified current port |
| M | `ExtrusionSimulator.cpp` | **not ported** | no identified current port |
| M | `ExtrusionSimulator.hpp` | **not ported** | no identified current port |
| M | `FaceDetector.cpp` | **not ported** | no identified current port |
| M | `FaceDetector.hpp` | **not ported** | no identified current port |
| M | `FileParserError.hpp` | **not ported** | no identified current port |
| M | `Format/3mf.cpp` | partial port — `C:project/load.rs`, `C:project/archive.rs` | bounded byte-oriented project/model import; new beta resources/types not ported |
| M | `Format/3mf.hpp` | partial port — `C:project/load.rs`, `C:project/archive.rs` | bounded byte-oriented project/model import; new beta resources/types not ported |
| M | `Format/AMF.cpp` | **not ported** | no identified current port |
| M | `Format/AMF.hpp` | **not ported** | no identified current port |
| A | `Format/AssembleList.cpp` | **not ported** | new beta file |
| A | `Format/AssembleList.hpp` | **not ported** | new beta file |
| A | `Format/AssimpImport.cpp` | **not ported** | new beta file |
| A | `Format/AssimpImport.hpp` | **not ported** | new beta file |
| M | `Format/DRC.cpp` | **not ported** | no identified current port |
| M | `Format/ModelIO.hpp` | **not ported** | no identified current port |
| M | `Format/OBJ.cpp` | **not ported** | no identified current port |
| M | `Format/OBJ.hpp` | **not ported** | no identified current port |
| A | `Format/ProjectStrategy.hpp` | **not ported** | new beta file |
| A | `Format/ResourcePathUtils.hpp` | **not ported** | new beta file |
| M | `Format/SL1.cpp` | **not ported** | no identified current port |
| M | `Format/SL1.hpp` | **not ported** | no identified current port |
| M | `Format/STEP.cpp` | **not ported** | no identified current port |
| M | `Format/STEP.hpp` | **not ported** | no identified current port |
| M | `Format/STL.cpp` | partial port — `C:stl.rs`, `C:project/domain/profile_input/stl_mesh.rs` | bounded 2.4.2 slice |
| M | `Format/ZipperArchiveImport.cpp` | **not ported** | no identified current port |
| M | `Format/ZipperArchiveImport.hpp` | **not ported** | no identified current port |
| M | `Format/bbs_3mf.cpp` | partial port — `C:project/load.rs`, `C:project/archive.rs` | bounded byte-oriented project/model import; new beta resources/types not ported |
| M | `Format/bbs_3mf.hpp` | partial port — `C:project/load.rs`, `C:project/archive.rs` | bounded byte-oriented project/model import; new beta resources/types not ported |
| M | `Format/objparser.cpp` | **not ported** | no identified current port |
| M | `Format/objparser.hpp` | **not ported** | no identified current port |
| M | `Format/svg.cpp` | **not ported** | no identified current port |
| M | `Format/svg.hpp` | **not ported** | no identified current port |
| M | `Geometry.cpp` | partial port — `C:geometry.rs`, `C:geometry/coord.rs` | bounded 2.4.2 slice |
| M | `Geometry.hpp` | partial port — `C:geometry.rs`, `C:geometry/coord.rs` | bounded 2.4.2 slice |
| M | `Geometry/ArcWelder.hpp` | partial port — `C:project_slice/gcode_emit/processor/motion/arc.rs` | bounded 2.4.2 slice |
| M | `Geometry/Bicubic.hpp` | **not ported** | no identified current port |
| M | `Geometry/Circle.cpp` | **not ported** | no identified current port |
| M | `Geometry/Circle.hpp` | **not ported** | no identified current port |
| M | `Geometry/ConvexHull.cpp` | partial port — `C:geometry/convex_hull.rs` | bounded 2.4.2 slice |
| M | `Geometry/ConvexHull.hpp` | partial port — `C:geometry/convex_hull.rs` | bounded 2.4.2 slice |
| M | `Geometry/Curves.hpp` | partial port — `C:project_slice/seam_placement/spline/qr.rs` | bounded spline/QR routines |
| M | `Geometry/MedialAxis.cpp` | partial port — `C:geometry/medial_axis.rs` | bounded 2.4.2 slice |
| M | `Geometry/Voronoi.cpp` | partial port — `C:geometry/voronoi/mod.rs` | bounded 2.4.2 slice |
| M | `Geometry/Voronoi.hpp` | partial port — `C:geometry/voronoi/mod.rs` | bounded 2.4.2 slice |
| M | `Geometry/VoronoiOffset.cpp` | **not ported** | no identified current port |
| M | `Geometry/VoronoiOffset.hpp` | **not ported** | no identified current port |
| M | `Geometry/VoronoiUtils.cpp` | partial port — `C:arachne/trapezoidation/voronoi.rs` | bounded graph/discretization utilities |
| M | `Geometry/VoronoiUtils.hpp` | partial port — `C:arachne/trapezoidation/voronoi.rs` | bounded graph/discretization utilities |
| M | `Geometry/VoronoiUtilsCgal.cpp` | partial port — `C:geometry/voronoi/predicates/mod.rs`, `C:geometry/voronoi/planarity.rs` | filtered/exact predicates, not CGAL runtime |
| M | `Geometry/VoronoiUtilsCgal.hpp` | partial port — `C:geometry/voronoi/predicates/mod.rs`, `C:geometry/voronoi/planarity.rs` | filtered/exact predicates, not CGAL runtime |
| M | `Geometry/VoronoiVisualUtils.hpp` | **not ported** | no identified current port |
| A | `IMEXArrange.cpp` | **not ported** | new beta file |
| A | `IMEXArrange.hpp` | **not ported** | new beta file |
| A | `IMEXHelpers.cpp` | **not ported** | new beta file |
| A | `IMEXHelpers.hpp` | **not ported** | new beta file |
| A | `IMEXZones.cpp` | **not ported** | new beta file |
| A | `IMEXZones.hpp` | **not ported** | new beta file |
| A | `InstanceLock.cpp` | **not ported** | new beta file |
| A | `InstanceLock.hpp` | **not ported** | new beta file |
| M | `Int128.hpp` | partial port — `C:geometry/clipper/predicates.rs` | wide products use Rust i128; no native class ABI |
| M | `IntersectionPoints.cpp` | **not ported** | no identified current port |
| M | `IntersectionPoints.hpp` | **not ported** | no identified current port |
| D | `JumpPointSearch.cpp` | **not ported** | deleted upstream; no existing port to retire |
| D | `JumpPointSearch.hpp` | **not ported** | deleted upstream; no existing port to retire |
| M | `KDTreeIndirect.hpp` | partial port — `C:project_slice/seam_placement/kdtree.rs` | bounded 2.4.2 slice |
| A | `LayOnFace.cpp` | **not ported** | new beta file |
| A | `LayOnFace.hpp` | **not ported** | new beta file |
| M | `Line.cpp` | partial port — `C:geometry/line.rs` | bounded 2.4.2 slice |
| M | `Line.hpp` | partial port — `C:geometry/line.rs` | bounded 2.4.2 slice |
| M | `LocalesUtils.cpp` | **not ported** | no identified current port |
| M | `LocalesUtils.hpp` | **not ported** | no identified current port |
| M | `MarchingSquares.hpp` | partial port — `C:geometry/marching_squares.rs` | bounded 2.4.2 slice |
| M | `Measure.cpp` | **not ported** | no identified current port |
| M | `Measure.hpp` | **not ported** | no identified current port |
| M | `MeasureUtils.hpp` | **not ported** | no identified current port |
| M | `MeshBoolean.cpp` | **not ported** | no identified current port |
| M | `MeshBoolean.hpp` | **not ported** | no identified current port |
| A | `MeshRepair.cpp` | **not ported** | new beta file |
| A | `MeshRepair.hpp` | **not ported** | new beta file |
| M | `MeshSplitImpl.hpp` | partial port — `C:project_slice/seam_placement/short_edge_collapse/topology.rs` | bounded mesh topology helpers |
| M | `MinAreaBoundingBox.cpp` | **not ported** | no identified current port |
| M | `MinimumSpanningTree.cpp` | partial port — `C:project_slice/support_material/tree/non_organic/spanning_tree.rs` | bounded 2.4.2 slice |
| M | `Model.cpp` | partial port — `C:project/domain.rs`, `C:project/load.rs` | bounded model/config import; no CAD/texture model rewrite |
| M | `Model.hpp` | partial port — `C:project/domain.rs`, `C:project/load.rs` | bounded model/config import; no CAD/texture model rewrite |
| M | `ModelArrange.cpp` | **not ported** | no identified current port |
| M | `ModelArrange.hpp` | **not ported** | no identified current port |
| M | `MultiMaterialSegmentation.cpp` | partial port — `C:project_slice/region_slices/painted/mod.rs` | bounded 2.4.2 slice |
| M | `MultiMaterialSegmentation.hpp` | partial port — `C:project_slice/region_slices/painted/mod.rs` | bounded 2.4.2 slice |
| A | `MultiNozzleUtils.cpp` | **not ported** | new beta file |
| A | `MultiNozzleUtils.hpp` | **not ported** | new beta file |
| M | `MultiPoint.cpp` | partial port — `C:geometry/polygon.rs`, `C:geometry/polyline.rs` | bounded 2.4.2 slice |
| M | `MultiPoint.hpp` | partial port — `C:geometry/polygon.rs`, `C:geometry/polyline.rs` | bounded 2.4.2 slice |
| M | `MutablePolygon.cpp` | partial port — `C:geometry/mutable_polygon.rs` | bounded 2.4.2 slice |
| M | `MutablePolygon.hpp` | partial port — `C:geometry/mutable_polygon.rs` | bounded 2.4.2 slice |
| M | `MutablePriorityQueue.hpp` | **not ported** | no identified current port |
| M | `NSVGUtils.cpp` | **not ported** | no identified current port |
| M | `NSVGUtils.hpp` | **not ported** | no identified current port |
| M | `NormalUtils.cpp` | partial port — `C:project_slice/seam_placement/short_edge_collapse/topology.rs` | bounded mesh topology helpers |
| M | `NormalUtils.hpp` | partial port — `C:project_slice/seam_placement/short_edge_collapse/topology.rs` | bounded mesh topology helpers |
| M | `ObjColorUtils.cpp` | **not ported** | no identified current port |
| M | `ObjColorUtils.hpp` | **not ported** | no identified current port |
| M | `ObjectID.cpp` | partial port — `C:project/domain/profile_input/stl.rs` | bounded model identity allocation, not native process globals |
| M | `ObjectID.hpp` | partial port — `C:project/domain/profile_input/stl.rs` | bounded model identity allocation, not native process globals |
| M | `OpenVDBUtils.cpp` | **not ported** | no identified current port |
| M | `OpenVDBUtils.hpp` | **not ported** | no identified current port |
| M | `Optimize/BruteforceOptimizer.hpp` | **not ported** | no identified current port |
| M | `Optimize/NLoptOptimizer.hpp` | **not ported** | no identified current port |
| M | `Optimize/Optimizer.hpp` | **not ported** | no identified current port |
| M | `Orient.cpp` | **not ported** | no identified current port |
| M | `Orient.hpp` | **not ported** | no identified current port |
| M | `Point.cpp` | partial port — `C:geometry/coord.rs` | bounded 2.4.2 slice |
| M | `Point.hpp` | partial port — `C:geometry/coord.rs` | bounded 2.4.2 slice |
| M | `Polygon.cpp` | partial port — `C:geometry/polygon.rs`, `C:geometry/simplification.rs` | bounded 2.4.2 slice |
| M | `Polygon.hpp` | partial port — `C:geometry/polygon.rs`, `C:geometry/simplification.rs` | bounded 2.4.2 slice |
| M | `PolygonTrimmer.cpp` | **not ported** | no identified current port |
| M | `PolygonTrimmer.hpp` | **not ported** | no identified current port |
| M | `Polyline.cpp` | partial port — `C:geometry/polyline.rs` | bounded 2.4.2 slice |
| M | `Polyline.hpp` | partial port — `C:geometry/polyline.rs` | bounded 2.4.2 slice |
| M | `PrincipalComponents2D.cpp` | partial port — `C:geometry/bridge_direction/principal_components.rs` | bounded 2.4.2 slice |
| M | `PrincipalComponents2D.hpp` | partial port — `C:geometry/bridge_direction/principal_components.rs` | bounded 2.4.2 slice |
| M | `QuadricEdgeCollapse.cpp` | **not ported** | no identified current port |
| M | `QuadricEdgeCollapse.hpp` | **not ported** | no identified current port |
| M | `SLA/IndexedMesh.cpp` | partial port — `C:project_slice/contour_z/mesh.rs` | only reused indexed-mesh geometry; SLA pipeline not ported |
| M | `SLA/IndexedMesh.hpp` | partial port — `C:project_slice/contour_z/mesh.rs` | only reused indexed-mesh geometry; SLA pipeline not ported |
| D | `Shape/TextShape.cpp` | **not ported** | deleted upstream; no existing port to retire |
| D | `Shape/TextShape.hpp` | **not ported** | deleted upstream; no existing port to retire |
| M | `ShortEdgeCollapse.cpp` | partial port — `C:project_slice/seam_placement/short_edge_collapse/mod.rs` | bounded 2.4.2 slice |
| M | `ShortEdgeCollapse.hpp` | partial port — `C:project_slice/seam_placement/short_edge_collapse/mod.rs` | bounded 2.4.2 slice |
| M | `ShortestPath.cpp` | partial port — `C:geometry/chain_points.rs`, `C:project_slice/perimeters/classic/shortest_path.rs` | bounded 2.4.2 slice |
| M | `ShortestPath.hpp` | partial port — `C:geometry/chain_points.rs`, `C:project_slice/perimeters/classic/shortest_path.rs` | bounded 2.4.2 slice |
| M | `SlicesToTriangleMesh.cpp` | **not ported** | no identified current port |
| M | `SlicesToTriangleMesh.hpp` | **not ported** | no identified current port |
| M | `SurfaceMesh.hpp` | **not ported** | no identified current port |
| M | `Tesselate.cpp` | partial port — `C:geometry/tessellation.rs` | bounded 2.4.2 slice |
| M | `Tesselate.hpp` | partial port — `C:geometry/tessellation.rs` | bounded 2.4.2 slice |
| A | `TextureBake/TextureBakeDebug.cpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureBake/TextureBakeDebug.hpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureBake/TextureBakeDecimate.cpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureBake/TextureBakeDecimate.hpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureBake/TextureBakeDisplace.cpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureBake/TextureBakeDisplace.hpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureBake/TextureBakeFlip.cpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureBake/TextureBakeFlip.hpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureBake/TextureBakeIndex.cpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureBake/TextureBakeIndex.hpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureBake/TextureBakeMesh.cpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureBake/TextureBakeMesh.hpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureBake/TextureBakePipeline.cpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureBake/TextureBakePipeline.hpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureBake/TextureBakeRegularize.cpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureBake/TextureBakeRegularize.hpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureBake/TextureBakeRelocate.cpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureBake/TextureBakeRelocate.hpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureBake/TextureBakeRepair.cpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureBake/TextureBakeRepair.hpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureBake/TextureBakeSubdivide.cpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureBake/TextureBakeSubdivide.hpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureDisplacement.cpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureDisplacement.hpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TexturePainting.cpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TexturePainting.hpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureToColor/Callbacks.hpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureToColor/CgalUtils.hpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureToColor/ColorUtils.cpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureToColor/ColorUtils.hpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureToColor/Repair.hpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureToColor/TextureToColor.cpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureToColor/TextureToColor.hpp` | **not ported** | deferred CAD/texture execution; no current port |
| A | `TextureToColor/TriMesh.hpp` | **not ported** | deferred CAD/texture execution; no current port |
| M | `TriangleMesh.cpp` | partial port — `C:mesh_slicer/topology.rs`, `C:project/load/mesh_prepare.rs` | bounded mesh indexing/transforms; not all repair/boolean methods |
| M | `TriangleMesh.hpp` | partial port — `C:mesh_slicer/topology.rs`, `C:project/load/mesh_prepare.rs` | bounded mesh indexing/transforms; not all repair/boolean methods |
| M | `TriangleMeshDeal.cpp` | **not ported** | no identified current port |
| M | `TriangleMeshDeal.hpp` | **not ported** | no identified current port |
| M | `TriangleMeshSlicer.cpp` | partial port — `C:mesh_slicer.rs`, `C:mesh_slicer/slabs.rs`, `C:mesh_slicer/chaining.rs` | bounded 2.4.2 slice |
| M | `TriangleMeshSlicer.hpp` | partial port — `C:mesh_slicer.rs`, `C:mesh_slicer/slabs.rs`, `C:mesh_slicer/chaining.rs` | bounded 2.4.2 slice |
| M | `TriangleSelector.cpp` | partial port — `C:project/facets_annotation/selector.rs` | bounded 2.4.2 slice |
| M | `TriangleSelector.hpp` | partial port — `C:project/facets_annotation/selector.rs` | bounded 2.4.2 slice |
| M | `TriangleSetSampling.cpp` | partial port — `C:project_slice/seam_placement/sampling.rs` | bounded 2.4.2 slice |
| M | `TriangleSetSampling.hpp` | partial port — `C:project_slice/seam_placement/sampling.rs` | bounded 2.4.2 slice |
| M | `Triangulation.cpp` | **not ported** | no identified current port |
| M | `Triangulation.hpp` | **not ported** | no identified current port |
| M | `Utils.hpp` | partial port — `C:slicer_context.rs`, `C:lib.rs` | only cancellation/error boundary subset; native utilities not generally ported |
| M | `Zipper.cpp` | **not ported** | no identified current port |
| M | `Zipper.hpp` | **not ported** | no identified current port |
| M | `calib.cpp` | **not ported** | no identified current port |
| M | `calib.hpp` | **not ported** | no identified current port |
| D | `clipper.cpp` | partial port — `C:geometry/clipper.rs` | legacy Clipper1 state machine exists; upstream deletes it in beta |
| D | `clipper.hpp` | partial port — `C:geometry/clipper.rs` | legacy Clipper1 state machine exists; upstream deletes it in beta |
| M | `libslic3r.h` | **not ported** | no identified current port |
| M | `miniz_extension.cpp` | **not ported** | no identified current port |
| M | `miniz_extension.hpp` | **not ported** | no identified current port |
| M | `utils.cpp` | **not ported** | no identified current port |
