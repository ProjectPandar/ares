use super::{SliceOptions, temperature_vector};

use crate::SliceError;
use serde_json::Value;

const SUPPORT_KEY: &str = "support_air_filtration";
const ACTIVATE_KEY: &str = "activate_air_filtration";
const DURING_ACTIVE_KEY: &str = "activate_air_filtration_during_print";
const COMPLETION_ACTIVE_KEY: &str = "activate_air_filtration_on_completion";
const DURING_SPEED_KEY: &str = "during_print_exhaust_fan_speed";
const COMPLETION_SPEED_KEY: &str = "complete_print_exhaust_fan_speed";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ExhaustFanControl {
    during_print_speed: Option<u8>,
    completion_speed: Option<u8>,
}

impl ExhaustFanControl {
    pub(crate) const fn new(during_print_speed: Option<u8>, completion_speed: Option<u8>) -> Self {
        Self {
            during_print_speed,
            completion_speed,
        }
    }

    pub(crate) const fn disabled() -> Self {
        Self::new(None, None)
    }

    pub(crate) const fn during_print_speed(self) -> Option<u8> {
        self.during_print_speed
    }

    pub(crate) const fn completion_speed(self) -> Option<u8> {
        self.completion_speed
    }
}

impl SliceOptions {
    pub(crate) fn exhaust_fan_control(&self) -> Result<ExhaustFanControl, SliceError> {
        if !support_air_filtration(self.values().get(SUPPORT_KEY))? {
            return Ok(ExhaustFanControl::disabled());
        }
        let active = bool_vector(ACTIVATE_KEY, self.values().get(ACTIVATE_KEY), &[false])?;
        let during_active = bool_vector(
            DURING_ACTIVE_KEY,
            self.values().get(DURING_ACTIVE_KEY),
            &[true],
        )?;
        let completion_active = bool_vector(
            COMPLETION_ACTIVE_KEY,
            self.values().get(COMPLETION_ACTIVE_KEY),
            &[true],
        )?;
        let during_speed =
            percent_vector(DURING_SPEED_KEY, self.values().get(DURING_SPEED_KEY), &[60])?;
        let completion_speed = percent_vector(
            COMPLETION_SPEED_KEY,
            self.values().get(COMPLETION_SPEED_KEY),
            &[80],
        )?;
        Ok(ExhaustFanControl::new(
            selected_speed(&active, &during_active, &during_speed),
            selected_speed(&active, &completion_active, &completion_speed),
        ))
    }

    pub(crate) fn during_print_exhaust_fan_speed_num_values(&self) -> Result<Vec<u32>, SliceError> {
        Ok(percent_vector(DURING_SPEED_KEY, self.values().get(DURING_SPEED_KEY), &[60])?
            .into_iter()
            .map(exhaust_fan_speed_num)
            .collect())
    }
}

fn support_air_filtration(value: Option<&Value>) -> Result<bool, SliceError> {
    let Some(value) = value else {
        return Ok(true);
    };
    match value {
        Value::Bool(value) => Ok(*value),
        Value::String(text) => match text.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(invalid(SUPPORT_KEY, "must be a lowercase boolean")),
        },
        _ => Err(invalid(SUPPORT_KEY, "must be a boolean")),
    }
}

fn bool_vector(key: &str, value: Option<&Value>, default: &[bool]) -> Result<Vec<bool>, SliceError> {
    let Some(value) = value else {
        return Ok(default.to_vec());
    };
    match value {
        Value::Bool(value) => Ok(vec![*value]),
        Value::String(text) => parse_bool_text(key, text),
        Value::Array(values) => {
            if values.is_empty() {
                return Err(invalid(key, "must not be empty"));
            }
            values
                .iter()
                .map(|value| match value {
                    Value::Bool(value) => Ok(*value),
                    _ => Err(invalid(key, "must contain boolean values")),
                })
                .collect()
        }
        _ => Err(invalid(key, "must be a boolean or boolean list")),
    }
}

fn parse_bool_text(key: &str, text: &str) -> Result<Vec<bool>, SliceError> {
    let parts = text.split([';', ',']).map(str::trim).collect::<Vec<_>>();
    if parts.iter().any(|part| part.is_empty()) {
        return Err(invalid(key, "must not be empty"));
    }
    parts
        .into_iter()
        .map(|part| match part {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(invalid(key, "must contain lowercase boolean values")),
        })
        .collect()
}

fn percent_vector(key: &str, value: Option<&Value>, default: &[u8]) -> Result<Vec<u8>, SliceError> {
    let Some(value) = value else {
        return Ok(default.to_vec());
    };
    temperature_vector::parse_integer_vector(key, value)?
        .into_iter()
        .map(|value| {
            u8::try_from(value)
                .ok()
                .filter(|value| *value <= 100)
                .ok_or_else(|| invalid(key, "must be a percent from 0 to 100"))
        })
        .collect()
}

fn exhaust_fan_speed_num(speed: u8) -> u32 {
    (f64::from(speed) / 100.0 * 255.0) as u32
}

fn selected_speed(active: &[bool], phase_active: &[bool], speed: &[u8]) -> Option<u8> {
    let len = active.len().max(phase_active.len()).max(speed.len());
    (0..len)
        .filter(|index| value_at(active, *index) && value_at(phase_active, *index))
        .map(|index| value_at(speed, index))
        .max()
}

fn value_at<T: Copy>(values: &[T], index: usize) -> T {
    values[index.min(values.len() - 1)]
}

fn invalid(key: &str, reason: &str) -> SliceError {
    SliceError::InvalidInput(format!("{key} {reason}"))
}
