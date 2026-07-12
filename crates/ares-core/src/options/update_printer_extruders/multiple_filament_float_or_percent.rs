use serde_json::{Number, Value};

use crate::SliceError;

#[derive(Clone, Copy)]
struct FloatOrPercent {
    value: f64,
    percent: bool,
}

pub(super) fn copy_float_or_percent_values(
    value: &Value,
    key: &str,
    variant_indices: &[usize],
) -> Result<Vec<Value>, SliceError> {
    let values = float_or_percent_vector(value, key)?;
    let mut copied = vec![Value::Number(Number::from(0)); variant_indices.len()];
    for (filament_index, variant_index) in variant_indices.iter().enumerate() {
        if *variant_index < values.len() {
            copied[filament_index] = float_or_percent_value(values[*variant_index], key)?;
        }
    }
    Ok(copied)
}

fn float_or_percent_vector(value: &Value, key: &str) -> Result<Vec<FloatOrPercent>, SliceError> {
    let values = value.as_array().ok_or_else(|| {
        SliceError::InvalidInput(format!("{key} must be a FloatOrPercent vector"))
    })?;
    values
        .iter()
        .map(|value| match value {
            Value::Number(number) => number
                .as_f64()
                .filter(|value| value.is_finite())
                .map(|value| FloatOrPercent {
                    value,
                    percent: false,
                })
                .ok_or_else(|| {
                    SliceError::InvalidInput(format!(
                        "{key} must contain finite FloatOrPercent values"
                    ))
                }),
            Value::String(text) => parse_float_or_percent(text, key),
            _ => Err(SliceError::InvalidInput(format!(
                "{key} must contain FloatOrPercent values"
            ))),
        })
        .collect()
}

fn parse_float_or_percent(text: &str, key: &str) -> Result<FloatOrPercent, SliceError> {
    let text = text.trim();
    let (text, percent) = text
        .strip_suffix('%')
        .map_or((text, false), |text| (text.trim(), true));
    let value = text.parse::<f64>().map_err(|_| {
        SliceError::InvalidInput(format!("{key} must contain finite FloatOrPercent values"))
    })?;
    value
        .is_finite()
        .then_some(FloatOrPercent { value, percent })
        .ok_or_else(|| {
            SliceError::InvalidInput(format!("{key} must contain finite FloatOrPercent values"))
        })
}

fn float_or_percent_value(value: FloatOrPercent, key: &str) -> Result<Value, SliceError> {
    if value.percent {
        Ok(Value::String(format!("{}%", value.value)))
    } else {
        Number::from_f64(value.value)
            .map(Value::Number)
            .ok_or_else(|| SliceError::InvalidInput(format!("{key} must contain finite numbers")))
    }
}
