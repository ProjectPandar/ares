use serde_json::Value;

use crate::SliceError;

pub(crate) fn parse_numeric_vector(key: &str, value: &Value) -> Result<Vec<f64>, SliceError> {
    match value {
        Value::Number(number) => number
            .as_f64()
            .map(|value| vec![value])
            .ok_or_else(|| SliceError::InvalidInput(format!("{key} must contain numbers"))),
        Value::String(text) => parse_numeric_text(key, text),
        Value::Array(values) => {
            if values.is_empty() {
                return Err(SliceError::InvalidInput(format!("{key} must not be empty")));
            }
            values
                .iter()
                .map(|value| match value {
                    Value::Number(number) => number.as_f64().ok_or_else(|| {
                        SliceError::InvalidInput(format!("{key} must contain numbers"))
                    }),
                    Value::String(text) => parse_single_number(key, text),
                    _ => Err(SliceError::InvalidInput(format!(
                        "{key} must contain only numeric values"
                    ))),
                })
                .collect()
        }
        _ => Err(SliceError::InvalidInput(format!(
            "{key} must be a number or numeric list"
        ))),
    }
}

fn parse_numeric_text(key: &str, text: &str) -> Result<Vec<f64>, SliceError> {
    let parts = text.split([';', ',']).map(str::trim).collect::<Vec<_>>();
    if parts.iter().any(|part| part.is_empty()) {
        return Err(SliceError::InvalidInput(format!("{key} must not be empty")));
    }
    parts
        .into_iter()
        .map(|part| parse_single_number(key, part))
        .collect()
}

fn parse_single_number(key: &str, text: &str) -> Result<f64, SliceError> {
    text.parse::<f64>()
        .map_err(|_| SliceError::InvalidInput(format!("{key} must contain numbers")))
}
