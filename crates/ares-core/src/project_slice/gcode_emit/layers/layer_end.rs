//! Layer-end timelapse + retraction tail for a chunk's last object entry
//! (`GCode.cpp:5527-5546`).

use super::super::{
    SliceError, append_layer_end_timelapse, motion, object::ObjectLabels, timelapse,
};

pub(super) fn emit(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    object_labels: Option<&ObjectLabels>,
    timelapse_inserted: bool,
    traditional_timelapse: bool,
    timelapse_context: timelapse::Context<'_>,
) -> Result<(), SliceError> {
    // The layer-end timelapse sequence closes every layer chunk
    // (`GCode.cpp:5527-5546`), after all of the chunk's objects.
    if object_labels.is_none() && timelapse_inserted {
        motion::defer_layer_retraction(state);
    } else if object_labels.is_none() {
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
