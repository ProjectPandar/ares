use super::{SliceOptions, temperature_vector};

use crate::SliceError;

const DEFAULT_BED_TYPE: &str = "Cool Plate";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FirstLayerBedTemperature(u32);

impl FirstLayerBedTemperature {
    pub(crate) const fn new(value: u32) -> Self {
        Self(value)
    }

    pub(crate) const fn value(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OtherLayerBedTemperature(u32);

impl OtherLayerBedTemperature {
    pub(crate) const fn new(value: u32) -> Self {
        Self(value)
    }

    pub(crate) const fn value(self) -> u32 {
        self.0
    }
}

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize,
)]
pub enum BedTemperatureFormula {
    #[serde(rename = "by_first_filament")]
    FirstFilament,
    #[default]
    #[serde(rename = "by_highest_temp")]
    HighestTemp,
}

impl BedTemperatureFormula {
    fn parse(options: &SliceOptions) -> Result<Self, SliceError> {
        match options.values().get("bed_temperature_formula") {
            Some(value) => match value.as_str() {
                Some("by_first_filament") => Ok(Self::FirstFilament),
                Some("by_highest_temp") => Ok(Self::HighestTemp),
                Some(_) => Err(invalid_bed_temperature_formula("has an invalid value")),
                None => Err(invalid_bed_temperature_formula("must be a string")),
            },
            None => Ok(Self::HighestTemp),
        }
    }

    fn select(self, values: &[u32]) -> u32 {
        match self {
            Self::FirstFilament => values[0],
            Self::HighestTemp => values.iter().copied().max().unwrap(),
        }
    }
}

impl SliceOptions {
    pub(crate) fn first_layer_bed_temperature_values(&self) -> Result<Vec<u32>, SliceError> {
        let key = first_layer_bed_temperature_key(self.curr_bed_type()?)?;
        match self.values().get(key) {
            Some(value) => temperature_vector::parse_integer_vector(key, value),
            None => Ok(vec![default_first_layer_bed_temperature(key)]),
        }
    }

    pub(crate) fn first_layer_bed_temperature(
        &self,
    ) -> Result<FirstLayerBedTemperature, SliceError> {
        let formula = BedTemperatureFormula::parse(self)?;
        let values = self.first_layer_bed_temperature_values()?;
        Ok(FirstLayerBedTemperature::new(formula.select(&values)))
    }

    pub(crate) fn other_layer_bed_temperature_values(&self) -> Result<Vec<u32>, SliceError> {
        let bed_type = self.curr_bed_type()?;
        let key = other_layer_bed_temperature_key(bed_type)?;
        match self.values().get(key) {
            Some(value) => temperature_vector::parse_integer_vector(key, value),
            None => Ok(vec![self.first_layer_bed_temperature()?.value()]),
        }
    }

    pub(crate) fn other_layer_bed_temperature(
        &self,
    ) -> Result<OtherLayerBedTemperature, SliceError> {
        let formula = BedTemperatureFormula::parse(self)?;
        let values = self.other_layer_bed_temperature_values()?;
        Ok(OtherLayerBedTemperature::new(formula.select(&values)))
    }

    fn curr_bed_type(&self) -> Result<&str, SliceError> {
        match self.values().get("curr_bed_type") {
            Some(value) => value
                .as_str()
                .ok_or_else(|| invalid_curr_bed_type("must be a string")),
            None => Ok(DEFAULT_BED_TYPE),
        }
    }
}

fn other_layer_bed_temperature_key(bed_type: &str) -> Result<&'static str, SliceError> {
    match bed_type {
        "Cool Plate" => Ok("cool_plate_temp"),
        "Textured Cool Plate" => Ok("textured_cool_plate_temp"),
        "Engineering Plate" => Ok("eng_plate_temp"),
        "High Temp Plate" => Ok("hot_plate_temp"),
        "Textured PEI Plate" => Ok("textured_plate_temp"),
        "Supertack Plate" | "SuperTack Plate" => Ok("supertack_plate_temp"),
        _ => Err(invalid_curr_bed_type("has an invalid value")),
    }
}

fn first_layer_bed_temperature_key(bed_type: &str) -> Result<&'static str, SliceError> {
    match bed_type {
        "Cool Plate" => Ok("cool_plate_temp_initial_layer"),
        "Textured Cool Plate" => Ok("textured_cool_plate_temp_initial_layer"),
        "Engineering Plate" => Ok("eng_plate_temp_initial_layer"),
        "High Temp Plate" => Ok("hot_plate_temp_initial_layer"),
        "Textured PEI Plate" => Ok("textured_plate_temp_initial_layer"),
        "Supertack Plate" | "SuperTack Plate" => Ok("supertack_plate_temp_initial_layer"),
        _ => Err(invalid_curr_bed_type("has an invalid value")),
    }
}

fn default_first_layer_bed_temperature(key: &str) -> u32 {
    match key {
        "cool_plate_temp_initial_layer" | "supertack_plate_temp_initial_layer" => 35,
        "textured_cool_plate_temp_initial_layer" => 40,
        "eng_plate_temp_initial_layer"
        | "hot_plate_temp_initial_layer"
        | "textured_plate_temp_initial_layer" => 45,
        _ => unreachable!("validated first-layer bed temperature key"),
    }
}

fn invalid_curr_bed_type(reason: &str) -> SliceError {
    SliceError::InvalidInput(format!("curr_bed_type {reason}"))
}

fn invalid_bed_temperature_formula(reason: &str) -> SliceError {
    SliceError::InvalidInput(format!("bed_temperature_formula {reason}"))
}
