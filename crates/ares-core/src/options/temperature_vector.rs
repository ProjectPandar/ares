use serde_json::Value;

use crate::SliceError;

pub(crate) fn parse_integer_vector(key: &str, value: &Value) -> Result<Vec<u32>, SliceError> {
    match value {
        Value::Number(number) => parse_number(key, number),
        Value::String(text) => parse_text(key, text),
        Value::Array(values) => {
            if values.is_empty() {
                return Err(invalid(key, "must not be empty"));
            }
            values
                .iter()
                .map(|value| match value {
                    Value::Number(number) => {
                        parse_number(key, number).map(|mut values| values.remove(0))
                    }
                    _ => Err(invalid(key, "must contain integer values")),
                })
                .collect()
        }
        _ => Err(invalid(key, "must be an integer or integer list")),
    }
}

fn parse_number(key: &str, number: &serde_json::Number) -> Result<Vec<u32>, SliceError> {
    number
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
        .map(|value| vec![value])
        .ok_or_else(|| invalid(key, "must contain non-negative integers"))
}

fn parse_text(key: &str, text: &str) -> Result<Vec<u32>, SliceError> {
    let parts = text.split([';', ',']).map(str::trim).collect::<Vec<_>>();
    if parts.iter().any(|part| part.is_empty()) {
        return Err(invalid(key, "must not be empty"));
    }
    parts
        .into_iter()
        .map(|part| parse_single_text(key, part))
        .collect()
}

fn parse_single_text(key: &str, text: &str) -> Result<u32, SliceError> {
    text.parse::<u32>()
        .map_err(|_| invalid(key, "must contain non-negative integers"))
}

fn invalid(key: &str, reason: &str) -> SliceError {
    SliceError::InvalidInput(format!("{key} {reason}"))
}
