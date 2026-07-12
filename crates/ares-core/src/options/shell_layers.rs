use crate::{ShellLayerOptions, SliceError};

pub(super) fn parse_shell_layer_options(
    options: &super::SliceOptions,
) -> Result<ShellLayerOptions, SliceError> {
    Ok(ShellLayerOptions::with_thicknesses(
        options.non_negative_u32("bottom_shell_layers", 3)? as usize,
        options.range_f64("bottom_shell_thickness", 0.0, 0.0, f64::INFINITY)?,
        options.non_negative_u32("top_shell_layers", 4)? as usize,
        options.range_f64("top_shell_thickness", 0.6, 0.0, f64::INFINITY)?,
    ))
}
