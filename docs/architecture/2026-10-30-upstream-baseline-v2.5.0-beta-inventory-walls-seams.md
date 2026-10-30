# Upstream v2.5.0-beta inventory — Walls and seams

Part 3 of the [baseline migration ARD](2026-10-30-upstream-baseline-v2.5.0-beta.md).
Planning base: Ares `e4a12268782154587e164a11bb67f09701599d3b`. Target: upstream
`345efec64c7b95a05ce5cf96df0dde66a158835c`; source baseline `v2.4.2` -> `v2.5.0-beta`.

All upstream paths below have prefix `src/libslic3r/`; statuses are Git `M`/`A`/`D`.
`C:` = `crates/ares-core/src/`; `V:` = `crates/ares-vgcode/src/`.
`partial port` identifies an existing bounded 2.4.2 rewrite/counterpart, NOT full-file
coverage or 2.5.0 qualification. `not ported` includes new files, native-only APIs
and helpers without an identified source-port owner; a similarly named type or
comment alone is not evidence of a port. Deleted files retain their old owner
when present. The walls/seams owner is the proposed sole upstream-file owner; deferred rows
are tracking ownership, not approval to implement those features.

Rows: **48**. Every listed Rust path was checked at the planning base.

| Status | Upstream file | Current Ares port/counterpart | Scope / gap |
|---|---|---|---|
| M | `Arachne/BeadingStrategy/BeadingStrategy.cpp` | partial port — `C:arachne/beading/base.rs` | bounded 2.4.2 slice |
| M | `Arachne/BeadingStrategy/BeadingStrategyFactory.cpp` | partial port — `C:arachne/beading/factory.rs` | bounded 2.4.2 slice |
| M | `Arachne/BeadingStrategy/DistributedBeadingStrategy.cpp` | partial port — `C:arachne/beading/distributed.rs` | bounded 2.4.2 slice |
| M | `Arachne/BeadingStrategy/LimitedBeadingStrategy.cpp` | partial port — `C:arachne/beading/limited.rs` | bounded 2.4.2 slice |
| M | `Arachne/BeadingStrategy/OuterWallInsetBeadingStrategy.cpp` | partial port — `C:arachne/beading/outer_inset.rs` | bounded 2.4.2 slice |
| M | `Arachne/BeadingStrategy/RedistributeBeadingStrategy.cpp` | partial port — `C:arachne/beading/redistribute.rs` | bounded 2.4.2 slice |
| M | `Arachne/BeadingStrategy/WideningBeadingStrategy.cpp` | partial port — `C:arachne/beading/widening.rs` | bounded 2.4.2 slice |
| M | `Arachne/SkeletalTrapezoidation.cpp` | partial port — `C:arachne/trapezoidation.rs`, `C:arachne/trapezoidation/transitions.rs` | bounded 2.4.2 slice |
| M | `Arachne/SkeletalTrapezoidation.hpp` | partial port — `C:arachne/trapezoidation.rs`, `C:arachne/trapezoidation/transitions.rs` | bounded 2.4.2 slice |
| M | `Arachne/SkeletalTrapezoidationEdge.hpp` | partial port — `C:arachne/skeletal/payload.rs` | bounded 2.4.2 slice |
| M | `Arachne/SkeletalTrapezoidationGraph.cpp` | partial port — `C:arachne/skeletal/graph.rs` | bounded 2.4.2 slice |
| M | `Arachne/SkeletalTrapezoidationJoint.hpp` | partial port — `C:arachne/skeletal/payload.rs` | bounded 2.4.2 slice |
| M | `Arachne/WallToolPaths.cpp` | partial port — `C:arachne/wall_toolpaths.rs`, `C:arachne/wall_toolpaths/outline.rs` | bounded 2.4.2 slice |
| M | `Arachne/WallToolPaths.hpp` | partial port — `C:arachne/wall_toolpaths.rs`, `C:arachne/wall_toolpaths/outline.rs` | bounded 2.4.2 slice |
| M | `Arachne/utils/ExtrusionJunction.hpp` | partial port — `C:arachne/extrusion_line.rs` | bounded 2.4.2 slice |
| M | `Arachne/utils/ExtrusionLine.cpp` | partial port — `C:arachne/extrusion_line.rs` | bounded 2.4.2 slice |
| M | `Arachne/utils/ExtrusionLine.hpp` | partial port — `C:arachne/extrusion_line.rs` | bounded 2.4.2 slice |
| M | `Arachne/utils/PolygonsPointIndex.hpp` | **not ported** | no identified current port |
| M | `Arachne/utils/PolygonsSegmentIndex.hpp` | **not ported** | no identified current port |
| M | `Arachne/utils/PolylineStitcher.cpp` | partial port — `C:arachne/wall_toolpaths/stitch.rs` | bounded endpoint grid/stitch traversal; no general C++ container API |
| M | `Arachne/utils/PolylineStitcher.hpp` | partial port — `C:arachne/wall_toolpaths/stitch.rs` | bounded endpoint grid/stitch traversal; no general C++ container API |
| M | `Arachne/utils/SparseGrid.hpp` | partial port — `C:arachne/wall_toolpaths/stitch.rs` | bounded endpoint grid/stitch traversal; no general C++ container API |
| M | `Arachne/utils/SparseLineGrid.hpp` | **not ported** | no identified current port |
| M | `Arachne/utils/SparsePointGrid.hpp` | partial port — `C:arachne/wall_toolpaths/stitch.rs` | bounded endpoint grid/stitch traversal; no general C++ container API |
| M | `Arachne/utils/SquareGrid.cpp` | partial port — `C:arachne/wall_toolpaths/stitch.rs` | bounded endpoint grid/stitch traversal; no general C++ container API |
| M | `Arachne/utils/linearAlg2D.hpp` | partial port — `C:arachne/wall_toolpaths/outline.rs` | bounded 2D predicates, not general utility API |
| M | `BridgeDetector.cpp` | partial port — `C:project_slice/perimeters/classic/counterbore/detector.rs` | bounded 2.4.2 slice |
| M | `BridgeDetector.hpp` | partial port — `C:project_slice/perimeters/classic/counterbore/detector.rs` | bounded 2.4.2 slice |
| M | `ExtrusionEntity.cpp` | partial port — `C:extrusion_entity.rs` | bounded 2.4.2 slice |
| M | `ExtrusionEntity.hpp` | partial port — `C:extrusion_entity.rs` | bounded 2.4.2 slice |
| M | `ExtrusionEntityCollection.cpp` | partial port — `C:project_slice/perimeters/classic/entity_collections/types.rs`, `C:project_slice/support_material/extrusion_path/chain.rs` | bounded 2.4.2 slice |
| M | `ExtrusionEntityCollection.hpp` | partial port — `C:project_slice/perimeters/classic/entity_collections/types.rs`, `C:project_slice/support_material/extrusion_path/chain.rs` | bounded 2.4.2 slice |
| M | `Feature/FuzzySkin/FuzzySkin.cpp` | partial port — `C:perimeters/fuzzy_skin_noise.rs`, `C:perimeters/fuzzy_skin_coherent_noise.rs` | bounded 2.4.2 slice |
| M | `Feature/FuzzySkin/FuzzySkin.hpp` | partial port — `C:perimeters/fuzzy_skin_noise.rs`, `C:perimeters/fuzzy_skin_coherent_noise.rs` | bounded 2.4.2 slice |
| A | `GCode/PreciseSeam.cpp` | **not ported** | new beta file |
| A | `GCode/PreciseSeam.hpp` | **not ported** | new beta file |
| A | `GCode/PreciseSeamInternal.hpp` | **not ported** | new beta file |
| A | `GCode/SeamContour.cpp` | **not ported** | new beta file |
| A | `GCode/SeamContour.hpp` | **not ported** | new beta file |
| M | `GCode/SeamPlacer.cpp` | partial port — `C:project_slice/seam_placement.rs` | bounded 2.4.2 slice |
| M | `GCode/SeamPlacer.hpp` | partial port — `C:project_slice/seam_placement.rs` | bounded 2.4.2 slice |
| M | `Layer.cpp` | partial port — `C:project_slice/layers.rs`, `C:project_slice/slice_ordering.rs` | bounded 2.4.2 slice |
| M | `Layer.hpp` | partial port — `C:project_slice/layers.rs`, `C:project_slice/slice_ordering.rs` | bounded 2.4.2 slice |
| M | `LayerRegion.cpp` | partial port — `C:project_slice/perimeters/layer_region.rs`, `C:project_slice/fill_entities/simplify.rs` | bounded 2.4.2 slice |
| M | `PerimeterGenerator.cpp` | partial port — `C:project_slice/perimeters.rs`, `C:project_slice/perimeters/arachne.rs` | bounded 2.4.2 slice |
| M | `PerimeterGenerator.hpp` | partial port — `C:project_slice/perimeters.rs`, `C:project_slice/perimeters/arachne.rs` | bounded 2.4.2 slice |
| M | `VariableWidth.cpp` | partial port — `C:project_slice/perimeters/arachne/variable_width.rs` | bounded 2.4.2 slice |
| M | `VariableWidth.hpp` | partial port — `C:project_slice/perimeters/arachne/variable_width.rs` | bounded 2.4.2 slice |
