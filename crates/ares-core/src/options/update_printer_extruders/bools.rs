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
    stride: usize,
    allow_nil: bool,
) -> Result<Vec<Value>, SliceError> {
    let values = bool_vector(value, key, allow_nil)?;
    let mut copied = Vec::with_capacity(variant_indices.len() * stride);
    for variant_index in variant_indices {
        for offset in 0..stride {
            let source_index = super::source_index(*variant_index, stride, offset)?;
            copied.push(bool_value(bool_get_at(&values, key, source_index)?));
        }
    }
    Ok(copied)
}

fn bool_vector(value: &Value, key: &str, allow_nil: bool) -> Result<Vec<NullableBool>, SliceError> {
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a bool vector")))?;
    if values.is_empty() {
        return Err(SliceError::InvalidInput(format!("{key} must not be empty")));
    }

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

fn bool_get_at(values: &[NullableBool], key: &str, id: usize) -> Result<NullableBool, SliceError> {
    values
        .get(id)
        .or_else(|| values.first())
        .copied()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must not be empty")))
}

fn bool_value(value: NullableBool) -> Value {
    match value {
        NullableBool::Nil => Value::String("nil".to_owned()),
        NullableBool::Value(value) => Value::Bool(value),
    }
}
