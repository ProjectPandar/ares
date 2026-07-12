use crate::{LayerExtrusionMoves, LayerSpeedMoves, SliceError, SliceOptions};

pub(crate) fn finish(
    options: &SliceOptions,
    gcode: String,
    layer_extrusion_moves: &[LayerExtrusionMoves],
    layer_speed_moves: &[LayerSpeedMoves],
) -> Result<String, SliceError> {
    crate::gcode_line_numbers::apply(
        options,
        render(options, gcode, layer_extrusion_moves, layer_speed_moves)?,
    )
}

fn render(
    options: &SliceOptions,
    gcode: String,
    layer_extrusion_moves: &[LayerExtrusionMoves],
    layer_speed_moves: &[LayerSpeedMoves],
) -> Result<String, SliceError> {
    let print_time_sec =
        crate::gcode_filament_stats::normal_print_time_s(options, layer_speed_moves)?;
    let used_filament_length_m =
        crate::gcode_filament_stats::used_filament_mm(layer_extrusion_moves) / 1000.0;

    Ok(gcode
        .replace(
            crate::gcode_reserved_tags::PRINT_TIME_SEC,
            &format!("{print_time_sec:.2}"),
        )
        .replace(
            crate::gcode_reserved_tags::USED_FILAMENT_LENGTH,
            &format!("{used_filament_length_m:.2}"),
        ))
}
