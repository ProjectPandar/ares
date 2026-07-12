mod parsing;
mod role;

use parsing::{
    first_integer, first_percent, first_percent_integer, first_range_f64,
    first_signed_role_fan_speed_value, scalar_percent,
};
pub(crate) use role::{
    InternalBridgeFanSpeed, LayerRoleFanControl, OverhangFanThreshold, RoleFanControl, RoleFanSpeed,
};
use super::SliceOptions;

use crate::SliceError;

const DEFAULT_FAN_MIN_SPEED: u8 = 20;
const DEFAULT_FAN_MAX_SPEED: u8 = 100;
const DEFAULT_FULL_FAN_SPEED_LAYER: u32 = 0;
const DEFAULT_CLOSE_FAN_FIRST_LAYERS: u32 = 1;
const DEFAULT_PART_COOLING_FAN_MIN_PWM: u8 = 0;
const DEFAULT_FAN_COOLING_LAYER_TIME: f64 = 60.0;
const DEFAULT_FAN_KICKSTART: f64 = 0.0;
const DEFAULT_REDUCE_FAN_STOP_START_FREQ: bool = false;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PartCoolingFanRamp {
    min_speed: u8,
    max_speed: u8,
    full_speed_layer: u32,
    close_fan_first_layers: u32,
    slow_down_layer_time_s: f64,
    fan_cooling_layer_time_s: f64,
    fan_kickstart_s: f64,
    reduce_fan_stop_start_freq: bool,
}

impl PartCoolingFanRamp {
    pub(crate) const fn fan_kickstart_s(self) -> f64 {
        self.fan_kickstart_s
    }

    pub(crate) const fn close_fan_first_layers(self) -> u32 {
        self.close_fan_first_layers
    }

    #[cfg(test)]
    pub(crate) const fn fan_cooling_layer_time_s(self) -> f64 {
        self.fan_cooling_layer_time_s
    }

    pub(crate) fn speed_for_layer_time(
        self,
        layer_index: usize,
        layer_time_s: Option<f64>,
    ) -> Option<u8> {
        if self.max_speed == 0 {
            return None;
        }
        let layer_id = u32::try_from(layer_index).unwrap_or(u32::MAX);
        if layer_id < self.close_fan_first_layers {
            return None;
        }
        let base_speed = match layer_time_s {
            Some(layer_time_s) if layer_time_s.is_finite() => {
                self.base_speed_for_runtime_layer_time(layer_time_s)
            }
            _ => self.base_speed_for_layer(layer_index),
        };
        if layer_id.saturating_add(1) >= self.full_speed_layer
            || self.full_speed_layer <= self.close_fan_first_layers
        {
            return Some(base_speed);
        }
        let denominator = self.full_speed_layer - self.close_fan_first_layers;
        let numerator = layer_id + 1 - self.close_fan_first_layers;
        let factor = f64::from(numerator) / f64::from(denominator);
        let speed = f64::from(base_speed) * factor;
        Some((speed + 0.5).floor().clamp(0.0, 100.0) as u8)
    }

    pub(crate) fn speed_for_layer(self, layer_index: usize) -> Option<u8> {
        self.speed_for_layer_time(layer_index, None)
    }

    pub(crate) fn role_speed_for_layer(self, layer_index: usize, raw_speed: u8) -> Option<u8> {
        let layer_id = u32::try_from(layer_index).unwrap_or(u32::MAX);
        if layer_id < self.close_fan_first_layers {
            return None;
        }
        if layer_id.saturating_add(1) >= self.full_speed_layer
            || self.full_speed_layer <= self.close_fan_first_layers
        {
            return Some(raw_speed);
        }
        let denominator = self.full_speed_layer - self.close_fan_first_layers;
        let numerator = layer_id + 1 - self.close_fan_first_layers;
        let factor = f64::from(numerator) / f64::from(denominator);
        Some(
            (f64::from(raw_speed) * factor + 0.5)
                .floor()
                .clamp(0.0, 100.0) as u8,
        )
    }

