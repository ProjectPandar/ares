use std::collections::BTreeMap;

use serde_json::Value;

use crate::SliceError;

const DEFAULT_SLOW_DOWN_LAYERS: u32 = 0;
const DEFAULT_DONT_SLOW_DOWN_OUTER_WALL: bool = false;
const DEFAULT_SLOW_DOWN_FOR_LAYER_COOLING: bool = true;
const DEFAULT_SLOW_DOWN_LAYER_TIME: f64 = 5.0;
const DEFAULT_SLOW_DOWN_MIN_SPEED: f64 = 10.0;

pub fn parse_slow_down_layers(values: &BTreeMap<String, Value>) -> Result<u32, SliceError> {
    let Some(value) = values.get("slow_down_layers") else {
        return Ok(DEFAULT_SLOW_DOWN_LAYERS);
    };
    let parsed = match value {
        Value::Number(number) => number.as_u64().and_then(|value| u32::try_from(value).ok()),
        Value::String(text) => text.parse::<u32>().ok(),
        _ => None,
    };
    parsed.ok_or_else(|| {
        SliceError::InvalidInput("slow_down_layers must be a non-negative integer".to_owned())
    })
}

pub fn parse_dont_slow_down_outer_wall(
    values: &BTreeMap<String, Value>,
) -> Result<bool, SliceError> {
    let Some(value) = values.get("dont_slow_down_outer_wall") else {
        return Ok(DEFAULT_DONT_SLOW_DOWN_OUTER_WALL);
    };
    first_bool("dont_slow_down_outer_wall", value)
}

pub fn parse_slow_down_for_layer_cooling(
    values: &BTreeMap<String, Value>,
) -> Result<bool, SliceError> {
    let Some(value) = values.get("slow_down_for_layer_cooling") else {
        return Ok(DEFAULT_SLOW_DOWN_FOR_LAYER_COOLING);
    };
    first_bool("slow_down_for_layer_cooling", value)
}

pub fn parse_slow_down_layer_time(values: &BTreeMap<String, Value>) -> Result<f64, SliceError> {
    let Some(value) = values.get("slow_down_layer_time") else {
        return Ok(DEFAULT_SLOW_DOWN_LAYER_TIME);
    };
    let value = first_number("slow_down_layer_time", value)?;
    if value.is_finite() && (0.0..=1000.0).contains(&value) {
        Ok(value)
    } else {
        Err(SliceError::InvalidInput(
            "slow_down_layer_time is out of range".to_owned(),
        ))
    }
}

pub fn parse_slow_down_min_speed(values: &BTreeMap<String, Value>) -> Result<f64, SliceError> {
    let Some(value) = values.get("slow_down_min_speed") else {
        return Ok(DEFAULT_SLOW_DOWN_MIN_SPEED);
    };
    let value = first_number("slow_down_min_speed", value)?;
    if value.is_finite() && value >= 0.0 {
        Ok(value)
    } else {
        Err(SliceError::InvalidInput(
            "slow_down_min_speed must be non-negative".to_owned(),
        ))
    }
}

fn first_bool(key: &str, value: &Value) -> Result<bool, SliceError> {
    match value {
        Value::Bool(value) => Ok(*value),
        Value::Array(values) => values
            .first()
            .and_then(Value::as_bool)
            .ok_or_else(|| invalid_bool(key)),
        _ => Err(invalid_bool(key)),
    }
}

fn invalid_bool(key: &str) -> SliceError {
    SliceError::InvalidInput(format!("{key} must be a boolean or boolean list"))
}

fn first_number(key: &str, value: &Value) -> Result<f64, SliceError> {
    match value {
        Value::Number(_) | Value::String(_) => number_value(key, value),
        Value::Array(values) => values
            .first()
            .ok_or_else(|| SliceError::InvalidInput(format!("{key} must not be empty")))
            .and_then(|value| number_value(key, value)),
        _ => Err(SliceError::InvalidInput(format!("{key} must be a number"))),
    }
}

fn number_value(key: &str, value: &Value) -> Result<f64, SliceError> {
    match value {
        Value::Number(number) => number
            .as_f64()
            .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a number"))),
        Value::String(text) => text
            .parse::<f64>()
            .map_err(|_| SliceError::InvalidInput(format!("{key} must be a number"))),
        _ => Err(SliceError::InvalidInput(format!("{key} must be a number"))),
    }
}
