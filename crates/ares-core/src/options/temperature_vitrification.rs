use super::{SliceOptions, temperature_vector};

use crate::SliceError;

const KEY: &str = "temperature_vitrification";
const DEFAULT_TEMPERATURE_VITRIFICATION: u32 = 100;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TemperatureVitrification(u32);

impl TemperatureVitrification {
    pub(crate) const fn new(value: u32) -> Self {
        Self(value)
    }

    pub(crate) const fn value(self) -> u32 {
        self.0
    }
}

impl SliceOptions {
    pub(crate) fn temperature_vitrification(
        &self,
    ) -> Result<TemperatureVitrification, SliceError> {
        let values = match self.values().get(KEY) {
            Some(value) => temperature_vector::parse_integer_vector(KEY, value)?,
            None => vec![DEFAULT_TEMPERATURE_VITRIFICATION],
        };
        Ok(TemperatureVitrification::new(
            values.into_iter().min().unwrap(),
        ))
    }
}
