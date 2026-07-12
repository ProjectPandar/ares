use serde_json::Value;

use super::{SliceOptions, parsing};
use crate::SliceError;

const ENABLE_KEY: &str = "input_shaping_emit";
const TYPE_KEY: &str = "input_shaping_type";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InputShaperType {
    Default,
    Mzv,
    Zv,
    Zvd,
    Zvdd,
    Zvddd,
    Ei,
    Ei2,
    TwoHumpEi,
    Ei3,
    ThreeHumpEi,
    Daa,
    Disable,
}

impl InputShaperType {
    pub(crate) const fn as_gcode_value(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Mzv => "MZV",
            Self::Zv => "ZV",
            Self::Zvd => "ZVD",
            Self::Zvdd => "ZVDD",
            Self::Zvddd => "ZVDDD",
            Self::Ei => "EI",
            Self::Ei2 => "EI2",
            Self::TwoHumpEi => "2HUMP_EI",
            Self::Ei3 => "EI3",
            Self::ThreeHumpEi => "3HUMP_EI",
            Self::Daa => "DAA",
            Self::Disable => "Disable",
        }
    }

    pub(crate) const fn disables_input_shaping(self) -> bool {
        matches!(self, Self::Disable)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct InputShapingConfig {
    pub(crate) emit: bool,
    pub(crate) shaper_type: InputShaperType,
    pub(crate) freq_x: f64,
    pub(crate) freq_y: f64,
    pub(crate) damp_x: f64,
    pub(crate) damp_y: f64,
}

impl SliceOptions {
    pub(crate) fn input_shaping_config(&self) -> Result<InputShapingConfig, SliceError> {
        Ok(InputShapingConfig {
            emit: bool_option(ENABLE_KEY, self.values().get(ENABLE_KEY), false)?,
            shaper_type: input_shaper_type(self.values().get(TYPE_KEY))?,
            freq_x: bounded_float(
                "input_shaping_freq_x",
                self.values().get("input_shaping_freq_x"),
                0.0,
                1000.0,
                0.0,
            )?,
            freq_y: bounded_float(
                "input_shaping_freq_y",
                self.values().get("input_shaping_freq_y"),
                0.0,
                1000.0,
                0.0,
            )?,
            damp_x: bounded_float(
                "input_shaping_damp_x",
                self.values().get("input_shaping_damp_x"),
                0.0,
                1.0,
                0.1,
            )?,
            damp_y: bounded_float(
                "input_shaping_damp_y",
                self.values().get("input_shaping_damp_y"),
                0.0,
                1.0,
                0.1,
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

fn input_shaper_type(value: Option<&Value>) -> Result<InputShaperType, SliceError> {
    let Some(value) = value else {
        return Ok(InputShaperType::Default);
    };
    let Some(value) = value.as_str() else {
        return Err(SliceError::InvalidInput(format!(
            "{TYPE_KEY} must be a string"
        )));
    };
    match value {
        "Default" => Ok(InputShaperType::Default),
        "MZV" => Ok(InputShaperType::Mzv),
        "ZV" => Ok(InputShaperType::Zv),
        "ZVD" => Ok(InputShaperType::Zvd),
        "ZVDD" => Ok(InputShaperType::Zvdd),
        "ZVDDD" => Ok(InputShaperType::Zvddd),
        "EI" => Ok(InputShaperType::Ei),
        "EI2" => Ok(InputShaperType::Ei2),
        "2HUMP_EI" => Ok(InputShaperType::TwoHumpEi),
        "EI3" => Ok(InputShaperType::Ei3),
        "3HUMP_EI" => Ok(InputShaperType::ThreeHumpEi),
        "DAA" => Ok(InputShaperType::Daa),
        "Disable" => Ok(InputShaperType::Disable),
        _ => Err(SliceError::InvalidInput(format!(
            "{TYPE_KEY} has invalid value"
        ))),
    }
}

fn bounded_float(
    key: &str,
    value: Option<&Value>,
    min: f64,
    max: f64,
    default: f64,
) -> Result<f64, SliceError> {
    let Some(value) = value else {
        return Ok(default);
    };
    let value = parsing::parse_numeric_vector(key, value)?
        .into_iter()
        .next()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must not be empty")))?;
    if value.is_finite() && (min..=max).contains(&value) {
        Ok(value)
    } else {
        Err(SliceError::InvalidInput(format!(
            "{key} contains invalid value"
        )))
    }
}
