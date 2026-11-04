//! Layer-end timelapse + retraction tail for a chunk's last object entry
//! (`GCode.cpp:5527-5546`).

use super::super::{SliceError, append_layer_end_timelapse, motion, timelapse};

/// Start-insert result and traditional-mode gate for the layer-end
/// `insert_timelapse_gcode` path (`GCode.cpp:5171-5178`).
pub(super) struct LayerTimelapse {
    pub(super) timelapse_inserted: bool,
    pub(super) traditional_timelapse: bool,
}

pub(super) fn emit(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    LayerTimelapse {
        timelapse_inserted,
        traditional_timelapse,
    }: LayerTimelapse,
    timelapse_context: timelapse::Context<'_>,
) -> Result<(), SliceError> {
    // `GCode.cpp:8165`: publish the inline insert before the layer-end tail.
    state.has_insert_timelapse_gcode |= state.farthest_point.inserted_this_layer;
    // The layer-end timelapse sequence closes every layer chunk
    // (`GCode.cpp:5527-5546`), after all of the chunk's objects.
    if timelapse_inserted || state.has_insert_timelapse_gcode {
        motion::defer_layer_retraction(state);
    } else {
        motion::end_layer_for_timelapse(output, state);
    }
    append_layer_end_timelapse(
        output,
        state,
        timelapse_inserted,
        traditional_timelapse,
        timelapse_context,
    )
}
