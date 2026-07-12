use serde_json::Value;

use crate::SliceError;

use super::super::{SliceOptions, parsing::parse_numeric_vector, registry};

pub(super) const SCALING_FACTOR: f64 = 0.000001;

impl SliceOptions {
    pub(super) fn validate_pattern_option(
        &self,
        errors: &mut std::collections::BTreeMap<String, String>,
        key: &str,
        is_valid: fn(&str) -> bool,
    ) -> Result<(), SliceError> {
        let value = self.string_or_default(key)?;
        if !is_valid(&value) {
            errors.insert(key.to_owned(), invalid_value_message(value));
        }
        Ok(())
    }

    pub(super) fn scalar_f64_or_default(&self, key: &str) -> Result<f64, SliceError> {
        match self.values.get(key) {
            Some(value) => scalar_f64(value, key),
            None => registry_default(key)?
                .parse::<f64>()
                .map_err(|_| SliceError::InvalidInput(format!("{key} default must be a number"))),
        }
    }

    pub(super) fn scalar_i64_or_default(&self, key: &str) -> Result<i64, SliceError> {
        match self.values.get(key) {
            Some(value) => scalar_i64(value, key),
            None => registry_default(key)?
                .parse::<i64>()
                .map_err(|_| SliceError::InvalidInput(format!("{key} default must be an integer"))),
        }
    }

    pub(super) fn numeric_vector_or_default(&self, key: &str) -> Result<Vec<f64>, SliceError> {
        match self.values.get(key) {
            Some(value) => parse_numeric_vector(key, value),
            None => parse_numeric_vector(key, &Value::String(registry_default(key)?.to_owned())),
        }
    }

    pub(super) fn bool_or_default(&self, key: &str) -> Result<bool, SliceError> {
        match self.values.get(key) {
            Some(value) => bool_value(value, key),
            None => registry_bool(key),
        }
    }

    pub(super) fn string_or_default(&self, key: &str) -> Result<String, SliceError> {
        match self.values.get(key) {
            Some(Value::String(value)) => Ok(value.clone()),
            Some(_) => Err(SliceError::InvalidInput(format!("{key} must be a string"))),
            None => Ok(registry_default(key)?.to_owned()),
        }
    }

    pub(super) fn bool_vector_or_default(&self, key: &str) -> Result<Vec<bool>, SliceError> {
        match self.values.get(key) {
            Some(Value::Bool(value)) => Ok(vec![*value]),
            Some(Value::Array(values)) => {
                values.iter().map(|value| bool_value(value, key)).collect()
            }
            Some(_) => Err(SliceError::InvalidInput(format!(
                "{key} must be a bool or bool array"
            ))),
            None => Ok(vec![registry_bool(key)?]),
        }
    }
}

pub(super) fn registry_default(key: &str) -> Result<&'static str, SliceError> {
    registry::option_definition(key)
        .map(|definition| definition.default_value)
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} has no option definition")))
}

fn scalar_f64(value: &Value, key: &str) -> Result<f64, SliceError> {
    match value {
        Value::Number(number) => number
            .as_f64()
            .filter(|value| value.is_finite())
            .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a finite number"))),
        Value::String(text) => text
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
            .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a finite number"))),
        _ => Err(SliceError::InvalidInput(format!(
            "{key} must be a finite number"
        ))),
    }
}

fn scalar_i64(value: &Value, key: &str) -> Result<i64, SliceError> {
    match value {
        Value::Number(number) => number
            .as_i64()
            .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be an integer"))),
        Value::String(text) => text
            .parse::<i64>()
            .map_err(|_| SliceError::InvalidInput(format!("{key} must be an integer"))),
        _ => Err(SliceError::InvalidInput(format!(
            "{key} must be an integer"
        ))),
    }
}

fn bool_value(value: &Value, key: &str) -> Result<bool, SliceError> {
    match value {
        Value::Bool(value) => Ok(*value),
        _ => Err(SliceError::InvalidInput(format!("{key} must be a bool"))),
    }
}

fn registry_bool(key: &str) -> Result<bool, SliceError> {
    registry_default(key)?
        .parse::<bool>()
        .map_err(|_| SliceError::InvalidInput(format!("{key} default must be a bool")))
}

pub(super) fn invalid_value_message(value: impl ToString) -> String {
    format!("invalid value {}", value.to_string())
}

pub(super) fn invalid_float_value_message(value: f64) -> String {
    format!("invalid value {value:.6}")
}

pub(super) fn serialize_numbers(values: &[f64]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