    fn base_speed_for_layer(self, layer_index: usize) -> u8 {
        if self.full_speed_layer <= 1 {
            return self.max_speed;
        }
        let layer_number = u32::try_from(layer_index)
            .ok()
            .and_then(|index| index.checked_add(1))
            .unwrap_or(u32::MAX);
        if layer_number >= self.full_speed_layer {
            return self.max_speed;
        }
        let span = f64::from(self.full_speed_layer - 1);
        let factor = f64::from(layer_number - 1) / span;
        let speed =
            f64::from(self.min_speed) + f64::from(self.max_speed - self.min_speed) * factor;
        (speed + 0.5).floor() as u8
    }

    fn base_speed_for_runtime_layer_time(self, layer_time_s: f64) -> u8 {
        if layer_time_s >= self.fan_cooling_layer_time_s {
            return self.long_layer_baseline_speed();
        }
        if layer_time_s <= self.slow_down_layer_time_s
            || self.fan_cooling_layer_time_s <= self.slow_down_layer_time_s
        {
            return self.max_speed;
        }
        let factor = (self.fan_cooling_layer_time_s - layer_time_s)
            / (self.fan_cooling_layer_time_s - self.slow_down_layer_time_s);
        let speed =
            f64::from(self.min_speed) + f64::from(self.max_speed - self.min_speed) * factor;
        (speed + 0.5).floor().clamp(0.0, 100.0) as u8
    }

    const fn long_layer_baseline_speed(self) -> u8 {
        if self.reduce_fan_stop_start_freq {
            self.min_speed
        } else {
            0
        }
    }
}

impl SliceOptions {
    pub(crate) fn part_cooling_fan_ramp(&self) -> Result<PartCoolingFanRamp, SliceError> {
        let max_speed = first_percent(
            "fan_max_speed",
            self.values().get("fan_max_speed"),
            DEFAULT_FAN_MAX_SPEED,
        )?;
        let min_speed = first_percent(
            "fan_min_speed",
            self.values().get("fan_min_speed"),
            DEFAULT_FAN_MIN_SPEED,
        )?
        .min(max_speed);
        let full_speed_layer = first_integer(
            "full_fan_speed_layer",
            self.values().get("full_fan_speed_layer"),
            DEFAULT_FULL_FAN_SPEED_LAYER,
        )?;
        let close_fan_first_layers = first_integer(
            "close_fan_the_first_x_layers",
            self.values().get("close_fan_the_first_x_layers"),
            DEFAULT_CLOSE_FAN_FIRST_LAYERS,
        )?;
        let slow_down_layer_time_s =
            crate::options::slow_down_layers::parse_slow_down_layer_time(self.values())?;
        let fan_cooling_layer_time_s = first_range_f64(
            "fan_cooling_layer_time",
            self.values().get("fan_cooling_layer_time"),
            DEFAULT_FAN_COOLING_LAYER_TIME,
            0.0,
            1000.0,
        )?;
        let fan_kickstart_s = scalar_non_negative_f64(
            "fan_kickstart",
            self.values().get("fan_kickstart"),
            DEFAULT_FAN_KICKSTART,
        )?;
        let reduce_fan_stop_start_freq = first_bool(
            "reduce_fan_stop_start_freq",
            self.values().get("reduce_fan_stop_start_freq"),
            DEFAULT_REDUCE_FAN_STOP_START_FREQ,
        )?;
        Ok(PartCoolingFanRamp {
            min_speed,
            max_speed,
            full_speed_layer,
            close_fan_first_layers,
            slow_down_layer_time_s,
            fan_cooling_layer_time_s,
            fan_kickstart_s,
            reduce_fan_stop_start_freq,
        })
    }

    pub(crate) fn part_cooling_fan_min_pwm(&self) -> Result<u8, SliceError> {
        scalar_percent(
            "part_cooling_fan_min_pwm",
            self.values().get("part_cooling_fan_min_pwm"),
            DEFAULT_PART_COOLING_FAN_MIN_PWM,
        )
    }

