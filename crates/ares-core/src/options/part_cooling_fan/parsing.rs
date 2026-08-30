use super::super::temperature_vector;

use crate::SliceError;
use serde_json::Value;

pub(super) fn scalar_percent(
    key: &str,
    value: Option<&Value>,
    default: u8,
) -> Result<u8, SliceError> {
    let Some(value) = value else {
        return Ok(default);
    };
    let Value::Number(number) = value else {
        return Err(invalid(key, "must be an integer percent from 0 to 100"));
    };
    let Some(value) = number.as_f64() else {
        return Err(invalid(key, "must be an integer percent from 0 to 100"));
    };
    if value.fract() == 0.0 && (0.0..=100.0).contains(&value) {
        Ok(value as u8)
    } else {
        Err(invalid(key, "must be an integer percent from 0 to 100"))
    }
}

pub(super) fn first_percent(
    key: &str,
    value: Option<&Value>,
    default: u8,
) -> Result<f64, SliceError> {
    let Some(value) = value else {
        return Ok(f64::from(default));
    };
    let value = parse_float_vector(key, value)?
        .into_iter()
        .next()
        .ok_or_else(|| invalid(key, "must not be empty"))?;
    if value.is_finite() && (0.0..=100.0).contains(&value) {
        Ok(value)
    } else {
        Err(invalid(key, "must be a percent from 0 to 100"))
    }
}

pub(super) fn first_integer(
    key: &str,
    value: Option<&Value>,
    default: u32,
) -> Result<u32, SliceError> {
    let Some(value) = value else {
        return Ok(default);
    };
    temperature_vector::parse_integer_vector(key, value)?
        .into_iter()
        .next()
        .ok_or_else(|| invalid(key, "must not be empty"))
}

pub(super) fn first_percent_integer(
    key: &str,
    value: Option<&Value>,
    default: u8,
) -> Result<u8, SliceError> {
    let value = first_integer(key, value, u32::from(default))?;
    u8::try_from(value)
        .ok()
        .filter(|value| *value <= 100)
        .ok_or_else(|| invalid(key, "must be an integer percent from 0 to 100"))
}

pub(super) fn first_range_f64(
    key: &str,
    value: Option<&Value>,
    default: f64,
    min: f64,
    max: f64,
) -> Result<f64, SliceError> {
    let Some(value) = value else {
        return Ok(default);
    };
    let value = parse_float_vector(key, value)?
        .into_iter()
        .next()
        .ok_or_else(|| invalid(key, "must not be empty"))?;
    if value.is_finite() && (min..=max).contains(&value) {
        Ok(value)
    } else {
        Err(invalid(key, "is out of range"))
    }
}

pub(super) fn first_signed_role_fan_speed_value(
    key: &str,
    value: Option<&Value>,
    default: i32,
) -> Result<i32, SliceError> {
    let Some(value) = value else {
        return Ok(default);
    };
    let values = parse_float_vector(key, value)?;
    if values.is_empty() {
        return Err(invalid(key, "must not be empty"));
    }
    if values.iter().any(|value| {
        value.fract() != 0.0
            || *value < f64::from(i32::MIN)
            || *value > f64::from(i32::MAX)
    }) {
        return Err(invalid(key, "must contain integer values"));
    }
    let values = values
        .into_iter()
        .map(|value| value as i32)
        .collect::<Vec<_>>();
    if values.iter().any(|value| !(-1..=100).contains(value)) {
        return Err(invalid(
            key,
            "must be -1 or an integer percent from 0 to 100",
        ));
    }
    Ok(values[0])
}

fn parse_float_vector(key: &str, value: &Value) -> Result<Vec<f64>, SliceError> {
    match value {
        Value::Number(number) => parse_float_number(key, number).map(|value| vec![value]),
        Value::String(text) => parse_float_text(key, text),
        Value::Array(values) => {
            if values.is_empty() {
                return Err(invalid(key, "must not be empty"));
            }
            values
                .iter()
                .map(|value| match value {
                    Value::Number(number) => parse_float_number(key, number),
                    _ => Err(invalid(key, "must contain numeric values")),
                })
                .collect()
        }
        _ => Err(invalid(key, "must be a number or number list")),
    }
}

fn parse_float_number(key: &str, number: &serde_json::Number) -> Result<f64, SliceError> {
    number
        .as_f64()
        .filter(|value| value.is_finite())
        .ok_or_else(|| invalid(key, "must contain finite numbers"))
}

fn parse_float_text(key: &str, text: &str) -> Result<Vec<f64>, SliceError> {
    let parts = text.split([';', ',']).map(str::trim).collect::<Vec<_>>();
    if parts.iter().any(|part| part.is_empty()) {
        return Err(invalid(key, "must not be empty"));
    }
    parts
        .into_iter()
        .map(|part| {
            part.parse::<f64>()
                .ok()
                .filter(|value| value.is_finite())
                .ok_or_else(|| invalid(key, "must contain finite numbers"))
        })
        .collect()
}

fn invalid(key: &str, reason: &str) -> SliceError {
    SliceError::InvalidInput(format!("{key} {reason}"))
}
