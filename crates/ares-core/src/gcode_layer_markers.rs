use crate::{SliceError, SliceOptions, gcode_writer::GCodeWriter};

pub(crate) fn push(
    gcode: &mut String,
    writer: &GCodeWriter,
    options: &SliceOptions,
    layer: (usize, usize, &str),
) -> Result<(), SliceError> {
    let (layer_index, layer_num, layer_z) = layer;
    gcode.push_str(&crate::gcode_wrapping_detection::layer_command(
        options, layer_num, layer_z,
    )?);
    gcode.push_str(crate::gcode_scan_first_layer::layer_command(
        options,
        layer_index,
    )?);
    gcode.push_str(&crate::gcode_junction_deviation::first_layer_command(
        writer,
        options,
        layer_index,
    )?);
    Ok(())
}
