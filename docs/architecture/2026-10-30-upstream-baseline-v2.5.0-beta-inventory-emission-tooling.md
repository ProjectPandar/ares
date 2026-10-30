# Upstream v2.5.0-beta inventory — G-code emission and material/tool planning

Part 5 of the [baseline migration ARD](2026-10-30-upstream-baseline-v2.5.0-beta.md).
Planning base: Ares `e4a12268782154587e164a11bb67f09701599d3b`. Target: upstream
`345efec64c7b95a05ce5cf96df0dde66a158835c`; source baseline `v2.4.2` -> `v2.5.0-beta`.

All upstream paths below have prefix `src/libslic3r/`; statuses are Git `M`/`A`/`D`.
`C:` = `crates/ares-core/src/`; `V:` = `crates/ares-vgcode/src/`.
`partial port` identifies an existing bounded 2.4.2 rewrite/counterpart, NOT full-file
coverage or 2.5.0 qualification. `not ported` includes new files, native-only APIs
and helpers without an identified source-port owner; a similarly named type or
comment alone is not evidence of a port. Deleted files retain their old owner
when present. The emission/tooling owner is the proposed sole upstream-file owner; deferred rows
are tracking ownership, not approval to implement those features.

Rows: **73**. Every listed Rust path was checked at the planning base.

