use super::{SliceOptions, temperature_vector};

use crate::SliceError;

const DEFAULT_FIRST_LAYER_NOZZLE_TEMPERATURE: u32 = 200;
const DEFAULT_OTHER_LAYER_NOZZLE_TEMPERATURE: u32 = 200;
const DEFAULT_IDLE_TEMPERATURE: u32 = 0;
const DEFAULT_NOZZLE_TEMPERATURE_RANGE_LOW: u32 = 190;
const DEFAULT_NOZZLE_TEMPERATURE_RANGE_HIGH: u32 = 240;
const DEFAULT_STANDBY_TEMPERATURE_DELTA: i32 = -5;
const KEY: &str = "nozzle_temperature_initial_layer";
const OTHER_LAYER_KEY: &str = "nozzle_temperature";
const OOZE_PREVENTION_KEY: &str = "ooze_prevention";
const IDLE_TEMPERATURE_KEY: &str = "idle_temperature";
const RANGE_LOW_KEY: &str = "nozzle_temperature_range_low";
const RANGE_HIGH_KEY: &str = "nozzle_temperature_range_high";
const STANDBY_TEMPERATURE_DELTA_KEY: &str = "standby_temperature_delta";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FirstLayerNozzleTemperature(u32);

impl FirstLayerNozzleTemperature {
    pub(crate) const fn new(value: u32) -> Self {
        Self(value)
    }

    pub(crate) const fn value(self) -> u32 {
        self.0
    }

