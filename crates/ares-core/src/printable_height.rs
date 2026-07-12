use crate::{Layer, SliceError, SliceOptions};

const EPSILON: f64 = 1e-6;

pub(crate) fn validate_layers(layers: &[Layer], options: &SliceOptions) -> Result<(), SliceError> {
    let global_printable_height = printable_height(options)?;
    let first_extruder_printable_height = first_extruder_printable_height(options)?;
    let (limit_name, effective_printable_height) =
        match first_extruder_printable_height.filter(|height| *height < global_printable_height) {
            Some(height) => ("extruder_printable_height", height),
            None => ("printable_height", global_printable_height),
        };
    let max_print_z = layers.iter().map(Layer::print_z).fold(0.0_f64, f64::max);

    if max_print_z > effective_printable_height + EPSILON {
        return Err(SliceError::InvalidInput(format!(
            "{limit_name} {effective_printable_height} is below planned max print Z {max_print_z}"
        )));
    }

    Ok(())
}

fn printable_height(options: &SliceOptions) -> Result<f64, SliceError> {
    let value = match options.values().get("printable_height") {
        Some(value) => numeric_value(value, "printable_height").map(Some)?,
        None => crate::options::registry::option_definition("printable_height")
            .and_then(|definition| definition.default_value.parse().ok()),
    }
    .ok_or_else(|| {
        SliceError::InvalidInput("printable_height must be a finite number".to_owned())
    })?;

    if value.is_finite() && value >= 0.0 {
        Ok(value)
    } else {
        Err(SliceError::InvalidInput(
            "printable_height must be non-negative".to_owned(),
        ))
    }
}

fn first_extruder_printable_height(options: &SliceOptions) -> Result<Option<f64>, SliceError> {
    let Some(value) = options.values().get("extruder_printable_height") else {
        return Ok(None);
    };
    let Some(value) = first_extruder_height_value(value) else {
        return Ok(None);
    };
    let value = numeric_value(value, "extruder_printable_height")?;

    if !value.is_finite() {
        return Err(SliceError::InvalidInput(
            "extruder_printable_height must be finite".to_owned(),
        ));
    }
    if !(0.0..=1000.0).contains(&value) {
        return Err(SliceError::InvalidInput(
            "extruder_printable_height must be between 0 and 1000".to_owned(),
        ));
    }
    if value == 0.0 {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

fn first_extruder_height_value(value: &serde_json::Value) -> Option<&serde_json::Value> {
    match value {
        serde_json::Value::Null => None,
        serde_json::Value::Array(values) => values.first().filter(|value| !value.is_null()),
        value => Some(value),
    }
}

fn numeric_value(value: &serde_json::Value, key: &str) -> Result<f64, SliceError> {
    match value {
        serde_json::Value::Number(number) => number.as_f64(),
        serde_json::Value::String(text) => text.parse().ok(),
        _ => None,
    }
    .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a finite number")))
}
