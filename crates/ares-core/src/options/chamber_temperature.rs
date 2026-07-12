use super::{SliceOptions, temperature_vector};

use crate::SliceError;
use serde_json::Value;

const ACTIVATE_KEY: &str = "activate_chamber_temp_control";
const SUPPORT_KEY: &str = "support_chamber_temp_control";
const TEMPERATURE_KEY: &str = "chamber_temperature";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ChamberTemperatureControl {
    temperature: Option<u32>,
}

impl ChamberTemperatureControl {
    pub(crate) const fn disabled() -> Self {
        Self { temperature: None }
    }

    pub(crate) const fn enabled(temperature: u32) -> Self {
        Self {
            temperature: Some(temperature),
        }
    }

    pub(crate) const fn temperature(self) -> Option<u32> {
        self.temperature
    }
}

impl SliceOptions {
    pub(crate) fn chamber_temperature_values(&self) -> Result<Vec<u32>, SliceError> {
        match self.values().get(TEMPERATURE_KEY) {
            Some(value) => temperature_vector::parse_integer_vector(TEMPERATURE_KEY, value),
            None => Ok(vec![0]),
        }
    }

    pub(crate) fn overall_chamber_temperature(&self) -> Result<u32, SliceError> {
        Ok(self.chamber_temperature_values()?.into_iter().max().unwrap())
    }

    pub(crate) fn chamber_temperature_control(
        &self,
    ) -> Result<ChamberTemperatureControl, SliceError> {
        if !self.bool_option(SUPPORT_KEY, true)? {
            return Ok(ChamberTemperatureControl::disabled());
        }

        let activate = match self.values().get(ACTIVATE_KEY) {
            Some(value) => parse_bool_vector(ACTIVATE_KEY, value)?,
            None => vec![false],
        };
        let temperature = self.overall_chamber_temperature()?;
        if activate.into_iter().any(|value| value) && temperature > 0 {
            Ok(ChamberTemperatureControl::enabled(temperature))
        } else {
            Ok(ChamberTemperatureControl::disabled())
        }
    }
}

fn parse_bool_vector(key: &str, value: &Value) -> Result<Vec<bool>, SliceError> {
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

fn invalid(key: &str, reason: &str) -> SliceError {
    SliceError::InvalidInput(format!("{key} {reason}"))
}
