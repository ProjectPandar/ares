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
        traversal,
        metadata,
        start_position,
        first_layer_bounds,
    )?;
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
