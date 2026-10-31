use crate::{SliceError, SliceOptions};

const GCODE_ADD_LINE_NUMBER: &str = "gcode_add_line_number";

pub(crate) fn apply(options: &SliceOptions, gcode: String) -> Result<String, SliceError> {
    Ok(add_line_numbers_if_enabled(gcode, enabled(options)?))
}

fn enabled(options: &SliceOptions) -> Result<bool, SliceError> {
    let Some(value) = options.values().get(GCODE_ADD_LINE_NUMBER) else {
        return Ok(false);
    };
    value.as_bool().ok_or_else(|| {
        SliceError::InvalidInput(format!("{GCODE_ADD_LINE_NUMBER} must be a boolean"))
    })
}

fn add_line_numbers_if_enabled(gcode: String, enabled: bool) -> String {
    if !enabled {
        return gcode;
    }

    let mut numbered = String::new();
    for (index, line) in gcode.lines().enumerate() {
        numbered.push('N');
        numbered.push_str(&(index + 1).to_string());
        numbered.push(' ');
        numbered.push_str(line);
        numbered.push('\n');
    }
    numbered
}
