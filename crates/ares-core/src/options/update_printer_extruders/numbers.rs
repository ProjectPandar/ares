use serde_json::{Number, Value};

use crate::SliceError;

#[derive(Clone, Copy)]
enum NullableNumber {
    Nil,
    Value(f64),
}

pub(super) fn copy_number_values(
    value: &Value,
    key: &str,
    variant_indices: &[usize],
    stride: usize,
    allow_nil: bool,
) -> Result<Vec<Value>, SliceError> {
    let values = number_vector(value, key, allow_nil)?;
    let mut copied = Vec::with_capacity(variant_indices.len() * stride);
    for variant_index in variant_indices {
        for offset in 0..stride {
            let source_index = super::source_index(*variant_index, stride, offset)?;
            copied.push(number_value(
                number_get_at(&values, key, source_index)?,
                key,
            )?);
        }
    }
    Ok(copied)
}

fn number_vector(
    value: &Value,
    key: &str,
    allow_nil: bool,
) -> Result<Vec<NullableNumber>, SliceError> {
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a number vector")))?;
    if values.is_empty() {
        return Err(SliceError::InvalidInput(format!("{key} must not be empty")));
    }

    values
        .iter()
        .map(|value| match value {
            Value::Number(number) => number
                .as_f64()
                .filter(|value| value.is_finite())
                .map(NullableNumber::Value)
                .ok_or_else(|| {
                    SliceError::InvalidInput(format!("{key} must contain finite numbers"))
                }),
            Value::String(text) if allow_nil && text == "nil" => Ok(NullableNumber::Nil),
            _ => Err(SliceError::InvalidInput(format!(
                "{key} must contain finite numbers"
            ))),
        })
        .collect()
}

fn number_get_at(
    values: &[NullableNumber],
    key: &str,
    id: usize,
) -> Result<NullableNumber, SliceError> {
    values
        .get(id)
        .or_else(|| values.first())
        .copied()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must not be empty")))
}

fn number_value(value: NullableNumber, key: &str) -> Result<Value, SliceError> {
    match value {
        NullableNumber::Nil => Ok(Value::String("nil".to_owned())),
        NullableNumber::Value(value) => Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| SliceError::InvalidInput(format!("{key} must contain finite numbers"))),
    }
}
