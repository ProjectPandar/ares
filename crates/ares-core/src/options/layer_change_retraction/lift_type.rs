use std::collections::BTreeMap;

use serde_json::Value;

use super::ZHopLiftMode;
use crate::{SliceError, options::parsing::parse_numeric_vector};

const Z_HOP_TYPES: &str = "z_hop_types";
const FILAMENT_Z_HOP_TYPES: &str = "filament_z_hop_types";
const TRAVEL_SLOPE: &str = "travel_slope";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ZHopLiftType {
    Auto,
    Normal,
    Slope,
    Spiral,
}

pub(super) fn parse_z_hop_lift_config(
    values: &BTreeMap<String, Value>,
) -> Result<ZHopLiftMode, SliceError> {
    let unprefixed = first_z_hop_type(values.get(Z_HOP_TYPES), Z_HOP_TYPES, ZHopLiftType::Slope)?;
    let effective =
        first_nullable_z_hop_type(values.get(FILAMENT_Z_HOP_TYPES), unprefixed)?.unwrap_or(unprefixed);
    let slope_radians = first_travel_slope(values.get(TRAVEL_SLOPE))?.to_radians();
    Ok(match effective {
        ZHopLiftType::Auto => ZHopLiftMode::Auto {
            radians: slope_radians,
        },
        ZHopLiftType::Normal => ZHopLiftMode::Normal,
        ZHopLiftType::Slope => ZHopLiftMode::Slope {
            radians: slope_radians,
        },
        ZHopLiftType::Spiral => ZHopLiftMode::Spiral {
            radians: slope_radians,
        },
    })
}

fn first_travel_slope(value: Option<&Value>) -> Result<f64, SliceError> {
    let Some(value) = value else {
        return Ok(3.0);
    };
    let values = parse_numeric_vector(TRAVEL_SLOPE, value)?;
    if values
        .iter()
        .all(|value| value.is_finite() && (1.0..=90.0).contains(value))
    {
        Ok(values[0])
    } else {
        Err(SliceError::InvalidInput(format!(
            "{TRAVEL_SLOPE} is out of range"
        )))
    }
}

fn first_z_hop_type(
    value: Option<&Value>,
    key: &str,
    default: ZHopLiftType,
) -> Result<ZHopLiftType, SliceError> {
    let Some(value) = value else {
        return Ok(default);
    };
    let values = z_hop_types(key, value)?;
    Ok(values[0])
}

fn first_nullable_z_hop_type(
    value: Option<&Value>,
    default: ZHopLiftType,
) -> Result<Option<ZHopLiftType>, SliceError> {
    let Some(value) = value else {
        return Ok(Some(default));
    };
    let values = nullable_z_hop_types(FILAMENT_Z_HOP_TYPES, value)?;
    Ok(values[0])
}

fn z_hop_types(key: &str, value: &Value) -> Result<Vec<ZHopLiftType>, SliceError> {
    match value {
        Value::String(text) => parse_z_hop_type_text(key, text),
        Value::Array(values) => values
            .iter()
            .map(|value| match value {
                Value::String(text) => z_hop_type(key, text),
                _ => Err(invalid_type(key)),
            })
            .collect::<Result<Vec<_>, _>>()
            .and_then(|values| non_empty(key, values)),
        _ => Err(invalid_type(key)),
    }
}

fn nullable_z_hop_types(key: &str, value: &Value) -> Result<Vec<Option<ZHopLiftType>>, SliceError> {
    match value {
        Value::String(text) => parse_nullable_z_hop_type_text(key, text),
        Value::Array(values) => values
            .iter()
            .map(|value| match value {
                Value::Null => Ok(None),
                Value::String(text) if text == "nil" => Ok(None),
                Value::String(text) => z_hop_type(key, text).map(Some),
                _ => Err(invalid_type(key)),
            })
            .collect::<Result<Vec<_>, _>>()
            .and_then(|values| non_empty(key, values)),
        Value::Null => Ok(vec![None]),
        _ => Err(invalid_type(key)),
    }
}

fn parse_z_hop_type_text(key: &str, text: &str) -> Result<Vec<ZHopLiftType>, SliceError> {
    text.split(',')
        .map(|token| z_hop_type(key, token.trim()))
        .collect::<Result<Vec<_>, _>>()
        .and_then(|values| non_empty(key, values))
}

fn parse_nullable_z_hop_type_text(
    key: &str,
    text: &str,
) -> Result<Vec<Option<ZHopLiftType>>, SliceError> {
    text.split(',')
        .map(|token| match token.trim() {
            "nil" => Ok(None),
            token => z_hop_type(key, token).map(Some),
        })
        .collect::<Result<Vec<_>, _>>()
        .and_then(|values| non_empty(key, values))
}

fn z_hop_type(key: &str, value: &str) -> Result<ZHopLiftType, SliceError> {
    match value {
        "Auto Lift" => Ok(ZHopLiftType::Auto),
        "Normal Lift" => Ok(ZHopLiftType::Normal),
        "Spiral Lift" => Ok(ZHopLiftType::Spiral),
        "Slope Lift" => Ok(ZHopLiftType::Slope),
        _ => Err(invalid_type(key)),
    }
}

fn non_empty<T>(key: &str, values: Vec<T>) -> Result<Vec<T>, SliceError> {
    if values.is_empty() {
        Err(invalid_type(key))
    } else {
        Ok(values)
    }
}

fn invalid_type(key: &str) -> SliceError {
    SliceError::InvalidInput(format!("{key} has invalid lift type"))
}
