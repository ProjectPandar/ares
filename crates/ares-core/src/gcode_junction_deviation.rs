use crate::{SliceError, SliceOptions, gcode_writer::GCodeWriter, options::GCodeFlavor};

/// `GCodeWriter.cpp:352-361` `set_junction_deviation`: marlin2-only,
/// gated on a positive machine maximum and value, `M205 J` with the
/// value clamped to the maximum at 3 fixed decimals and the
/// ` ; Junction Deviation` suffix under full comments.
pub(crate) fn junction_deviation_command(
    flavor: GCodeFlavor,
    junction_deviation: f64,
    max_junction_deviation: f64,
    comments: bool,
) -> String {
    if flavor != GCodeFlavor::MarlinFirmware
        || max_junction_deviation <= 0.0
        || junction_deviation <= 0.0
    {
        return String::new();
    }
    let value = junction_deviation.min(max_junction_deviation);
    let suffix = if comments {
        " ; Junction Deviation"
    } else {
        ""
    };
    format!("M205 J{value:.3}{suffix}\n")
}

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
