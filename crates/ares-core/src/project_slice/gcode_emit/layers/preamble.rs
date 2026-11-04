//! First-chunk print preamble — the start-of-print g-code plus the
//! trailing-XY tracking upstream needs for the brim split
//! (`GCode.cpp:3139-3140`, `:5693`).

use super::super::{
    GenerationMetadata, SliceError, footprint, layer_gcode, motion, tags::Tags, trailing_gcode_xy,
};
use crate::project_slice::perimeters::classic::traversal::types::PreparedPostClassicTraversal;

pub(super) fn emit(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    traversal: &PreparedPostClassicTraversal,
    metadata: GenerationMetadata,
    first_layer_bounds: Option<footprint::FirstLayerBounds>,
) -> Result<usize, SliceError> {
    // `GCode::preamble` performs a *silent* `travel_to_z(z_offset)`
    // (`GCode.cpp:5676-5679` — the returned G-code is discarded, only
    // `m_pos(2)` updates). Ordering differs by printer family: for BBL
    // machines the filament-start template runs BEFORE this reset
    // (`GCode.cpp:3150-3153` then `3202`), for non-BBL `preamble()` runs
    // first and `set_extruder`'s filament-start follows (`3227`).
    // Absolute-E likewise resets (`GCodeWriter.cpp:103`).
    let reset_writer = |state: &mut motion::EmitState| {
        // `travel_to_z` leaves Z untouched below EPSILON
        // (`GCodeWriter.cpp:816-822,927-928`) — a sub-EPSILON assigned
        // Z survives the silent move.
        let current = state.writer_z.unwrap_or(0.0);
        if (state.options.z_offset - current).abs() >= 1e-4 {
            state.writer_z = Some(state.options.z_offset);
        } else if state.writer_z.is_none() {
            state.writer_z = Some(0.0);
        }
        // The absolute-E counter resets via `GCodeWriter::preamble`'s
        // `reset_e(true)` (`GCodeWriter.cpp:95-103`) — but only when an
        // extruder is ACTIVE (`reset_e` no-ops with filament id -1,
        // `GCodeWriter.cpp:487-490`). BBL initializes the initial
        // extruder early (`GCode.cpp:3143-3144`); non-BBL selects it
        // inside `set_extruder` AFTER rendering filament-start
        // (`GCode.cpp:7736-7746`), so a machine-start `e_position`
        // write-back survives into the non-BBL filament-start.
        if !state.options.use_relative_e_distances && Tags::of(traversal).is_bbl() {
            state.extruder.e_position = 0.0;
        }
    };
    if !Tags::of(traversal).is_bbl() {
        reset_writer(state);
    }
    if !state
        .wipe_tower
        .as_ref()
        .is_some_and(|tower| tower.priming_enabled())
    {
        layer_gcode::append_print_preamble(output, state, traversal, metadata, first_layer_bounds)?;
    }
    if Tags::of(traversal).is_bbl() {
        reset_writer(state);
    }
    // Upstream's writer does NOT know the Z after the start g-code
    // (`GCode.cpp:3139-3140`), so `writer_z` stays unset
    // (`GCode.cpp:5693`). The brim split target uses the nozzle XY
    // the start g-code left — track it in live gcode coordinates, plus the
    // writer's plate offset (`GCode.cpp:4546-4548`).
    if let Some((x, y)) = trailing_gcode_xy(output) {
        let [offset_x, offset_y] = state.xy_offset_f32();
        state.x = x + f64::from(offset_x);
        state.y = y + f64::from(offset_y);
    }
    Ok(output.len())
}
