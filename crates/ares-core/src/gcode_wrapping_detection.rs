use serde_json::Value;

use crate::{SliceError, SliceOptions};

const ENABLE_KEY: &str = "enable_wrapping_detection";
const GCODE_KEY: &str = "wrapping_detection_gcode";
const LAYERS_KEY: &str = "wrapping_detection_layers";
const EXCLUDE_AREA_KEY: &str = "wrapping_exclude_area";
const DEFAULT_LAYERS: usize = 20;

pub(crate) fn layer_command(
    options: &SliceOptions,
    layer_num: usize,
    layer_z: &str,
) -> Result<String, SliceError> {
    let template = template(options)?;
    let has_wrapping_exclude_area =
        has_wrapping_exclude_area(options.values().get(EXCLUDE_AREA_KEY))?;
    if !options.bool_option(ENABLE_KEY, false)?
        || template.is_empty()
        || !has_wrapping_exclude_area
        || !within_wrapping_detection_layers(options, layer_num)?
    {
        return Ok(String::new());
    }

    let layer_num = layer_num.to_string();
    let mut rendered = replace_placeholder(template, "layer_num", &layer_num);
    rendered = replace_placeholder(&rendered, "layer_z", layer_z);
    rendered = replace_placeholder(&rendered, "max_layer_z", layer_z);
    let physical_extruder_id = options.physical_extruder_id_for_logical(0)?.to_string();
    rendered = replace_placeholder(
        &rendered,
        "most_used_physical_extruder_id",
        &physical_extruder_id,
    );
    rendered = replace_placeholder(
        &rendered,
        "curr_physical_extruder_id",
        &physical_extruder_id,
    );
    Ok(ensure_trailing_newline(rendered))
}

fn template(options: &SliceOptions) -> Result<&str, SliceError> {
    let Some(value) = options.values().get(GCODE_KEY) else {
        return Ok("");
    };
    value
        .as_str()
        .ok_or_else(|| SliceError::InvalidInput(format!("{GCODE_KEY} must be a string")))
}

fn within_wrapping_detection_layers(
    options: &SliceOptions,
    layer_num: usize,
) -> Result<bool, SliceError> {
    Ok(layer_num <= wrapping_detection_layers(options.values().get(LAYERS_KEY))?)
}

fn wrapping_detection_layers(value: Option<&Value>) -> Result<usize, SliceError> {
    let Some(value) = value else {
        return Ok(DEFAULT_LAYERS);
    };
    match value {
        Value::Number(number) => number
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(invalid_layers),
        Value::String(text) => text.trim().parse().map_err(|_| invalid_layers()),
        _ => Err(invalid_layers()),
    }
}

fn has_wrapping_exclude_area(value: Option<&Value>) -> Result<bool, SliceError> {
    Ok(wrapping_exclude_area_points(value)? > 2)
}

fn wrapping_exclude_area_points(value: Option<&Value>) -> Result<usize, SliceError> {
    match value {
        None => Ok(0),
        Some(Value::String(text)) => wrapping_exclude_area_text_points(text),
        Some(Value::Array(points)) => wrapping_exclude_area_array_points(points),
        Some(_) => Err(invalid_exclude_area("unsupported value")),
    }
}

fn wrapping_exclude_area_text_points(text: &str) -> Result<usize, SliceError> {
    let text = text.trim();
    if text.is_empty() || text == "0x0" {
        return Ok(0);
    }

    let mut count = 0;
    for token in text.split(',') {
        let token = token.trim();
        let Some((x, y)) = token.split_once('x') else {
            return Err(invalid_exclude_area("malformed point"));
        };
        if y.contains('x') || x.trim().is_empty() || y.trim().is_empty() {
            return Err(invalid_exclude_area("malformed point"));
        }
        parse_exclude_area_number(x)?;
        parse_exclude_area_number(y)?;
        count += 1;
    }
    Ok(count)
}

fn wrapping_exclude_area_array_points(values: &[Value]) -> Result<usize, SliceError> {
    for value in values {
        let Value::Array(coords) = value else {
            return Err(invalid_exclude_area("malformed JSON point"));
        };
        let [x, y] = coords.as_slice() else {
            return Err(invalid_exclude_area("malformed JSON point"));
        };
        let Some(x) = x.as_f64() else {
            return Err(invalid_exclude_area("malformed JSON point"));
        };
        let Some(y) = y.as_f64() else {
            return Err(invalid_exclude_area("malformed JSON point"));
        };
        if !x.is_finite() || !y.is_finite() {
            return Err(invalid_exclude_area("non-finite coordinate"));
        }
    }
    Ok(values.len())
}

fn parse_exclude_area_number(value: &str) -> Result<f64, SliceError> {
    let parsed = value
        .trim()
        .parse::<f64>()
        .map_err(|_| invalid_exclude_area("malformed coordinate"))?;
    if parsed.is_finite() {
        Ok(parsed)
    } else {
        Err(invalid_exclude_area("non-finite coordinate"))
    }
}

fn invalid_exclude_area(reason: &str) -> SliceError {
    SliceError::InvalidInput(format!("{EXCLUDE_AREA_KEY} {reason}"))
}

fn invalid_layers() -> SliceError {
    SliceError::InvalidInput(format!("{LAYERS_KEY} must be a non-negative integer"))
}

fn replace_placeholder(template: &str, key: &str, value: &str) -> String {
    template
        .replace(&format!("{{{key}}}"), value)
        .replace(&format!("[{key}]"), value)
}

fn ensure_trailing_newline(mut gcode: String) -> String {
    if !gcode.ends_with('\n') {
        gcode.push('\n');
    }
    gcode
}