    pub(crate) const fn emits_command(self) -> bool {
        self.0 > 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OtherLayerNozzleTemperature(u32);

impl OtherLayerNozzleTemperature {
    pub(crate) const fn new(value: u32) -> Self {
        Self(value)
    }

    pub(crate) const fn value(self) -> u32 {
        self.0
    }
}

impl SliceOptions {
    pub(crate) fn first_layer_nozzle_temperature_values(&self) -> Result<Vec<u32>, SliceError> {
        match self.values().get(KEY) {
            Some(value) => temperature_vector::parse_integer_vector(KEY, value),
            None => Ok(vec![DEFAULT_FIRST_LAYER_NOZZLE_TEMPERATURE]),
        }
    }

    pub(crate) fn first_layer_nozzle_temperature(
        &self,
    ) -> Result<FirstLayerNozzleTemperature, SliceError> {
        let values = self.first_layer_nozzle_temperature_values()?;
        Ok(FirstLayerNozzleTemperature::new(values[0]))
    }

    pub(crate) fn first_layer_nozzle_temperature_for_tool(
        &self,
        tool_index: usize,
    ) -> Result<FirstLayerNozzleTemperature, SliceError> {
        let values = self.first_layer_nozzle_temperature_values()?;
        Ok(FirstLayerNozzleTemperature::new(orca_vector_value_at(
            &values, tool_index,
        )))
    }

    pub(crate) fn ooze_prevention(&self) -> Result<bool, SliceError> {
        match self.values().get(OOZE_PREVENTION_KEY) {
            Some(value) => parse_bool(OOZE_PREVENTION_KEY, value),
            None => Ok(false),
        }
    }

    pub(crate) fn idle_temperature_values(&self) -> Result<Vec<u32>, SliceError> {
        match self.values().get(IDLE_TEMPERATURE_KEY) {
            Some(value) => temperature_vector::parse_integer_vector(IDLE_TEMPERATURE_KEY, value),
            None => Ok(vec![DEFAULT_IDLE_TEMPERATURE]),
        }
    }

    pub(crate) fn idle_temperature_for_tool(&self, tool_index: usize) -> Result<u32, SliceError> {
        let values = self.idle_temperature_values()?;
        Ok(orca_vector_value_at(&values, tool_index))
    }

    pub(crate) fn nozzle_temperature_range_low_values(&self) -> Result<Vec<u32>, SliceError> {
        match self.values().get(RANGE_LOW_KEY) {
            Some(value) => temperature_vector::parse_integer_vector(RANGE_LOW_KEY, value),
            None => Ok(vec![DEFAULT_NOZZLE_TEMPERATURE_RANGE_LOW]),
        }
    }

    pub(crate) fn nozzle_temperature_range_high_values(&self) -> Result<Vec<u32>, SliceError> {
        match self.values().get(RANGE_HIGH_KEY) {
            Some(value) => temperature_vector::parse_integer_vector(RANGE_HIGH_KEY, value),
            None => Ok(vec![DEFAULT_NOZZLE_TEMPERATURE_RANGE_HIGH]),
        }
    }

    pub(crate) fn standby_temperature_delta(&self) -> Result<i32, SliceError> {
        match self.values().get(STANDBY_TEMPERATURE_DELTA_KEY) {
            Some(value) => parse_i32(STANDBY_TEMPERATURE_DELTA_KEY, value),
            None => Ok(DEFAULT_STANDBY_TEMPERATURE_DELTA),
        }
    }

    pub(crate) fn validate_startup_nozzle_temperature_options(&self) -> Result<(), SliceError> {
        self.first_layer_nozzle_temperature_values()?;
        self.ooze_prevention()?;
        self.idle_temperature_values()?;
        self.standby_temperature_delta()?;
        Ok(())
    }

    pub(crate) fn machine_start_temperature_placeholder(&self) -> Result<u32, SliceError> {
        let values = match self.values().get(OTHER_LAYER_KEY) {
            Some(value) => temperature_vector::parse_integer_vector(OTHER_LAYER_KEY, value)?,
            None => vec![DEFAULT_OTHER_LAYER_NOZZLE_TEMPERATURE],
        };
        Ok(values[0])
    }

    pub(crate) fn other_layer_nozzle_temperature(
        &self,
    ) -> Result<OtherLayerNozzleTemperature, SliceError> {
        let values = match self.values().get(OTHER_LAYER_KEY) {
            Some(value) => temperature_vector::parse_integer_vector(OTHER_LAYER_KEY, value)?,
            None => vec![DEFAULT_OTHER_LAYER_NOZZLE_TEMPERATURE],
        };
        Ok(OtherLayerNozzleTemperature::new(values[0]))
    }

    pub(crate) fn validate_nozzle_temperature_ranges(&self) -> Result<(), SliceError> {
        let temperatures = match self.values().get(OTHER_LAYER_KEY) {
            Some(value) => temperature_vector::parse_integer_vector(OTHER_LAYER_KEY, value)?,
            None => vec![DEFAULT_OTHER_LAYER_NOZZLE_TEMPERATURE],
        };
        let lows = self.nozzle_temperature_range_low_values()?;
        let highs = self.nozzle_temperature_range_high_values()?;
        let count = self.nozzle_temperature_range_validation_count(&temperatures, &lows, &highs);

        for index in 0..count {
            let low = orca_vector_value_at(&lows, index);
            let high = orca_vector_value_at(&highs, index);
            if low >= high {
                return Err(SliceError::InvalidInput(format!(
                    "{RANGE_LOW_KEY} must be lower than {RANGE_HIGH_KEY}"
                )));
            }
        }

        for first in 0..count {
            for second in first + 1..count {
                let first_temperature = orca_vector_value_at(&temperatures, first);
                let second_temperature = orca_vector_value_at(&temperatures, second);
                let first_low = orca_vector_value_at(&lows, first);
                let first_high = orca_vector_value_at(&highs, first);
                let second_low = orca_vector_value_at(&lows, second);
                let second_high = orca_vector_value_at(&highs, second);
                if second_temperature < first_low
                    || second_temperature > first_high
                    || first_temperature < second_low
                    || first_temperature > second_high
                {
                    return Err(SliceError::InvalidInput(format!(
                        "{OTHER_LAYER_KEY} must be mutually compatible with {RANGE_LOW_KEY} and {RANGE_HIGH_KEY}"
                    )));
                }
            }
        }

        Ok(())
    }

    pub(crate) fn nozzle_temperature_range_validation_count(
        &self,
        temperatures: &[u32],
        lows: &[u32],
        highs: &[u32],
    ) -> usize {
        let mut count = temperatures.len().max(lows.len()).max(highs.len()).max(1);
        count = count.max(option_vector_like_len(self.values().get("filament_type")));
        count = count.max(option_vector_like_len(self.values().get("filament_diameter")));
        count = count.max(option_vector_like_len(self.values().get("nozzle_diameter")));
        count
    }
}

fn orca_vector_value_at(values: &[u32], index: usize) -> u32 {
    values.get(index).copied().unwrap_or(values[0])
}

fn option_vector_like_len(value: Option<&serde_json::Value>) -> usize {
    match value {
        Some(serde_json::Value::Array(values)) => values.len(),
        Some(serde_json::Value::String(value)) if value.contains(';') || value.contains(',') => {
            value
                .split([';', ','])
                .filter(|part| !part.trim().is_empty())
                .count()
        }
        Some(_) => 1,
        None => 0,
    }
}

fn parse_bool(key: &str, value: &serde_json::Value) -> Result<bool, SliceError> {
    match value {
        serde_json::Value::Bool(value) => Ok(*value),
        serde_json::Value::String(text) => match text.trim() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(SliceError::InvalidInput(format!(
                "{key} must contain lowercase boolean values"
            ))),
        },
        _ => Err(SliceError::InvalidInput(format!("{key} must be a boolean"))),
    }
}

fn parse_i32(key: &str, value: &serde_json::Value) -> Result<i32, SliceError> {
    match value {
        serde_json::Value::Number(number) => number
            .as_i64()
            .and_then(|value| i32::try_from(value).ok())
            .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be an integer"))),
        serde_json::Value::String(text) => text
            .trim()
            .parse::<i32>()
            .map_err(|_| SliceError::InvalidInput(format!("{key} must be an integer"))),
        _ => Err(SliceError::InvalidInput(format!("{key} must be an integer"))),
    }
}
