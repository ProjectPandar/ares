use serde_json::Value;

use super::RetractLiftEnforce;
use crate::{SliceError, options::parsing::parse_numeric_vector};

pub(super) fn first_bool(
    key: &str,
    value: Option<&Value>,
    default: bool,
) -> Result<bool, SliceError> {
    match value {
        None => Ok(default),
        Some(Value::Bool(value)) => Ok(*value),
        Some(Value::Array(values)) => {
            if values.is_empty() {
                return Err(SliceError::InvalidInput(format!("{key} must not be empty")));
            }
            if values.iter().any(|value| !matches!(value, Value::Bool(_))) {
                return Err(SliceError::InvalidInput(format!(
                    "{key} must contain only booleans"
                )));
            }
            Ok(values[0].as_bool().unwrap())
        }
        Some(_) => Err(SliceError::InvalidInput(format!(
            "{key} must be a boolean or boolean list"
        ))),
    }
}

pub(super) fn orca_serialized_bools(
    key: &str,
    value: Option<&Value>,
    default: bool,
) -> Result<Vec<bool>, SliceError> {
    match value {
        None => Ok(vec![default]),
        Some(Value::Bool(value)) => Ok(vec![*value]),
        Some(Value::Array(values)) => {
            if values.is_empty() {
                return Err(SliceError::InvalidInput(format!("{key} must not be empty")));
            }
            if values.iter().any(|value| !matches!(value, Value::Bool(_))) {
                return Err(SliceError::InvalidInput(format!(
                    "{key} must contain only booleans"
                )));
            }
            Ok(values.iter().map(|value| value.as_bool().unwrap()).collect())
        }
        Some(Value::String(text)) => serialized_bool_tokens(key, text),
        Some(_) => Err(SliceError::InvalidInput(format!(
            "{key} must be a boolean or comma-separated boolean list"
        ))),
    }
}

pub(super) fn orca_serialized_nullable_bools(
    key: &str,
    value: Option<&Value>,
    default: bool,
) -> Result<Vec<Option<bool>>, SliceError> {
    match value {
        None => Ok(vec![Some(default)]),
        Some(Value::Bool(value)) => Ok(vec![Some(*value)]),
        Some(Value::Null) => Ok(vec![None]),
        Some(Value::Array(values)) => {
            if values.is_empty() {
                return Err(SliceError::InvalidInput(format!("{key} must not be empty")));
            }
            values
                .iter()
                .map(|value| match value {
                    Value::Bool(value) => Ok(Some(*value)),
                    Value::Null => Ok(None),
                    _ => Err(SliceError::InvalidInput(format!(
                        "{key} must contain only booleans or nil"
                    ))),
                })
                .collect()
        }
        Some(Value::String(text)) => serialized_nullable_bool_tokens(key, text),
        Some(_) => Err(SliceError::InvalidInput(format!(
            "{key} must be a boolean, nil, or comma-separated nullable boolean list"
        ))),
    }
}

pub(super) fn orca_serialized_nullable_numbers(
    key: &str,
    value: &Value,
) -> Result<Vec<Option<f64>>, SliceError> {
    match value {
        Value::Number(number) => number
            .as_f64()
            .map(|value| vec![Some(value)])
            .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a finite number"))),
        Value::Null => Ok(vec![None]),
        Value::Array(values) => {
            if values.is_empty() {
                return Err(SliceError::InvalidInput(format!("{key} must not be empty")));
            }
            values
                .iter()
                .map(|value| match value {
                    Value::Number(number) => number.as_f64().map(Some).ok_or_else(|| {
                        SliceError::InvalidInput(format!("{key} must contain only numbers or nil"))
                    }),
                    Value::Null => Ok(None),
                    _ => Err(SliceError::InvalidInput(format!(
                        "{key} must contain only numbers or nil"
                    ))),
                })
                .collect()
        }
        Value::String(text) => serialized_nullable_number_tokens(key, text),
        _ => Err(SliceError::InvalidInput(format!(
            "{key} must be a number, nil, or comma-separated nullable number list"
        ))),
    }
}

