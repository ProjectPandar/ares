# Upstream v2.5.0-beta inventory — Fill, surfaces, support and brim

Part 4 of the [baseline migration ARD](2026-10-30-upstream-baseline-v2.5.0-beta.md).
Planning base: Ares `e4a12268782154587e164a11bb67f09701599d3b`. Target: upstream
`345efec64c7b95a05ce5cf96df0dde66a158835c`; source baseline `v2.4.2` -> `v2.5.0-beta`.

All upstream paths below have prefix `src/libslic3r/`; statuses are Git `M`/`A`/`D`.
`C:` = `crates/ares-core/src/`; `V:` = `crates/ares-vgcode/src/`.
`partial port` identifies an existing bounded 2.4.2 rewrite/counterpart, NOT full-file
coverage or 2.5.0 qualification. `not ported` includes new files, native-only APIs
and helpers without an identified source-port owner; a similarly named type or
comment alone is not evidence of a port. Deleted files retain their old owner
when present. The fill/support owner is the proposed sole upstream-file owner; deferred rows
are tracking ownership, not approval to implement those features.

Rows: **84**. Every listed Rust path was checked at the planning base.

| Status | Upstream file | Current Ares port/counterpart | Scope / gap |
|---|---|---|---|
| A | `BeltBrim.cpp` | **not ported** | deferred belt execution; no current port |
| A | `BeltBrim.hpp` | **not ported** | deferred belt execution; no current port |
| M | `Brim.cpp` | partial port — `C:brims.rs`, `C:project_slice/gcode_emit/brim.rs` | bounded 2.4.2 slice |
| M | `Brim.hpp` | partial port — `C:brims.rs`, `C:project_slice/gcode_emit/brim.rs` | bounded 2.4.2 slice |
| M | `BrimEarsPoint.hpp` | partial port — `C:project/brim_ear_points.rs` | bounded 2.4.2 slice |
| M | `Feature/Interlocking/InterlockingGenerator.cpp` | partial port — `C:project_slice/region_slices/interlocking/mod.rs`, `C:project_slice/region_slices/interlocking/voxel.rs` | bounded 2.4.2 slice |
| M | `Feature/Interlocking/InterlockingGenerator.hpp` | partial port — `C:project_slice/region_slices/interlocking/mod.rs`, `C:project_slice/region_slices/interlocking/voxel.rs` | bounded 2.4.2 slice |
| M | `Feature/Interlocking/VoxelUtils.cpp` | partial port — `C:project_slice/region_slices/interlocking/mod.rs`, `C:project_slice/region_slices/interlocking/voxel.rs` | bounded 2.4.2 slice |
| M | `Feature/Interlocking/VoxelUtils.hpp` | partial port — `C:project_slice/region_slices/interlocking/mod.rs`, `C:project_slice/region_slices/interlocking/voxel.rs` | bounded 2.4.2 slice |
| A | `Fill/AdaptiveInfillField.cpp` | **not ported** | new beta file |
| A | `Fill/AdaptiveInfillField.hpp` | **not ported** | new beta file |
| M | `Fill/Fill.cpp` | partial port — `C:project_slice/group_fills.rs`, `C:project_slice/fill_entities.rs` | bounded 2.4.2 slice |
| M | `Fill/Fill.hpp` | partial port — `C:project_slice/group_fills.rs`, `C:project_slice/fill_entities.rs` | bounded 2.4.2 slice |
| M | `Fill/Fill3DHoneycomb.cpp` | partial port — `C:fill/three_d_honeycomb.rs` | bounded 2.4.2 slice |
| M | `Fill/Fill3DHoneycomb.hpp` | partial port — `C:fill/three_d_honeycomb.rs` | bounded 2.4.2 slice |
| M | `Fill/FillAdaptive.cpp` | partial port — `C:fill/adaptive/mod.rs`, `C:project_slice/fill_entities/adaptive.rs` | bounded 2.4.2 slice |
| M | `Fill/FillAdaptive.hpp` | partial port — `C:fill/adaptive/mod.rs`, `C:project_slice/fill_entities/adaptive.rs` | bounded 2.4.2 slice |
| M | `Fill/FillBase.cpp` | partial port — `C:fill.rs`, `C:fill/connect.rs` | bounded 2.4.2 slice |
| M | `Fill/FillBase.hpp` | partial port — `C:fill.rs`, `C:fill/connect.rs` | bounded 2.4.2 slice |
| M | `Fill/FillConcentric.cpp` | partial port — `C:fill/concentric_sparse.rs` | bounded 2.4.2 slice |
| M | `Fill/FillConcentric.hpp` | partial port — `C:fill/concentric_sparse.rs` | bounded 2.4.2 slice |
| M | `Fill/FillConcentricInternal.cpp` | partial port — `C:project_slice/fill_entities/concentric.rs` | bounded 2.4.2 slice |
| M | `Fill/FillConcentricInternal.hpp` | partial port — `C:project_slice/fill_entities/concentric.rs` | bounded 2.4.2 slice |
| A | `Fill/FillCornerSmoothing.cpp` | **not ported** | new beta file |
| A | `Fill/FillCornerSmoothing.hpp` | **not ported** | new beta file |
| M | `Fill/FillCrossHatch.cpp` | partial port — `C:fill/cross_hatch.rs` | bounded 2.4.2 slice |
| M | `Fill/FillCrossHatch.hpp` | partial port — `C:fill/cross_hatch.rs` | bounded 2.4.2 slice |
| M | `Fill/FillGyroid.cpp` | partial port — `C:fill/gyroid.rs` | bounded 2.4.2 slice |
| M | `Fill/FillGyroid.hpp` | partial port — `C:fill/gyroid.rs` | bounded 2.4.2 slice |
| M | `Fill/FillHoneycomb.cpp` | partial port — `C:fill/honeycomb.rs` | bounded 2.4.2 slice |
| M | `Fill/FillHoneycomb.hpp` | partial port — `C:fill/honeycomb.rs` | bounded 2.4.2 slice |
| M | `Fill/FillLightning.cpp` | partial port — `C:project_slice/fill_entities/lightning.rs` | bounded 2.4.2 slice |
| M | `Fill/FillLightning.hpp` | partial port — `C:project_slice/fill_entities/lightning.rs` | bounded 2.4.2 slice |
| M | `Fill/FillLine.cpp` | partial port — `C:fill/line.rs` | bounded 2.4.2 slice |
| M | `Fill/FillLine.hpp` | partial port — `C:fill/line.rs` | bounded 2.4.2 slice |
| M | `Fill/FillPlanePath.cpp` | partial port — `C:fill/plane_path.rs` | bounded 2.4.2 slice |
| M | `Fill/FillPlanePath.hpp` | partial port — `C:fill/plane_path.rs` | bounded 2.4.2 slice |
| M | `Fill/FillRectilinear.cpp` | partial port — `C:fill/rectilinear.rs`, `C:fill/multiline.rs` | bounded 2.4.2 slice |
| M | `Fill/FillRectilinear.hpp` | partial port — `C:fill/rectilinear.rs`, `C:fill/multiline.rs` | bounded 2.4.2 slice |
| A | `Fill/FillSpiralInset.cpp` | **not ported** | new beta file |
| A | `Fill/FillSpiralInset.hpp` | **not ported** | new beta file |
| M | `Fill/FillTpmsD.cpp` | partial port — `C:fill/tpms_d.rs` | bounded 2.4.2 slice |
| M | `Fill/FillTpmsD.hpp` | partial port — `C:fill/tpms_d.rs` | bounded 2.4.2 slice |
| M | `Fill/FillTpmsFK.cpp` | partial port — `C:fill/tpms_fk.rs` | bounded 2.4.2 slice |
| M | `Fill/FillTpmsFK.hpp` | partial port — `C:fill/tpms_fk.rs` | bounded 2.4.2 slice |
| M | `Fill/Lightning/DistanceField.cpp` | partial port — `C:fill/lightning/distance_field.rs` | bounded 2.4.2 slice |
| M | `Fill/Lightning/DistanceField.hpp` | partial port — `C:fill/lightning/distance_field.rs` | bounded 2.4.2 slice |
| M | `Fill/Lightning/Generator.cpp` | partial port — `C:fill/lightning/generator.rs` | bounded 2.4.2 slice |
| M | `Fill/Lightning/Generator.hpp` | partial port — `C:fill/lightning/generator.rs` | bounded 2.4.2 slice |
| M | `Fill/Lightning/Layer.cpp` | partial port — `C:fill/lightning/layer.rs` | bounded 2.4.2 slice |
| M | `Fill/Lightning/Layer.hpp` | partial port — `C:fill/lightning/layer.rs` | bounded 2.4.2 slice |
| M | `Fill/Lightning/TreeNode.cpp` | partial port — `C:fill/lightning/tree_node.rs` | bounded 2.4.2 slice |
| M | `Fill/Lightning/TreeNode.hpp` | partial port — `C:fill/lightning/tree_node.rs` | bounded 2.4.2 slice |
| M | `Flow.cpp` | partial port — `C:project_slice/perimeters/flow.rs`, `C:extrusions/options.rs` | bounded 2.4.2 slice |
| M | `Flow.hpp` | partial port — `C:project_slice/perimeters/flow.rs`, `C:extrusions/options.rs` | bounded 2.4.2 slice |
| M | `PrintObject.cpp` | partial port — `C:project_slice/prepare_infill.rs`, `C:project_slice/support_material.rs` | bounded 2.4.2 slice |
| M | `PrintObjectSlice.cpp` | partial port — `C:project_slice/region_slices.rs`, `C:project_slice/compensation.rs` | bounded 2.4.2 slice |
| M | `Slicing.cpp` | partial port — `C:project_slice/planning.rs`, `C:project_slice/parameters.rs` | bounded 2.4.2 slice |
| M | `Slicing.hpp` | partial port — `C:project_slice/planning.rs`, `C:project_slice/parameters.rs` | bounded 2.4.2 slice |
| M | `SlicingAdaptive.cpp` | **not ported** | no identified current port |
| M | `SlicingAdaptive.hpp` | **not ported** | no identified current port |
| A | `Support/BeltFloorContext.cpp` | **not ported** | deferred belt execution; no current port |
| A | `Support/BeltFloorContext.hpp` | **not ported** | deferred belt execution; no current port |
| M | `Support/SupportCommon.cpp` | partial port — `C:project_slice/support_material/layer_index.rs` | bounded 2.4.2 slice |
| M | `Support/SupportCommon.hpp` | partial port — `C:project_slice/support_material/layer_index.rs` | bounded 2.4.2 slice |
| M | `Support/SupportLayer.hpp` | partial port — `C:project_slice/support_material/install_layers.rs` | bounded 2.4.2 slice |
| M | `Support/SupportMaterial.cpp` | partial port — `C:project_slice/support_material.rs` | bounded 2.4.2 slice |
| M | `Support/SupportMaterial.hpp` | partial port — `C:project_slice/support_material.rs` | bounded 2.4.2 slice |
| M | `Support/SupportParameters.hpp` | partial port — `C:project_slice/support_material/interface_counts.rs` | bounded 2.4.2 slice |
| M | `Support/SupportSpotsGenerator.cpp` | partial port — `C:project_slice/curl_estimation.rs` | bounded 2.4.2 slice |
| M | `Support/SupportSpotsGenerator.hpp` | partial port — `C:project_slice/curl_estimation.rs` | bounded 2.4.2 slice |
| M | `Support/TreeModelVolumes.cpp` | partial port — `C:project_slice/support_material/tree/model_volumes.rs` | bounded 2.4.2 slice |
| M | `Support/TreeModelVolumes.hpp` | partial port — `C:project_slice/support_material/tree/model_volumes.rs` | bounded 2.4.2 slice |
| M | `Support/TreeSupport.cpp` | partial port — `C:project_slice/support_material/tree/non_organic.rs` | bounded 2.4.2 slice |
| M | `Support/TreeSupport.hpp` | partial port — `C:project_slice/support_material/tree/non_organic.rs` | bounded 2.4.2 slice |
| M | `Support/TreeSupport3D.cpp` | partial port — `C:project_slice/support_material/tree/organic.rs` | bounded 2.4.2 slice |
| M | `Support/TreeSupport3D.hpp` | partial port — `C:project_slice/support_material/tree/organic.rs` | bounded 2.4.2 slice |
| M | `Support/TreeSupportCommon.hpp` | partial port — `C:project_slice/support_material/tree/organic/interface_placer.rs` | bounded 2.4.2 slice |
| M | `Surface.cpp` | partial port — `C:surface.rs` | bounded 2.4.2 slice |
| M | `Surface.hpp` | partial port — `C:surface.rs` | bounded 2.4.2 slice |
| M | `SurfaceCollection.cpp` | partial port — `C:surface.rs` | bounded 2.4.2 slice |
| M | `SurfaceCollection.hpp` | partial port — `C:surface.rs` | bounded 2.4.2 slice |
| M | `TriangulateWall.cpp` | **not ported** | no identified current port |
| M | `TriangulateWall.hpp` | **not ported** | no identified current port |
