use super::SliceOptions;

use crate::{PrintPathRole, SliceError};
use serde_json::Value;

const FAN_SPEEDUP_TIME_KEY: &str = "fan_speedup_time";
const FAN_SPEEDUP_OVERHANGS_KEY: &str = "fan_speedup_overhangs";
const DEFAULT_FAN_SPEEDUP_TIME: f64 = 0.0;
const DEFAULT_FAN_SPEEDUP_OVERHANGS: bool = true;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FanSpeedupControl {
    time_s: f64,
    only_overhangs: bool,
}

impl FanSpeedupControl {
    pub(crate) const fn new(time_s: f64, only_overhangs: bool) -> Self {
        Self {
            time_s,
            only_overhangs,
        }
    }

    #[cfg(test)]
    pub(crate) const fn time_s(self) -> f64 {
        self.time_s
    }

    #[cfg(test)]
    pub(crate) const fn only_overhangs(self) -> bool {
        self.only_overhangs
    }

    pub(crate) const fn enabled(self) -> bool {
        self.time_s > 0.0
    }

    pub(crate) const fn applies_to_role(self, role: PrintPathRole) -> bool {
        if !self.enabled() {
            return false;
        }
        if !self.only_overhangs {
            return true;
        }
        matches!(
            role,
            PrintPathRole::Bridge
                | PrintPathRole::InternalBridge
                | PrintPathRole::OverhangPerimeter
        )
    }
}

impl SliceOptions {
    pub(crate) fn fan_speedup_control(&self) -> Result<FanSpeedupControl, SliceError> {
        let time_s = scalar_non_negative_f64(
            FAN_SPEEDUP_TIME_KEY,
            self.values().get(FAN_SPEEDUP_TIME_KEY),
            DEFAULT_FAN_SPEEDUP_TIME,
        )?;
        let only_overhangs = scalar_bool(
            FAN_SPEEDUP_OVERHANGS_KEY,
            self.values().get(FAN_SPEEDUP_OVERHANGS_KEY),
            DEFAULT_FAN_SPEEDUP_OVERHANGS,
        )?;
        Ok(FanSpeedupControl::new(time_s, only_overhangs))
    }
}

fn scalar_non_negative_f64(
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
    .ok_or_else(|| invalid(key, "must be a number"))?;
    if value.is_finite() && value >= 0.0 {
        Ok(value)
    } else {
        Err(invalid(key, "must be non-negative"))
    }
}

fn scalar_bool(key: &str, value: Option<&Value>, default: bool) -> Result<bool, SliceError> {
    let Some(value) = value else {
        return Ok(default);
    };
    value
        .as_bool()
        .ok_or_else(|| invalid(key, "must be a boolean"))
}

fn invalid(key: &str, reason: &str) -> SliceError {
    SliceError::InvalidInput(format!("{key} {reason}"))
}
