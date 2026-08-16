# Spec: Task 22o.116 per-path layer-height processor tags

## Observable contract

Ares emits `; LAYER_HEIGHT: <height>` before an extrusion path whenever that path's height differs from the last processor height. The marker follows `FEATURE` and `LINE_WIDTH` markers and precedes feedrate. At each layer boundary, processor height resets to that layer's generated height, so ordinary same-height paths do not duplicate the layer header marker.

For KSR thick internal bridges, the emitted block contains `; LAYER_HEIGHT: 0.4` before `G1 F4500`.

## Upstream boundary

Port `OrcaSlicer/src/libslic3r/GCode.cpp:6781-6803`, specifically `m_last_height` tracking and `GCodeProcessor::ETags::Height` emission. The Rust destination is `gcode_emit/motion.rs` and its `PathProperties` seam.

## Deferred behavior

Exact layer-plane generation, path ordering, dynamic pressure-advance tags, and wipe-tower processor-state resets remain separate source-cited slices.
