use serde_json::Value;

use crate::SliceError;

#[derive(Clone, Copy)]
enum NullableBool {
    Nil,
    Value(bool),
}

pub(super) fn copy_bool_values(
    value: &Value,
    key: &str,
    variant_indices: &[usize],
    allow_nil: bool,
) -> Result<Vec<Value>, SliceError> {
    let values = bool_vector(value, key, allow_nil)?;
    let mut copied = vec![Value::Bool(false); variant_indices.len()];
    for (filament_index, variant_index) in variant_indices.iter().enumerate() {
        if *variant_index < values.len() {
            copied[filament_index] = bool_value(values[*variant_index]);
        }
    }
    Ok(copied)
}

fn bool_vector(value: &Value, key: &str, allow_nil: bool) -> Result<Vec<NullableBool>, SliceError> {
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a bool vector")))?;
    values
        .iter()
        .map(|value| match value {
            Value::Bool(value) => Ok(NullableBool::Value(*value)),
            Value::String(text) if allow_nil && text == "nil" => Ok(NullableBool::Nil),
            _ => Err(SliceError::InvalidInput(format!(
                "{key} must contain bool values"
            ))),
        })
        .collect()
}

fn bool_value(value: NullableBool) -> Value {
    match value {
        NullableBool::Nil => Value::String("nil".to_owned()),
        NullableBool::Value(value) => Value::Bool(value),
    }
}
