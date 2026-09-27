//! First-chunk print preamble — the start-of-print g-code plus the
//! trailing-XY tracking upstream needs for the brim split
//! (`GCode.cpp:3139-3140`, `:5693`).

use super::super::{
    GenerationMetadata, SliceError, footprint, layer_gcode, motion, trailing_gcode_xy, value,
};
use crate::project_slice::perimeters::classic::traversal::types::PreparedPostClassicTraversal;

pub(super) fn emit(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    traversal: &PreparedPostClassicTraversal,
    metadata: GenerationMetadata,
    start_position: Option<&value::Value>,
    first_layer_bounds: Option<footprint::FirstLayerBounds>,
) -> Result<(), SliceError> {
    layer_gcode::append_print_preamble(
        output,
        state,
        traversal,
        metadata,
        start_position,
        first_layer_bounds,
    )?;
    // `GCode::preamble` performs a *silent* `travel_to_z(z_offset)`
    // AFTER the filament-start template (`GCode.cpp:3202,5676-5679` —
    // the returned G-code is discarded, only `m_pos(2)` updates), so a
    // filament-start `position[2]` assignment is overwritten by the
    // offset reset. Absolute-E likewise resets (`GCodeWriter.cpp:103`).
    state.writer_z = Some(state.options.z_offset);
    if !state.options.use_relative_e_distances {
        state.e_position = 0.0;
    }
    // Upstream's writer does NOT know the Z after the start g-code
    // (`GCode.cpp:3139-3140`), so `writer_z` stays unset
    // (`GCode.cpp:5693`). The brim split target uses the nozzle XY
    // the start g-code left — track it in live gcode coordinates.
    if let Some((x, y)) = trailing_gcode_xy(output) {
        state.x = x;
        state.y = y;
    }
    Ok(())
}
