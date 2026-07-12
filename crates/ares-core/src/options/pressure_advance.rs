use serde_json::Value;

use super::{SliceOptions, parsing};
use crate::SliceError;

const ENABLE_KEY: &str = "enable_pressure_advance";
const PRESSURE_KEY: &str = "pressure_advance";
const ADAPTIVE_KEY: &str = "adaptive_pressure_advance";
const ADAPTIVE_BRIDGE_KEY: &str = "adaptive_pressure_advance_bridges";
const DEFAULT_ENABLE: bool = false;
const DEFAULT_PRESSURE_ADVANCE: f64 = 0.02;
const DEFAULT_ADAPTIVE: bool = false;
const DEFAULT_BRIDGE_PRESSURE_ADVANCE: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PressureAdvanceControl {
    enabled: bool,
    value: f64,
    adaptive_enabled: bool,
    bridge_value: f64,
}

impl PressureAdvanceControl {
    pub(crate) const fn new(
        enabled: bool,
        value: f64,
        adaptive_enabled: bool,
        bridge_value: f64,
    ) -> Self {
        Self {
            enabled,
            value,
            adaptive_enabled,
            bridge_value,
        }
    }

    pub(crate) const fn value(self) -> Option<f64> {
        if self.enabled { Some(self.value) } else { None }
    }

    pub(crate) const fn bridge_value(self) -> Option<f64> {
        if self.enabled && self.adaptive_enabled && self.bridge_value > 0.0 {
            Some(self.bridge_value)
        } else {
            None
        }
    }
}

impl SliceOptions {
    pub(crate) fn pressure_advance_control(&self) -> Result<PressureAdvanceControl, SliceError> {
        Ok(PressureAdvanceControl::new(
            first_bool(ENABLE_KEY, self.values().get(ENABLE_KEY), DEFAULT_ENABLE)?,
            pressure_advance(
                PRESSURE_KEY,
                self.values().get(PRESSURE_KEY),
                DEFAULT_PRESSURE_ADVANCE,
            )?,
            first_bool(
                ADAPTIVE_KEY,
                self.values().get(ADAPTIVE_KEY),
                DEFAULT_ADAPTIVE,
            )?,
            pressure_advance(
                ADAPTIVE_BRIDGE_KEY,
                self.values().get(ADAPTIVE_BRIDGE_KEY),
                DEFAULT_BRIDGE_PRESSURE_ADVANCE,
            )?,
        ))
    }
}

fn first_bool(key: &str, value: Option<&Value>, default: bool) -> Result<bool, SliceError> {
    let Some(value) = value else {
        return Ok(default);
    };
    match value {
        Value::Bool(value) => Ok(*value),
        Value::Array(values) => match values.first() {
            Some(Value::Bool(value)) => Ok(*value),
            Some(_) => Err(invalid(key, "must contain boolean values")),
            None => Err(invalid(key, "must not be empty")),
        },
        _ => Err(invalid(key, "must be a boolean or boolean list")),
    }
}

fn pressure_advance(key: &str, value: Option<&Value>, default: f64) -> Result<f64, SliceError> {
    let Some(value) = value else {
        return Ok(default);
    };
    let value = match value {
        Value::Array(values) => values
            .first()
            .ok_or_else(|| invalid(key, "must not be empty"))
            .and_then(|value| parse_pressure_advance_value(key, value))?,
        _ => parsing::parse_numeric_vector(key, value)?
            .into_iter()
            .next()
            .ok_or_else(|| invalid(key, "must not be empty"))?,
    };
    if value.is_finite() && (0.0..=2.0).contains(&value) {
        Ok(value)
    } else {
        Err(invalid(key, "must be from 0 to 2"))
    }
}

fn parse_pressure_advance_value(key: &str, value: &Value) -> Result<f64, SliceError> {
    match value {
        Value::Number(number) => number
            .as_f64()
            .ok_or_else(|| invalid(key, "must be numeric")),
        Value::String(value) => value
            .parse::<f64>()
            .map_err(|_| invalid(key, "must be numeric")),
        _ => Err(invalid(key, "must be numeric")),
    }
}

fn invalid(key: &str, reason: &str) -> SliceError {
    SliceError::InvalidInput(format!("{key} {reason}"))
}
