use crate::{SliceError, SliceOptions};

pub(crate) fn before_layer_change_gcode(
    options: &SliceOptions,
    layer_num: usize,
    layer_z: &str,
) -> Result<String, SliceError> {
    crate::gcode_placeholders::before_layer_change_gcode(options, layer_num, layer_z, layer_z)
}

pub(crate) fn after_z_gcode(
    options: &SliceOptions,
    layer_num: usize,
    layer_z: &str,
) -> Result<String, SliceError> {
    let mut gcode = time_lapse_gcode(options, layer_num, layer_z)?;
    gcode.push_str(&layer_change_gcode(options, layer_num, layer_z)?);
    Ok(gcode)
}

fn time_lapse_gcode(
    options: &SliceOptions,
    layer_num: usize,
    layer_z: &str,
) -> Result<String, SliceError> {
    crate::gcode_placeholders::time_lapse_gcode(options, layer_num, layer_z, layer_z)
}

fn layer_change_gcode(
    options: &SliceOptions,
    layer_num: usize,
    layer_z: &str,
) -> Result<String, SliceError> {
    crate::gcode_placeholders::layer_change_gcode(options, layer_num, layer_z, layer_z)
}