| Status | Upstream file | Current Ares port/counterpart | Scope / gap |
|---|---|---|---|
| M | `ArcFitter.cpp` | partial port — `C:project_slice/gcode_emit/motion/arc.rs` | bounded fitting/arc geometry |
| M | `ArcFitter.hpp` | partial port — `C:project_slice/gcode_emit/motion/arc.rs` | bounded fitting/arc geometry |
| A | `BeltGCode.cpp` | **not ported** | deferred belt execution; no current port |
| A | `BeltGCode.hpp` | **not ported** | deferred belt execution; no current port |
| A | `BeltPurge.cpp` | **not ported** | deferred belt execution; no current port |
| M | `Circle.cpp` | partial port — `C:project_slice/gcode_emit/motion/arc.rs` | bounded fitting/arc geometry |
| M | `Circle.hpp` | partial port — `C:project_slice/gcode_emit/motion/arc.rs` | bounded fitting/arc geometry |
| M | `CustomGCode.cpp` | partial port — `C:project/plate_custom_gcodes.rs`, `C:project_slice/gcode_emit/layers/custom_gcode.rs` | bounded 2.4.2 slice |
| M | `CustomGCode.hpp` | partial port — `C:project/plate_custom_gcodes.rs`, `C:project_slice/gcode_emit/layers/custom_gcode.rs` | bounded 2.4.2 slice |
| M | `Extruder.cpp` | partial port — `C:project_slice/gcode_emit/motion/extruder_state.rs` | bounded 2.4.2 slice |
| M | `Extruder.hpp` | partial port — `C:project_slice/gcode_emit/motion/extruder_state.rs` | bounded 2.4.2 slice |
| M | `FilamentGroup.cpp` | **not ported** | general grouping optimizer not ported; bounded helpers live in FilamentGroupUtils row |
| M | `FilamentGroup.hpp` | **not ported** | general grouping optimizer not ported; bounded helpers live in FilamentGroupUtils row |
| M | `FilamentGroupUtils.cpp` | partial port — `C:options/filament_map_recommend.rs` | only bounded TPU grouping/check helpers; general optimizer not ported |
| M | `FilamentGroupUtils.hpp` | partial port — `C:options/filament_map_recommend.rs` | only bounded TPU grouping/check helpers; general optimizer not ported |
| A | `FilamentMixer.cpp` | **not ported** | new beta file |
| A | `FilamentMixer.hpp` | **not ported** | new beta file |
| A | `FilamentMixerModel.hpp` | **not ported** | new beta file |
| M | `FlushVolCalc.cpp` | partial port — `C:options/flush_volume/mod.rs` | bounded 2.4.2 slice |
| M | `FlushVolCalc.hpp` | partial port — `C:options/flush_volume/mod.rs` | bounded 2.4.2 slice |
| M | `FlushVolPredictor.cpp` | partial port — `C:options/flush_volume/color_distance.rs` | bounded 2.4.2 slice |
| M | `GCode.cpp` | partial port — `C:project_slice/gcode_emit.rs` | bounded 2.4.2 export orchestration, not entire native exporter |
| M | `GCode.hpp` | partial port — `C:project_slice/gcode_emit.rs` | bounded 2.4.2 export orchestration, not entire native exporter |
| M | `GCode/AdaptivePAInterpolator.cpp` | partial port — `C:project_slice/gcode_emit/adaptive_pa/model.rs` | bounded 2.4.2 slice |
| M | `GCode/AdaptivePAProcessor.cpp` | partial port — `C:project_slice/gcode_emit/adaptive_pa.rs` | bounded 2.4.2 slice |
| M | `GCode/AdaptivePAProcessor.hpp` | partial port — `C:project_slice/gcode_emit/adaptive_pa.rs` | bounded 2.4.2 slice |
| M | `GCode/AvoidCrossingPerimeters.cpp` | partial port — `C:project_slice/gcode_emit/motion/path/avoid_crossing/router.rs` | bounded 2.4.2 slice |
| M | `GCode/AvoidCrossingPerimeters.hpp` | partial port — `C:project_slice/gcode_emit/motion/path/avoid_crossing/router.rs` | bounded 2.4.2 slice |
| A | `GCode/BeltBackTransform.cpp` | **not ported** | deferred belt execution; no current port |
| A | `GCode/BeltBackTransform.hpp` | **not ported** | deferred belt execution; no current port |
| A | `GCode/BeltKinematics.cpp` | **not ported** | deferred belt execution; no current port |
| A | `GCode/BeltKinematics.hpp` | **not ported** | deferred belt execution; no current port |
| M | `GCode/ConflictChecker.cpp` | partial port — `C:project_slice/gcode_emit/conflict.rs` | bounded 2.4.2 slice |
| M | `GCode/ConflictChecker.hpp` | partial port — `C:project_slice/gcode_emit/conflict.rs` | bounded 2.4.2 slice |
| A | `GCode/ExclusionVolumePathCheck.cpp` | **not ported** | new beta file |
| A | `GCode/ExclusionVolumePathCheck.hpp` | **not ported** | new beta file |
| A | `GCode/ExclusionVolumeTravelAvoidance.cpp` | **not ported** | new beta file |
| A | `GCode/ExclusionVolumeTravelAvoidance.hpp` | **not ported** | new beta file |
| M | `GCode/ExtrusionProcessor.hpp` | partial port — `C:project_slice/gcode_emit/motion/overhang.rs` | bounded 2.4.2 slice |
| A | `GCode/MachineFrameTransform.cpp` | **not ported** | new beta file |
| A | `GCode/MachineFrameTransform.hpp` | **not ported** | new beta file |
| A | `GCode/MachineKinematics.cpp` | **not ported** | new beta file |
| A | `GCode/MachineKinematics.hpp` | **not ported** | new beta file |
| A | `GCode/OrderingStrategies.cpp` | **not ported** | new beta file |
| A | `GCode/OrderingStrategies.hpp` | **not ported** | new beta file |
| M | `GCode/PchipInterpolatorHelper.cpp` | partial port — `C:project_slice/gcode_emit/adaptive_pa/pchip.rs` | bounded 2.4.2 slice |
| M | `GCode/PrintExtents.cpp` | partial port — `C:project_slice/gcode_emit/wipe_tower/extents.rs` | bounded 2.4.2 slice |
| M | `GCode/RetractWhenCrossingPerimeters.cpp` | partial port — `C:project_slice/gcode_emit/motion/travel.rs` | bounded 2.4.2 slice |
| M | `GCode/RetractWhenCrossingPerimeters.hpp` | partial port — `C:project_slice/gcode_emit/motion/travel.rs` | bounded 2.4.2 slice |
| M | `GCode/TimelapsePosPicker.cpp` | partial port — `C:project_slice/gcode_emit/timelapse_pos.rs` | bounded 2.4.2 slice |
| M | `GCode/TimelapsePosPicker.hpp` | partial port — `C:project_slice/gcode_emit/timelapse_pos.rs` | bounded 2.4.2 slice |
| M | `GCode/ToolOrderUtils.cpp` | partial port — `C:project_slice/gcode_emit/tool_ordering/reorder/solve.rs` | bounded 2.4.2 slice |
| M | `GCode/ToolOrderUtils.hpp` | partial port — `C:project_slice/gcode_emit/tool_ordering/reorder/solve.rs` | bounded 2.4.2 slice |
| M | `GCode/ToolOrdering.cpp` | partial port — `C:project_slice/gcode_emit/tool_ordering.rs` | bounded 2.4.2 slice |
| M | `GCode/ToolOrdering.hpp` | partial port — `C:project_slice/gcode_emit/tool_ordering.rs` | bounded 2.4.2 slice |
| A | `GCode/WipePathHelpers.cpp` | **not ported** | new beta file |
| A | `GCode/WipePathHelpers.hpp` | **not ported** | new beta file |
| M | `GCode/WipeTower.cpp` | partial port — `C:project_slice/gcode_emit/wipe_tower/type1.rs` | bounded 2.4.2 slice |
| M | `GCode/WipeTower.hpp` | partial port — `C:project_slice/gcode_emit/wipe_tower/type1.rs` | bounded 2.4.2 slice |
| M | `GCode/WipeTower2.cpp` | partial port — `C:project_slice/gcode_emit/wipe_tower.rs` | bounded 2.4.2 slice |
| M | `GCode/WipeTower2.hpp` | partial port — `C:project_slice/gcode_emit/wipe_tower.rs` | bounded 2.4.2 slice |
| A | `GCode/WipeTowerEstimate.cpp` | **not ported** | new beta file |
| A | `GCode/WipeTowerEstimate.hpp` | **not ported** | new beta file |
| M | `GCodeWriter.cpp` | partial port — `C:gcode_writer.rs`, `C:gcode_move_emit.rs` | bounded 2.4.2 slice |
| M | `GCodeWriter.hpp` | partial port — `C:gcode_writer.rs`, `C:gcode_move_emit.rs` | bounded 2.4.2 slice |
| M | `PlaceholderParser.cpp` | partial port — `C:project_slice/gcode_emit/template.rs`, `C:project_slice/gcode_emit/expression.rs` | bounded 2.4.2 slice |
| M | `PlaceholderParser.hpp` | partial port — `C:project_slice/gcode_emit/template.rs`, `C:project_slice/gcode_emit/expression.rs` | bounded 2.4.2 slice |
| M | `Print.cpp` | partial port — `C:print/validate.rs`, `C:project_slice/gcode_emit/tool_ordering/recommend.rs` | bounded validation/map/orchestration slices; no full Print class |
| M | `Print.hpp` | partial port — `C:print/validate.rs`, `C:project_slice/gcode_emit/tool_ordering/recommend.rs` | bounded validation/map/orchestration slices; no full Print class |
| M | `PrintBase.cpp` | partial port — `C:project_slice/state.rs`, `C:slicer_context.rs` | bounded status/config ownership/cancellation shell |
| M | `PrintBase.hpp` | partial port — `C:project_slice/state.rs`, `C:slicer_context.rs` | bounded status/config ownership/cancellation shell |
| M | `ProjectTask.cpp` | **not ported** | no identified current port |
| M | `ProjectTask.hpp` | **not ported** | no identified current port |
