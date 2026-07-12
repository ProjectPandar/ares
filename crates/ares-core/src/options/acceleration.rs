use std::collections::BTreeMap;

use serde_json::Value;

use crate::{AccelerationOptions, JerkOptions, SliceError, SliceOptions};

use super::parsing;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AccelToDecelConfig {
    enabled: bool,
    factor_percent: f64,
}

impl AccelToDecelConfig {
    pub(crate) const fn new(enabled: bool, factor_percent: f64) -> Self {
        Self {
            enabled,
            factor_percent,
        }
    }

    pub(crate) const fn enabled(self) -> bool {
        self.enabled
    }

    pub(crate) const fn factor_percent(self) -> f64 {
        self.factor_percent
    }
}

impl SliceOptions {
    pub(crate) fn accel_to_decel_config(&self) -> Result<AccelToDecelConfig, SliceError> {
        let enabled = self.bool_option("accel_to_decel_enable", true)?;
        let factor = accel_to_decel_factor_percent(self.values().get("accel_to_decel_factor"))?;
        Ok(AccelToDecelConfig::new(enabled, factor))
    }

    pub(crate) fn default_junction_deviation(&self) -> Result<f64, SliceError> {
        parse_default_junction_deviation(self.values())
    }
}

pub(crate) fn parse_acceleration_options(
    options: &BTreeMap<String, Value>,
) -> Result<AccelerationOptions, SliceError> {
    let default = non_negative_number_or_string(
        options.get("default_acceleration"),
        "default_acceleration",
        500.0,
    )?;
    let initial_layer = non_negative_number_or_string(
        options.get("initial_layer_acceleration"),
        "initial_layer_acceleration",
        300.0,
    )?;
    let outer_wall = non_negative_number_or_string(
        options.get("outer_wall_acceleration"),
        "outer_wall_acceleration",
        500.0,
    )?;
    let bridge = match options.get("bridge_acceleration") {
        Some(value) => parsing::parse_non_negative_numeric_or_percent_over_base(
            "bridge_acceleration",
            value,
            outer_wall,
        )?,
        None => outer_wall * 0.5,
    };
    let inner_wall = non_negative_number_or_string(
        options.get("inner_wall_acceleration"),
        "inner_wall_acceleration",
        10000.0,
    )?;
    let travel = non_negative_number_or_string(
        options.get("travel_acceleration"),
        "travel_acceleration",
        10000.0,
    )?;
    let initial_layer_travel = match options.get("initial_layer_travel_acceleration") {
        Some(value) => parsing::parse_non_negative_numeric_or_percent_over_base(
            "initial_layer_travel_acceleration",
            value,
            travel,
        )?,
        None => travel,
    };
    let sparse_infill = match options.get("sparse_infill_acceleration") {
        Some(value) => parsing::parse_non_negative_numeric_or_percent_over_base(
            "sparse_infill_acceleration",
            value,
            default,
        )?,
        None => default,
    };
    let internal_solid_infill = match options.get("internal_solid_infill_acceleration") {
        Some(value) => parsing::parse_non_negative_numeric_or_percent_over_base(
            "internal_solid_infill_acceleration",
            value,
            default,
        )?,
        None => default,
    };
    let top_surface = match options.get("top_surface_acceleration") {
        Some(value) => parsing::parse_non_negative_numeric_or_percent_over_base(
            "top_surface_acceleration",
            value,
            default,
        )?,
        None => 500.0,
    };

    Ok(AccelerationOptions {
        default_mm_s2: default,
        initial_layer_mm_s2: initial_layer,
        outer_wall_mm_s2: outer_wall,
        bridge_mm_s2: bridge,
        inner_wall_mm_s2: inner_wall,
        travel_mm_s2: travel,
        initial_layer_travel_mm_s2: initial_layer_travel,
        sparse_infill_mm_s2: sparse_infill,
        internal_solid_infill_mm_s2: internal_solid_infill,
        top_surface_mm_s2: top_surface,
    })
}

pub(crate) fn parse_jerk_options(
    options: &BTreeMap<String, Value>,
) -> Result<JerkOptions, SliceError> {
    let default = non_negative_number_or_string(options.get("default_jerk"), "default_jerk", 0.0)?;
    let outer_wall =
        non_negative_number_or_string(options.get("outer_wall_jerk"), "outer_wall_jerk", 9.0)?;
    let inner_wall =
        non_negative_number_or_string(options.get("inner_wall_jerk"), "inner_wall_jerk", 9.0)?;
    let infill = non_negative_number_or_string(options.get("infill_jerk"), "infill_jerk", 9.0)?;
    let top_surface =
        non_negative_number_or_string(options.get("top_surface_jerk"), "top_surface_jerk", 9.0)?;
    let initial_layer = non_negative_number_or_string(
        options.get("initial_layer_jerk"),
        "initial_layer_jerk",
        9.0,
    )?;
    let travel = non_negative_number_or_string(options.get("travel_jerk"), "travel_jerk", 12.0)?;
    let initial_layer_travel = match options.get("initial_layer_travel_jerk") {
        Some(value) => parsing::parse_non_negative_numeric_or_percent_over_base(
            "initial_layer_travel_jerk",
            value,
            travel,
        )?,
        None => travel,
    };

    Ok(JerkOptions {
        default_mm_s: default,
        initial_layer_mm_s: initial_layer,
        outer_wall_mm_s: outer_wall,
        inner_wall_mm_s: inner_wall,
        infill_mm_s: infill,
        top_surface_mm_s: top_surface,
        travel_mm_s: travel,
        initial_layer_travel_mm_s: initial_layer_travel,
    })
}

pub(crate) fn parse_default_junction_deviation(
    options: &BTreeMap<String, Value>,
) -> Result<f64, SliceError> {
    let value = non_negative_number_or_string(
        options.get("default_junction_deviation"),
        "default_junction_deviation",
        0.0,
    )?;
    if value <= 0.3 {
        Ok(value)
    } else {
        Err(SliceError::InvalidInput(
            "default_junction_deviation must be between 0 and 0.3".to_owned(),
        ))
    }
}

fn non_negative_number_or_string(
    value: Option<&Value>,
    key: &str,
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
    .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a number")))?;
    if value.is_finite() && value >= 0.0 {
        Ok(value)
    } else {
        Err(SliceError::InvalidInput(format!(
            "{key} must be non-negative"
        )))
    }
}

fn accel_to_decel_factor_percent(value: Option<&Value>) -> Result<f64, SliceError> {
    let Some(value) = value else {
        return Ok(50.0);
    };
    let value = value
        .as_f64()
        .ok_or_else(|| SliceError::InvalidInput("accel_to_decel_factor must be a number".into()))?;
    if value.is_finite() && (1.0..=100.0).contains(&value) {
        Ok(value)
    } else {
        Err(SliceError::InvalidInput(
            "accel_to_decel_factor is out of range".into(),
        ))
    }
}