    pub(crate) fn internal_bridge_fan_speed(&self) -> Result<InternalBridgeFanSpeed, SliceError> {
        let value = first_signed_role_fan_speed_value(
            "internal_bridge_fan_speed",
            self.values().get("internal_bridge_fan_speed"),
            -1,
        )?;
        match value {
            -1 => Ok(InternalBridgeFanSpeed::fallback()),
            0..=100 => Ok(InternalBridgeFanSpeed::new(value as u8)),
            _ => Err(invalid(
                "internal_bridge_fan_speed",
                "must be -1 or an integer percent from 0 to 100",
            )),
        }
    }

    pub(crate) fn role_fan_speed(&self, key: &str) -> Result<RoleFanSpeed, SliceError> {
        let value = first_signed_role_fan_speed_value(key, self.values().get(key), -1)?;
        match value {
            -1 => Ok(RoleFanSpeed::disabled()),
            0..=100 => Ok(RoleFanSpeed::new(value as u8)),
            _ => Err(invalid(
                key,
                "must be -1 or an integer percent from 0 to 100",
            )),
        }
    }

    pub(crate) fn role_fan_control(&self) -> Result<RoleFanControl, SliceError> {
        let enabled = self.bool_option("enable_overhang_bridge_fan", true)?;
        let overhang_speed = first_percent_integer(
            "overhang_fan_speed",
            self.values().get("overhang_fan_speed"),
            100,
        )?;
        let threshold = parse_overhang_fan_threshold(self.values().get("overhang_fan_threshold"))?;
        let support_interface_speed =
            self.role_fan_speed("support_material_interface_fan_speed")?;
        let ironing_speed = self.role_fan_speed("ironing_fan_speed")?;
        Ok(RoleFanControl::new(
            enabled,
            overhang_speed,
            self.internal_bridge_fan_speed()?,
            threshold,
        )
        .with_support_interface_speed(support_interface_speed)
        .with_ironing_speed(ironing_speed))
    }
}

fn parse_overhang_fan_threshold(
    value: Option<&serde_json::Value>,
) -> Result<OverhangFanThreshold, SliceError> {
    match value {
        None => Ok(OverhangFanThreshold::OverlapGated),
        Some(serde_json::Value::String(value)) => match value.as_str() {
            "0%" => Ok(OverhangFanThreshold::AllExternalPerimeters),
            "10%" | "25%" | "50%" | "75%" | "95%" => Ok(OverhangFanThreshold::OverlapGated),
            _ => Err(invalid(
                "overhang_fan_threshold",
                "must be one of 0%, 10%, 25%, 50%, 75%, or 95%",
            )),
        },
        _ => Err(invalid(
            "overhang_fan_threshold",
            "must be one of 0%, 10%, 25%, 50%, 75%, or 95%",
        )),
    }
}

fn scalar_non_negative_f64(
    key: &str,
    value: Option<&serde_json::Value>,
    default: f64,
) -> Result<f64, SliceError> {
    match value {
        None => Ok(default),
        Some(serde_json::Value::Number(number)) => match number.as_f64() {
            Some(value) if value.is_finite() && value >= 0.0 => Ok(value),
            _ => Err(invalid(key, "must be a non-negative finite number")),
        },
        _ => Err(invalid(key, "must be a non-negative finite number")),
    }
}

fn first_bool(
    key: &str,
    value: Option<&serde_json::Value>,
    default: bool,
) -> Result<bool, SliceError> {
    match value {
        None => Ok(default),
        Some(serde_json::Value::Bool(value)) => Ok(*value),
        Some(serde_json::Value::Array(values)) => match values.first() {
            Some(serde_json::Value::Bool(value)) => Ok(*value),
            Some(_) => Err(invalid(key, "must contain boolean values")),
            None => Err(invalid(key, "must not be empty")),
        },
        _ => Err(invalid(key, "must be a boolean or boolean list")),
    }
}

fn invalid(key: &str, reason: &str) -> SliceError {
    SliceError::InvalidInput(format!("{key} {reason}"))
}
