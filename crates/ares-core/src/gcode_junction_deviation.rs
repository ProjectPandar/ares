use crate::{SliceError, SliceOptions, gcode_writer::GCodeWriter};

pub(crate) fn first_layer_command(
    writer: &GCodeWriter,
    options: &SliceOptions,
    layer_index: usize,
) -> Result<String, SliceError> {
    if layer_index != 0 {
        return Ok(String::new());
    }
    Ok(writer.set_junction_deviation(
        options.default_junction_deviation()?,
        options.machine_limits()?.max_junction_deviation,
        options.gcode_comments()?,
    ))
}
