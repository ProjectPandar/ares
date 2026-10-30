# Upstream v2.5.0-beta inventory — G-code processing and native-only/platform surfaces

Part 6 of the [baseline migration ARD](2026-10-30-upstream-baseline-v2.5.0-beta.md).
Planning base: Ares `e4a12268782154587e164a11bb67f09701599d3b`. Target: upstream
`345efec64c7b95a05ce5cf96df0dde66a158835c`; source baseline `v2.4.2` -> `v2.5.0-beta`.

All upstream paths below have prefix `src/libslic3r/`; statuses are Git `M`/`A`/`D`.
`C:` = `crates/ares-core/src/`; `V:` = `crates/ares-vgcode/src/`.
`partial port` identifies an existing bounded 2.4.2 rewrite/counterpart, NOT full-file
coverage or 2.5.0 qualification. `not ported` includes new files, native-only APIs
and helpers without an identified source-port owner; a similarly named type or
comment alone is not evidence of a port. Deleted files retain their old owner
when present. The processor owner is the proposed sole upstream-file owner; deferred rows
are tracking ownership, not approval to implement those features.

Rows: **85**. Every listed Rust path was checked at the planning base.

| Status | Upstream file | Current Ares port/counterpart | Scope / gap |
|---|---|---|---|
| M | `AnyPtr.hpp` | **not ported** | no identified current port |
| M | `BlacklistedLibraryCheck.cpp` | **not ported** | no identified current port |
| M | `CMakeLists.txt` | **not ported** | no identified current port |
| M | `Channel.hpp` | **not ported** | no identified current port |
| M | `Color.cpp` | partial port — `V:convert.rs` | bounded color conversion only; not texture/mixer color models |
| M | `Color.hpp` | partial port — `V:convert.rs` | bounded color conversion only; not texture/mixer color models |
| M | `GCode/CoolingBuffer.cpp` | partial port — `C:project_slice/gcode_emit/cooling.rs` | bounded 2.4.2 slice |
| M | `GCode/CoolingBuffer.hpp` | partial port — `C:project_slice/gcode_emit/cooling.rs` | bounded 2.4.2 slice |
| A | `GCode/ElegooGCodeProcessorHelper.cpp` | **not ported** | new beta file |
| M | `GCode/FanMover.cpp` | partial port — `C:project_slice/gcode_emit/fan_mover.rs` | bounded 2.4.2 slice |
| M | `GCode/FanMover.hpp` | partial port — `C:project_slice/gcode_emit/fan_mover.rs` | bounded 2.4.2 slice |
| M | `GCode/GCodeProcessor.cpp` | partial port — `C:project_slice/gcode_emit/processor.rs` | bounded 2.4.2 slice |
| M | `GCode/GCodeProcessor.hpp` | partial port — `C:project_slice/gcode_emit/processor.rs` | bounded 2.4.2 slice |
| A | `GCode/GCodeStatistics.cpp` | **not ported** | new beta file |
| A | `GCode/GCodeStatistics.hpp` | **not ported** | new beta file |
| D | `GCode/PostProcessor.cpp` | partial port — `C:project_slice/gcode_emit/processor/export_lines.rs` | bounded old rewrite; deleted PostProcessor must move to beta processor owner |
| D | `GCode/PostProcessor.hpp` | partial port — `C:project_slice/gcode_emit/processor/export_lines.rs` | bounded old rewrite; deleted PostProcessor must move to beta processor owner |
| M | `GCode/PressureEqualizer.cpp` | partial port — `C:project_slice/pressure_equalizer.rs` | bounded 2.4.2 slice |
| M | `GCode/PressureEqualizer.hpp` | partial port — `C:project_slice/pressure_equalizer.rs` | bounded 2.4.2 slice |
| M | `GCode/SmallAreaInfillFlowCompensator.cpp` | partial port — `C:project_slice/gcode_emit/small_area.rs` | bounded 2.4.2 slice |
| M | `GCode/SmallAreaInfillFlowCompensator.hpp` | partial port — `C:project_slice/gcode_emit/small_area.rs` | bounded 2.4.2 slice |
| M | `GCode/SpiralVase.cpp` | partial port — `C:project_slice/gcode_emit/spiral_vase.rs` | bounded 2.4.2 slice |
| M | `GCode/SpiralVase.hpp` | partial port — `C:project_slice/gcode_emit/spiral_vase.rs` | bounded 2.4.2 slice |
| M | `GCode/ThumbnailData.cpp` | partial port — `C:gcode_thumbnails.rs` | bounded 2.4.2 slice |
| M | `GCode/ThumbnailData.hpp` | partial port — `C:gcode_thumbnails.rs` | bounded 2.4.2 slice |
| M | `GCode/Thumbnails.cpp` | partial port — `C:gcode_thumbnails.rs` | bounded 2.4.2 slice |
| M | `GCode/Thumbnails.hpp` | partial port — `C:gcode_thumbnails.rs` | bounded 2.4.2 slice |
| M | `GCodeReader.cpp` | partial port — `C:project_slice/gcode_emit/processor/export_lines/reader.rs` | bounded 2.4.2 slice |
| M | `GCodeReader.hpp` | partial port — `C:project_slice/gcode_emit/processor/export_lines/reader.rs` | bounded 2.4.2 slice |
| D | `GCodeSender.cpp` | **not ported** | deleted upstream; no existing port to retire |
| D | `GCodeSender.hpp` | **not ported** | deleted upstream; no existing port to retire |
| A | `LifecycleEvents.hpp` | **not ported** | new beta file |
| M | `MTUtils.hpp` | **not ported** | no identified current port |
| M | `PNGReadWrite.cpp` | **not ported** | no identified current port |
| M | `PNGReadWrite.hpp` | **not ported** | no identified current port |
| A | `ParallelResolve.hpp` | **not ported** | new beta file |
| M | `Platform.cpp` | **not ported** | no identified current port |
| M | `Platform.hpp` | **not ported** | no identified current port |
| M | `SLA/AGGRaster.hpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/BoostAdapter.hpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/Clustering.cpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/Clustering.hpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/ConcaveHull.cpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/ConcaveHull.hpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/Concurrency.hpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/Hollowing.cpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/Hollowing.hpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/Pad.cpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/RasterBase.cpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/RasterBase.hpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/RasterToPolygons.cpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/RasterToPolygons.hpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/ReprojectPointsOnMesh.hpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/Rotfinder.cpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/Rotfinder.hpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/SpatIndex.cpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/SpatIndex.hpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/SupportPoint.hpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/SupportPointGenerator.cpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/SupportPointGenerator.hpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/SupportTree.cpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/SupportTree.hpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/SupportTreeBuilder.cpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/SupportTreeBuilder.hpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/SupportTreeBuildsteps.cpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/SupportTreeBuildsteps.hpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/SupportTreeMesher.cpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLA/SupportTreeMesher.hpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| D | `SLA/bicubic.h` | **not ported** | deleted upstream; no existing port to retire |
| M | `SLAPrint.cpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLAPrint.hpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLAPrintSteps.cpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SLAPrintSteps.hpp` | **not ported** | SLA execution not ported; outside FFF baseline migration |
| M | `SVG.cpp` | **not ported** | no identified current port |
| M | `SVG.hpp` | **not ported** | no identified current port |
| M | `StreamUtils.hpp` | **not ported** | no identified current port |
| M | `Thread.cpp` | **not ported** | no identified current port |
| M | `Thread.hpp` | **not ported** | no identified current port |
| M | `Time.cpp` | **not ported** | no identified current port |
| M | `Timer.cpp` | **not ported** | no identified current port |
| M | `Timer.hpp` | **not ported** | no identified current port |
| M | `clonable_ptr.hpp` | **not ported** | no identified current port |
| M | `format.hpp` | **not ported** | no identified current port |
| M | `libslic3r_version.h.in` | **not ported** | no identified current port |
| M | `pchheader.hpp` | **not ported** | no identified current port |
