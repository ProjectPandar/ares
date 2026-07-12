use serde_json::Value;

use crate::SliceError;

pub(super) fn parse_extrusion_width_text(text: &str, nozzle_diameter: f64) -> Option<f64> {
    let text = text.trim();
    if let Some(percent) = text.strip_suffix('%') {
        percent
            .trim()
            .parse::<f64>()
            .ok()
            .map(|value| value / 100.0 * nozzle_diameter)
    } else {
        text.parse().ok()
    }
}

pub(crate) fn parse_positive_numeric_or_percent_over_base(
    key: &str,
    value: &Value,
    base: f64,
) -> Result<f64, SliceError> {
    let value = match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => parse_numeric_or_percent_text(text, base),
        _ => None,
    }
    .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a number")))?;
    if value.is_finite() && value > 0.0 {
        Ok(value)
    } else {
        Err(SliceError::InvalidInput(format!("{key} must be positive")))
    }
}

pub(crate) fn parse_non_negative_numeric_or_percent_over_base(
    key: &str,
    value: &Value,
    base: f64,
) -> Result<f64, SliceError> {
    let value = match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => parse_numeric_or_percent_text(text, base),
        _ => None,
    }
    .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a number")))?;
    if value.is_finite() && value >= 0.0 {
        Ok(value)
    } else {
        Err(SliceError::InvalidInput(format!(
            "{key} must be non-negative"
        )))
    }
}

pub(crate) fn parse_range_f64(
    key: &str,
    value: Option<&Value>,
    default: f64,
    min: f64,
    max: f64,
) -> Result<f64, SliceError> {
    let Some(value) = value else {
        return Ok(default);
    };
    let value = match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
    .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a number")))?;
    if value.is_finite() && value >= min && value <= max {
        Ok(value)
    } else {
        Err(SliceError::InvalidInput(format!("{key} is out of range")))
    }
}

pub(crate) fn parse_positive_number_or_string(
    key: &str,
    value: Option<&Value>,
    default: f64,
) -> Result<f64, SliceError> {
    let Some(value) = value else {
        return Ok(default);
    };
    let value = match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
    .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a number")))?;
    if value.is_finite() && value > 0.0 {
        Ok(value)
    } else {
        Err(SliceError::InvalidInput(format!("{key} must be positive")))
    }
}

fn parse_numeric_or_percent_text(text: &str, base: f64) -> Option<f64> {
    let text = text.trim();
    if let Some(percent) = text.strip_suffix('%') {
        percent
            .trim()
            .parse::<f64>()
            .ok()
            .map(|value| value / 100.0 * base)
    } else {
        text.parse().ok()
    }
}

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
