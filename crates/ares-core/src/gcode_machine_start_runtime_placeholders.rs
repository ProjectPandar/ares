use crate::{SliceError, SliceOptions};

const MAXIMUM_EXTRUDER_NUMBER: usize = 64;
const HIGH_LOW_TEMP_MIX_SOURCE_KEY: &str = "enable_high_low_temp_mixed_printing";

pub(crate) fn render(
    template: String,
    options: &SliceOptions,
    filament_count: usize,
    first_layer_print: Option<
        &crate::gcode_first_layer_print_placeholders::FirstLayerPrintPlaceholders,
    >,
) -> Result<String, SliceError> {
    let rendered = render_is_extruder_used(template, filament_count);
    let rendered = render_high_low_temp_mix(rendered, options)?;
    render_head_wrap_detect_zone(rendered, options, first_layer_print)
}

fn render_is_extruder_used(template: String, filament_count: usize) -> String {
    if !template.contains("[is_extruder_used]") {
        return template;
    }
    template.replace(
        "[is_extruder_used]",
        &format_is_extruder_used(filament_count),
    )
}

fn render_high_low_temp_mix(
    template: String,
    options: &SliceOptions,
) -> Result<String, SliceError> {
    if !template.contains("[enable_high_low_temp_mix]") {
        return Ok(template);
    }
    let enabled = options.bool_option(HIGH_LOW_TEMP_MIX_SOURCE_KEY, false)?;
    Ok(template.replace(
        "[enable_high_low_temp_mix]",
        if enabled { "1" } else { "0" },
    ))
}

fn format_is_extruder_used(filament_count: usize) -> String {
    let count = MAXIMUM_EXTRUDER_NUMBER.max(filament_count);
    (0..count)
        .map(|index| if index == 0 { "1" } else { "0" })
        .collect::<Vec<_>>()
        .join(",")
}

fn render_head_wrap_detect_zone(
    template: String,
    options: &SliceOptions,
    first_layer_print: Option<
        &crate::gcode_first_layer_print_placeholders::FirstLayerPrintPlaceholders,
    >,
) -> Result<String, SliceError> {
    if !template.contains("[in_head_wrap_detect_zone]") {
        return Ok(template);
    }
    let value = crate::gcode_head_wrap_detect_zone::placeholder_value(options, first_layer_print)?;
    Ok(template.replace("[in_head_wrap_detect_zone]", value))
}
