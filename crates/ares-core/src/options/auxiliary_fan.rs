use super::{SliceOptions, parsing, temperature_vector};

use crate::SliceError;
use serde_json::Value;

const AUXILIARY_FAN_KEY: &str = "auxiliary_fan";
const ADDITIONAL_COOLING_FAN_SPEED_KEY: &str = "additional_cooling_fan_speed";
const CLOSE_ADDITIONAL_FAN_FIRST_LAYERS_KEY: &str = "close_additional_fan_first_x_layers";
const ADDITIONAL_FAN_FULL_SPEED_LAYER_KEY: &str = "additional_fan_full_speed_layer";
const FIRST_X_LAYER_FAN_SPEED_KEY: &str = "first_x_layer_fan_speed";

const DEFAULT_AUXILIARY_FAN: bool = false;
const DEFAULT_ADDITIONAL_COOLING_FAN_SPEED: u8 = 0;
const DEFAULT_CLOSE_FAN_FIRST_LAYERS: u32 = 1;
const DEFAULT_ADDITIONAL_FAN_FULL_SPEED_LAYER: u32 = 0;
const DEFAULT_FIRST_X_LAYER_FAN_SPEED: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AuxiliaryFanControl {
    enabled: bool,
    speed: u8,
    first_x_layer_speed_percent: f64,
    close_additional_fan_first_layers: u32,
    additional_fan_full_speed_layer: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AuxiliaryFanPlaceholders {
    max_additional_fan: f64,
    first_x_layer_fan_speed: f64,
    close_additional_fan_first_x_layers: u32,
    additional_fan_full_speed_layer: u32,
}

impl AuxiliaryFanPlaceholders {
    pub(crate) const fn new(
        max_additional_fan: f64,
        first_x_layer_fan_speed: f64,
        close_additional_fan_first_x_layers: u32,
        additional_fan_full_speed_layer: u32,
    ) -> Self {
        Self {
            max_additional_fan,
            first_x_layer_fan_speed,
            close_additional_fan_first_x_layers,
            additional_fan_full_speed_layer,
        }
    }

    pub(crate) const fn max_additional_fan(self) -> f64 {
        self.max_additional_fan
    }

    pub(crate) const fn first_x_layer_fan_speed(self) -> f64 {
        self.first_x_layer_fan_speed
    }

    pub(crate) const fn close_additional_fan_first_x_layers(self) -> u32 {
        self.close_additional_fan_first_x_layers
    }

    pub(crate) const fn additional_fan_full_speed_layer(self) -> u32 {
        self.additional_fan_full_speed_layer
    }
}

impl AuxiliaryFanControl {
    pub(crate) const fn new(
        enabled: bool,
        speed: u8,
        first_x_layer_speed_percent: f64,
        close_additional_fan_first_layers: u32,
        additional_fan_full_speed_layer: u32,
    ) -> Self {
        Self {
            enabled,
            speed,
            first_x_layer_speed_percent,
            close_additional_fan_first_layers,
            additional_fan_full_speed_layer,
        }
    }

    pub(crate) fn speed_for_layer(self, layer_index: usize) -> Option<u8> {
        if !self.enabled || (self.speed == 0 && self.first_x_layer_speed_percent == 0.0) {
            return None;
        }
        let layer_number = u32::try_from(layer_index)
            .unwrap_or(u32::MAX)
            .saturating_add(1);
        let desired_percent = if layer_number <= self.close_additional_fan_first_layers {
            self.first_x_layer_speed_percent
        } else if layer_number < self.additional_fan_full_speed_layer
            && self.additional_fan_full_speed_layer > self.close_additional_fan_first_layers
        {
            let denominator =
                self.additional_fan_full_speed_layer - self.close_additional_fan_first_layers;
            let numerator = layer_number - self.close_additional_fan_first_layers;
            let factor = f64::from(numerator) / f64::from(denominator);
            self.first_x_layer_speed_percent
                + (f64::from(self.speed) - self.first_x_layer_speed_percent) * factor
        } else {
            f64::from(self.speed)
        };

        let speed = rounded_percent_u8(desired_percent);
        if speed > 0 {
            Some(speed)
        } else if self.close_additional_fan_first_layers > 0
            && rounded_percent_u8(self.first_x_layer_speed_percent) > 0
            && layer_number > self.close_additional_fan_first_layers
        {
            Some(0)
        } else {
            None
        }
    }

    pub(crate) fn completion_shutdown_speed(self) -> Option<u8> {
        if self.enabled
            && (self.speed > 0
                || (self.close_additional_fan_first_layers > 0
                    && rounded_percent_u8(self.first_x_layer_speed_percent) > 0))
        {
            Some(0)
        } else {
            None
        }
    }
}

impl SliceOptions {
    pub(crate) fn auxiliary_fan_control(&self) -> Result<AuxiliaryFanControl, SliceError> {
        let enabled = auxiliary_fan(self.values().get(AUXILIARY_FAN_KEY))?;
        let speed =
            additional_cooling_fan_speed(self.values().get(ADDITIONAL_COOLING_FAN_SPEED_KEY))?;
        let first_x_layer_speed_percent = first_percent_float(
            FIRST_X_LAYER_FAN_SPEED_KEY,
            self.values().get(FIRST_X_LAYER_FAN_SPEED_KEY),
            DEFAULT_FIRST_X_LAYER_FAN_SPEED,
        )?;
        let close_additional_fan_first_layers = first_integer(
            CLOSE_ADDITIONAL_FAN_FIRST_LAYERS_KEY,
            self.values().get(CLOSE_ADDITIONAL_FAN_FIRST_LAYERS_KEY),
            DEFAULT_CLOSE_FAN_FIRST_LAYERS,
        )?;
        let additional_fan_full_speed_layer = first_integer(
            ADDITIONAL_FAN_FULL_SPEED_LAYER_KEY,
            self.values().get(ADDITIONAL_FAN_FULL_SPEED_LAYER_KEY),
            DEFAULT_ADDITIONAL_FAN_FULL_SPEED_LAYER,
        )?;
        Ok(AuxiliaryFanControl::new(
            enabled,
            speed,
            first_x_layer_speed_percent,
            close_additional_fan_first_layers,
            additional_fan_full_speed_layer,
        ))
    }

    pub(crate) fn auxiliary_fan_placeholders(&self) -> Result<AuxiliaryFanPlaceholders, SliceError> {
        let max_additional_fan = f64::from(additional_cooling_fan_speed(
            self.values().get(ADDITIONAL_COOLING_FAN_SPEED_KEY),
        )?);
        let first_x_layer_fan_speed = first_percent_float(
            FIRST_X_LAYER_FAN_SPEED_KEY,
            self.values().get(FIRST_X_LAYER_FAN_SPEED_KEY),
            DEFAULT_FIRST_X_LAYER_FAN_SPEED,
        )?;
        let close_additional_fan_first_x_layers = first_integer(
            CLOSE_ADDITIONAL_FAN_FIRST_LAYERS_KEY,
            self.values().get(CLOSE_ADDITIONAL_FAN_FIRST_LAYERS_KEY),
            DEFAULT_CLOSE_FAN_FIRST_LAYERS,
        )?;
        let additional_fan_full_speed_layer = first_integer(
            ADDITIONAL_FAN_FULL_SPEED_LAYER_KEY,
            self.values().get(ADDITIONAL_FAN_FULL_SPEED_LAYER_KEY),
            DEFAULT_ADDITIONAL_FAN_FULL_SPEED_LAYER,
        )?;
        Ok(AuxiliaryFanPlaceholders::new(
            max_additional_fan,
            first_x_layer_fan_speed,
            close_additional_fan_first_x_layers,
            additional_fan_full_speed_layer,
        ))
    }
}

fn rounded_percent_u8(percent: f64) -> u8 {
    (percent + 0.5).floor().clamp(0.0, 100.0) as u8
}

fn auxiliary_fan(value: Option<&Value>) -> Result<bool, SliceError> {
    let Some(value) = value else {
        return Ok(DEFAULT_AUXILIARY_FAN);
    };
    match value {
        Value::Bool(value) => Ok(*value),
        _ => Err(invalid(AUXILIARY_FAN_KEY, "must be a boolean")),
    }
}

fn additional_cooling_fan_speed(value: Option<&Value>) -> Result<u8, SliceError> {
    let value = first_integer(
        ADDITIONAL_COOLING_FAN_SPEED_KEY,
        value,
        u32::from(DEFAULT_ADDITIONAL_COOLING_FAN_SPEED),
    )?;
    u8::try_from(value)
        .ok()
        .filter(|value| *value <= 100)
        .ok_or_else(|| invalid(ADDITIONAL_COOLING_FAN_SPEED_KEY, "must be a percent from 0 to 100"))
}

fn first_integer(key: &str, value: Option<&Value>, default: u32) -> Result<u32, SliceError> {
    let Some(value) = value else {
        return Ok(default);
    };
    temperature_vector::parse_integer_vector(key, value)?
        .into_iter()
        .next()
        .ok_or_else(|| invalid(key, "must not be empty"))
}

fn first_percent_float(key: &str, value: Option<&Value>, default: f64) -> Result<f64, SliceError> {
    let Some(value) = value else {
        return Ok(default);
    };
    let value = parsing::parse_numeric_vector(key, value)?
        .into_iter()
        .next()
        .ok_or_else(|| invalid(key, "must not be empty"))?;
    if value.is_finite() && (0.0..=100.0).contains(&value) {
        Ok(value)
    } else {
        Err(invalid(key, "must be a percent from 0 to 100"))
    }
}

fn invalid(key: &str, reason: &str) -> SliceError {
    SliceError::InvalidInput(format!("{key} {reason}"))
}
