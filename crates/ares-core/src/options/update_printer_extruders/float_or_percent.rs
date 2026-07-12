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
    stride: usize,
) -> Result<Vec<Value>, SliceError> {
    let values = float_or_percent_vector(value, key)?;
    let mut copied = Vec::with_capacity(variant_indices.len() * stride);
    for variant_index in variant_indices {
        for offset in 0..stride {
            let source_index = super::source_index(*variant_index, stride, offset)?;
            copied.push(float_or_percent_value(
                float_or_percent_get_at(&values, key, source_index)?,
                key,
            )?);
        }
    }
    Ok(copied)
}

fn float_or_percent_vector(value: &Value, key: &str) -> Result<Vec<FloatOrPercent>, SliceError> {
    let values = value.as_array().ok_or_else(|| {
        SliceError::InvalidInput(format!("{key} must be a FloatOrPercent vector"))
    })?;
    if values.is_empty() {
        return Err(SliceError::InvalidInput(format!("{key} must not be empty")));
    }

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
    if value.is_finite() {
        Ok(FloatOrPercent { value, percent })
    } else {
        Err(SliceError::InvalidInput(format!(
            "{key} must contain finite FloatOrPercent values"
        )))
    }
}

fn float_or_percent_get_at(
    values: &[FloatOrPercent],
    key: &str,
    id: usize,
) -> Result<FloatOrPercent, SliceError> {
    values
        .get(id)
        .or_else(|| values.first())
        .copied()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must not be empty")))
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
