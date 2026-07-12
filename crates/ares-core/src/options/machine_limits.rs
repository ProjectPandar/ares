use serde_json::Value;

use crate::SliceError;

use super::{SliceOptions, parsing};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MachineLimits {
    pub(crate) emit_to_gcode: bool,
    pub(crate) max_acceleration: [f64; 4],
    pub(crate) max_speed: [f64; 4],
    pub(crate) max_acceleration_extruding: f64,
    pub(crate) max_acceleration_retracting: f64,
    pub(crate) max_acceleration_travel: f64,
    pub(crate) max_jerk: [f64; 4],
    pub(crate) max_junction_deviation: f64,
}

impl SliceOptions {
    pub(crate) fn machine_limits(&self) -> Result<MachineLimits, SliceError> {
        Ok(MachineLimits {
            emit_to_gcode: bool_option(
                "emit_machine_limits_to_gcode",
                self.values().get("emit_machine_limits_to_gcode"),
                true,
            )?,
            max_acceleration: [
                first_non_negative(
                    "machine_max_acceleration_x",
                    self.values().get("machine_max_acceleration_x"),
                    1000.0,
                )?,
                first_non_negative(
                    "machine_max_acceleration_y",
                    self.values().get("machine_max_acceleration_y"),
                    1000.0,
                )?,
                first_non_negative(
                    "machine_max_acceleration_z",
                    self.values().get("machine_max_acceleration_z"),
                    500.0,
                )?,
                first_non_negative(
                    "machine_max_acceleration_e",
                    self.values().get("machine_max_acceleration_e"),
                    5000.0,
                )?,
            ],
            max_speed: [
                first_non_negative(
                    "machine_max_speed_x",
                    self.values().get("machine_max_speed_x"),
                    500.0,
                )?,
                first_non_negative(
                    "machine_max_speed_y",
                    self.values().get("machine_max_speed_y"),
                    500.0,
                )?,
                first_non_negative(
                    "machine_max_speed_z",
                    self.values().get("machine_max_speed_z"),
                    12.0,
                )?,
                first_non_negative(
                    "machine_max_speed_e",
                    self.values().get("machine_max_speed_e"),
                    120.0,
                )?,
            ],
            max_acceleration_extruding: first_non_negative(
                "machine_max_acceleration_extruding",
                self.values().get("machine_max_acceleration_extruding"),
                1500.0,
            )?,
            max_acceleration_retracting: first_non_negative(
                "machine_max_acceleration_retracting",
                self.values().get("machine_max_acceleration_retracting"),
                1500.0,
            )?,
            max_acceleration_travel: first_non_negative(
                "machine_max_acceleration_travel",
                self.values().get("machine_max_acceleration_travel"),
                0.0,
            )?,
            max_jerk: [
                first_non_negative(
                    "machine_max_jerk_x",
                    self.values().get("machine_max_jerk_x"),
                    10.0,
                )?,
                first_non_negative(
                    "machine_max_jerk_y",
                    self.values().get("machine_max_jerk_y"),
                    10.0,
                )?,
                first_non_negative(
                    "machine_max_jerk_z",
                    self.values().get("machine_max_jerk_z"),
                    0.2,
                )?,
                first_non_negative(
                    "machine_max_jerk_e",
                    self.values().get("machine_max_jerk_e"),
                    2.5,
                )?,
            ],
            max_junction_deviation: first_non_negative(
                "machine_max_junction_deviation",
                self.values().get("machine_max_junction_deviation"),
                0.01,
            )?,
        })
    }
}

fn bool_option(key: &str, value: Option<&Value>, default: bool) -> Result<bool, SliceError> {
    let Some(value) = value else {
        return Ok(default);
    };
    value
        .as_bool()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a boolean")))
}

fn first_non_negative(key: &str, value: Option<&Value>, default: f64) -> Result<f64, SliceError> {
    let Some(value) = value else {
        return Ok(default);
    };
    let value = parsing::parse_numeric_vector(key, value)?
        .into_iter()
        .next()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must not be empty")))?;
    if value.is_finite() && value >= 0.0 {
        Ok(value)
    } else {
        Err(SliceError::InvalidInput(format!(
            "{key} contains invalid value"
        )))
    }
}