pub(super) fn firmware_bool(
    key: &str,
    value: Option<&Value>,
    default: bool,
) -> Result<bool, SliceError> {
    match value {
        None => Ok(default),
        Some(Value::Bool(value)) => Ok(*value),
        Some(_) => Err(SliceError::InvalidInput(format!("{key} must be a boolean"))),
    }
}

pub(super) fn first_non_negative_f64(
    key: &str,
    value: Option<&Value>,
    default: f64,
) -> Result<f64, SliceError> {
    let Some(value) = value else {
        return Ok(default);
    };
    let values = parse_numeric_vector(key, value)?;
    let value = values[0];
    if value.is_finite() && value >= 0.0 {
        Ok(value)
    } else {
        Err(SliceError::InvalidInput(format!(
            "{key} must contain non-negative finite numbers"
        )))
    }
}

pub(super) fn first_non_negative_f64_all_values(
    key: &str,
    value: Option<&Value>,
    default: f64,
) -> Result<f64, SliceError> {
    let Some(value) = value else {
        return Ok(default);
    };
    let values = parse_numeric_vector(key, value)?;
    if values.iter().all(|value| value.is_finite() && *value >= 0.0) {
        Ok(values[0])
    } else {
        Err(SliceError::InvalidInput(format!(
            "{key} must contain non-negative finite numbers"
        )))
    }
}

pub(super) fn first_percent_fraction_all_values(
    key: &str,
    value: Option<&Value>,
    default_percent: f64,
) -> Result<f64, SliceError> {
    let Some(value) = value else {
        return Ok(default_percent / 100.0);
    };
    let values = parse_numeric_vector(key, value)?;
    if values
        .iter()
        .all(|value| value.is_finite() && (0.0..=100.0).contains(value))
    {
        Ok(values[0] / 100.0)
    } else {
        Err(SliceError::InvalidInput(format!(
            "{key} must be a percent from 0 to 100"
        )))
    }
}

pub(super) fn first_nullable_percent_fraction_all_values(
    key: &str,
    value: Option<&Value>,
    default_fraction: f64,
) -> Result<f64, SliceError> {
    let Some(value) = value else {
        return Ok(default_fraction);
    };
    let values = orca_serialized_nullable_numbers(key, value)?;
    if values.iter().all(|value| {
        value.is_none_or(|value| value.is_finite() && (0.0..=100.0).contains(&value))
    }) {
        Ok(values[0]
            .map(|value| value / 100.0)
            .unwrap_or(default_fraction))
    } else {
        Err(SliceError::InvalidInput(format!(
            "{key} must contain percents from 0 to 100 or nil"
        )))
    }
}

pub(super) fn first_nullable_non_negative_f64_all_values(
    key: &str,
    value: Option<&Value>,
    default: f64,
) -> Result<f64, SliceError> {
    let Some(value) = value else {
        return Ok(default);
    };
    let values = orca_serialized_nullable_numbers(key, value)?;
    if values
        .iter()
        .all(|value| value.is_none_or(|value| value.is_finite() && value >= 0.0))
    {
        Ok(values[0].unwrap_or(default))
    } else {
        Err(SliceError::InvalidInput(format!(
            "{key} must contain non-negative finite numbers or nil"
        )))
    }
}

