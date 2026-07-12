use serde_json::Value;

use crate::SliceError;

#[derive(Clone)]
enum NullableEnum {
    Nil,
    Value(String),
}

pub(super) fn copy_enum_values(
    value: &Value,
    key: &str,
    variant_indices: &[usize],
    stride: usize,
    allow_nil: bool,
) -> Result<Vec<Value>, SliceError> {
    let values = enum_vector(value, key, allow_nil)?;
    let mut copied = Vec::with_capacity(variant_indices.len() * stride);
    for variant_index in variant_indices {
        for offset in 0..stride {
            let source_index = super::source_index(*variant_index, stride, offset)?;
            copied.push(enum_value(enum_get_at(&values, key, source_index)?));
        }
    }
    Ok(copied)
}

fn enum_vector(value: &Value, key: &str, allow_nil: bool) -> Result<Vec<NullableEnum>, SliceError> {
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be an enum vector")))?;
    if values.is_empty() {
        return Err(SliceError::InvalidInput(format!("{key} must not be empty")));
    }

    values
        .iter()
        .map(|value| match value {
            Value::String(text) if text == "nil" && allow_nil => Ok(NullableEnum::Nil),
            Value::String(text) if text != "nil" => Ok(NullableEnum::Value(text.clone())),
            _ => Err(SliceError::InvalidInput(format!(
                "{key} must contain enum strings"
            ))),
        })
        .collect()
}

fn enum_get_at<'a>(
    values: &'a [NullableEnum],
    key: &str,
    id: usize,
) -> Result<&'a NullableEnum, SliceError> {
    values
        .get(id)
        .or_else(|| values.first())
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must not be empty")))
}

fn enum_value(value: &NullableEnum) -> Value {
    match value {
        NullableEnum::Nil => Value::String("nil".to_owned()),
        NullableEnum::Value(value) => Value::String(value.clone()),
    }
}
