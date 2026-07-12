use super::{SliceOptions, parsing, temperature_vector};

use crate::SliceError;

const FLUSH_SPEED_KEY: &str = "filament_flush_volumetric_speed";
const FALLBACK_SPEED_KEY: &str = "filament_max_volumetric_speed";
const FLUSH_TEMP_KEY: &str = "filament_flush_temp";
const FALLBACK_TEMP_KEY: &str = "nozzle_temperature_range_high";
const DEFAULT_FALLBACK_SPEED: &[f64] = &[2.0];
const DEFAULT_FALLBACK_TEMP: &[u32] = &[240];

#[derive(Debug, PartialEq)]
pub(crate) struct FlushPlaceholders {
    flush_volumetric_speeds: Vec<f64>,
    flush_temperatures: Vec<u32>,
}

impl FlushPlaceholders {
    pub(crate) fn new(flush_volumetric_speeds: Vec<f64>, flush_temperatures: Vec<u32>) -> Self {
        Self {
            flush_volumetric_speeds,
            flush_temperatures,
        }
    }

    pub(crate) fn flush_volumetric_speeds(&self) -> &[f64] {
        &self.flush_volumetric_speeds
    }

    pub(crate) fn flush_temperatures(&self) -> &[u32] {
        &self.flush_temperatures
    }
}

impl SliceOptions {
    pub(crate) fn flush_placeholders(&self) -> Result<FlushPlaceholders, SliceError> {
        let speed_fallback = parse_speed_vector(self, FALLBACK_SPEED_KEY, DEFAULT_FALLBACK_SPEED)?;
        let temp_fallback = parse_temp_vector(self, FALLBACK_TEMP_KEY, DEFAULT_FALLBACK_TEMP)?;

        Ok(FlushPlaceholders::new(
            replace_zero_f64(
                parse_speed_vector(self, FLUSH_SPEED_KEY, &[0.0])?,
                &speed_fallback,
            ),
            replace_zero_u32(
                parse_temp_vector(self, FLUSH_TEMP_KEY, &[0])?,
                &temp_fallback,
            ),
        ))
    }
}

fn parse_speed_vector(
    options: &SliceOptions,
    key: &str,
    default: &[f64],
) -> Result<Vec<f64>, SliceError> {
    let Some(value) = options.values().get(key) else {
        return Ok(default.to_vec());
    };
    let values = parsing::parse_numeric_vector(key, value)?;
    let valid = match key {
        FLUSH_SPEED_KEY => values
            .iter()
            .all(|value| value.is_finite() && *value >= 0.0 && *value <= 200.0),
        FALLBACK_SPEED_KEY => values.iter().all(|value| value.is_finite() && *value >= 0.0),
        _ => unreachable!(),
    };
    if valid {
        Ok(values)
    } else {
        Err(SliceError::InvalidInput(format!(
            "{key} contains invalid value"
        )))
    }
}

fn parse_temp_vector(
    options: &SliceOptions,
    key: &str,
    default: &[u32],
) -> Result<Vec<u32>, SliceError> {
    let Some(value) = options.values().get(key) else {
        return Ok(default.to_vec());
    };
    let values = temperature_vector::parse_integer_vector(key, value)?;
    if values.iter().all(|value| *value <= 1500) {
        Ok(values)
    } else {
        Err(SliceError::InvalidInput(format!(
            "{key} contains invalid value"
        )))
    }
}

fn replace_zero_f64(values: Vec<f64>, fallback: &[f64]) -> Vec<f64> {
    values
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            if value == 0.0 {
                fallback_at(fallback, index)
            } else {
                value
            }
        })
        .collect()
}

fn replace_zero_u32(values: Vec<u32>, fallback: &[u32]) -> Vec<u32> {
    values
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            if value == 0 {
                fallback_at(fallback, index)
            } else {
                value
            }
        })
        .collect()
}

fn fallback_at<T: Copy>(values: &[T], index: usize) -> T {
    values.get(index).copied().unwrap_or(values[0])
}