pub(super) fn retract_lift_enforce(
    key: &str,
    value: Option<&Value>,
) -> Result<RetractLiftEnforce, SliceError> {
    match value {
        None => Ok(RetractLiftEnforce::AllSurfaces),
        Some(Value::String(text)) => retract_lift_enforce_token(key, text),
        Some(Value::Array(values)) => {
            if values.is_empty() {
                return Err(SliceError::InvalidInput(format!("{key} must not be empty")));
            }
            let parsed = values
                .iter()
                .map(|value| match value {
                    Value::String(text) => retract_lift_enforce_token(key, text),
                    _ => Err(SliceError::InvalidInput(format!(
                        "{key} must contain only strings"
                    ))),
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(parsed[0])
        }
        Some(_) => Err(SliceError::InvalidInput(format!(
            "{key} must be a string or string list"
        ))),
    }
}

pub(super) fn first_nullable_retract_lift_enforce(
    key: &str,
    value: Option<&Value>,
    default: RetractLiftEnforce,
) -> Result<RetractLiftEnforce, SliceError> {
    let Some(value) = value else {
        return Ok(default);
    };
    let values = nullable_retract_lift_enforce_values(key, value)?;
    Ok(values[0].unwrap_or(default))
}

fn nullable_retract_lift_enforce_values(
    key: &str,
    value: &Value,
) -> Result<Vec<Option<RetractLiftEnforce>>, SliceError> {
    match value {
        Value::String(text) => serialized_nullable_retract_lift_enforce_tokens(key, text),
        Value::Null => Ok(vec![None]),
        Value::Array(values) => {
            if values.is_empty() {
                return Err(SliceError::InvalidInput(format!("{key} must not be empty")));
            }
            values
                .iter()
                .map(|value| match value {
                    Value::String(text) => retract_lift_enforce_token(key, text).map(Some),
                    Value::Null => Ok(None),
                    _ => Err(SliceError::InvalidInput(format!(
                        "{key} must contain only strings or nil"
                    ))),
                })
                .collect()
        }
        _ => Err(SliceError::InvalidInput(format!(
            "{key} must be a string, nil, or comma-separated nullable enum list"
        ))),
    }
}

fn serialized_bool_tokens(key: &str, text: &str) -> Result<Vec<bool>, SliceError> {
    let parts = text.split(',').map(str::trim).collect::<Vec<_>>();
    if parts.iter().any(|part| part.is_empty()) {
        return Err(SliceError::InvalidInput(format!("{key} must not be empty")));
    }
    if parts.iter().any(|part| !matches!(*part, "1" | "0")) {
        return Err(SliceError::InvalidInput(format!(
            "{key} must contain only 1 or 0"
        )));
    }
    Ok(parts.iter().map(|part| *part == "1").collect())
}

fn serialized_nullable_bool_tokens(
    key: &str,
    text: &str,
) -> Result<Vec<Option<bool>>, SliceError> {
    let parts = text.split(',').map(str::trim).collect::<Vec<_>>();
    if parts.iter().any(|part| part.is_empty()) {
        return Err(SliceError::InvalidInput(format!("{key} must not be empty")));
    }
    parts
        .iter()
        .map(|part| match *part {
            "nil" => Ok(None),
            "1" => Ok(Some(true)),
            "0" => Ok(Some(false)),
            _ => Err(SliceError::InvalidInput(format!(
                "{key} must contain only nil, 1, or 0"
            ))),
        })
        .collect()
}

fn serialized_nullable_number_tokens(
    key: &str,
    text: &str,
) -> Result<Vec<Option<f64>>, SliceError> {
    let parts = text.split(',').map(str::trim).collect::<Vec<_>>();
    if parts.iter().any(|part| part.is_empty()) {
        return Err(SliceError::InvalidInput(format!("{key} must not be empty")));
    }
    parts
        .iter()
        .map(|part| {
            if *part == "nil" {
                Ok(None)
            } else {
                part.parse::<f64>().map(Some).map_err(|_| {
                    SliceError::InvalidInput(format!("{key} must contain only numbers or nil"))
                })
            }
        })
        .collect()
}

fn serialized_nullable_retract_lift_enforce_tokens(
    key: &str,
    text: &str,
) -> Result<Vec<Option<RetractLiftEnforce>>, SliceError> {
    let parts = text.split(',').map(str::trim).collect::<Vec<_>>();
    if parts.iter().any(|part| part.is_empty()) {
        return Err(SliceError::InvalidInput(format!("{key} must not be empty")));
    }
    parts
        .iter()
        .map(|part| {
            if *part == "nil" {
                Ok(None)
            } else {
                retract_lift_enforce_token(key, part).map(Some)
            }
        })
        .collect()
}

fn retract_lift_enforce_token(
    key: &str,
    text: &str,
) -> Result<RetractLiftEnforce, SliceError> {
    match text {
        "All Surfaces" => Ok(RetractLiftEnforce::AllSurfaces),
        "Top Only" => Ok(RetractLiftEnforce::TopOnly),
        "Bottom Only" => Ok(RetractLiftEnforce::BottomOnly),
        "Top and Bottom" => Ok(RetractLiftEnforce::TopAndBottom),
        _ => Err(SliceError::InvalidInput(format!(
            "{key} must be All Surfaces, Top Only, Bottom Only, or Top and Bottom"
        ))),
    }
}
