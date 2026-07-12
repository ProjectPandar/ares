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
    allow_nil: bool,
) -> Result<Vec<Value>, SliceError> {
    let values = enum_vector(value, key, allow_nil)?;
    let mut copied = vec![Value::String(String::new()); variant_indices.len()];
    for (filament_index, variant_index) in variant_indices.iter().enumerate() {
        if *variant_index < values.len() {
            copied[filament_index] = enum_value(&values[*variant_index]);
        }
    }
    Ok(copied)
}

fn enum_vector(value: &Value, key: &str, allow_nil: bool) -> Result<Vec<NullableEnum>, SliceError> {
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be an enum vector")))?;
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

fn enum_value(value: &NullableEnum) -> Value {
    match value {
        NullableEnum::Nil => Value::String("nil".to_owned()),
        NullableEnum::Value(value) => Value::String(value.clone()),
    }
}
