use crate::{SliceError, SliceOptions};
use serde_json::Value;

const RAFT_EXPANSION: &str = "raft_expansion";
const RAFT_FIRST_LAYER_EXPANSION: &str = "raft_first_layer_expansion";
const RAFT_FIRST_LAYER_DENSITY: &str = "raft_first_layer_density";
const DEFAULT_RAFT_EXPANSION_MM: f64 = 1.5;
const DEFAULT_RAFT_FIRST_LAYER_EXPANSION_MM: f64 = 2.0;
const DEFAULT_RAFT_FIRST_LAYER_DENSITY_PERCENT: f64 = 90.0;
const MIN_RAFT_FIRST_LAYER_DENSITY_PERCENT: f64 = 10.0;
const MAX_RAFT_FIRST_LAYER_DENSITY_PERCENT: f64 = 100.0;
const RAFT_LAYERS_MAX: u32 = 100;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RaftOptions {
    layers: u32,
}

impl RaftOptions {
    pub(crate) const fn layers(self) -> u32 {
        self.layers
    }

    pub(crate) const fn has_raft(self) -> bool {
        self.layers > 0
    }
}

impl SliceOptions {
    pub(crate) fn raft_options(&self) -> Result<RaftOptions, SliceError> {
        Ok(RaftOptions {
            layers: parse_raft_layers(self.values().get("raft_layers"))?,
        })
    }

    pub(crate) fn raft_expansion_mm(&self) -> Result<f64, SliceError> {
        parse_raft_expansion(self.values().get(RAFT_EXPANSION))
    }

    pub(crate) fn raft_first_layer_expansion_mm(&self) -> Result<f64, SliceError> {
        parse_raft_first_layer_expansion(self.values().get(RAFT_FIRST_LAYER_EXPANSION))
    }

    pub(crate) fn raft_first_layer_density_percent(&self) -> Result<f64, SliceError> {
        parse_raft_first_layer_density(self.values().get(RAFT_FIRST_LAYER_DENSITY))
    }
}

fn parse_raft_layers(value: Option<&Value>) -> Result<u32, SliceError> {
    let Some(value) = value else {
        return Ok(0);
    };
    let parsed = match value {
        Value::Number(number) => number.as_u64().and_then(|value| u32::try_from(value).ok()),
        Value::String(text) => text.parse::<u32>().ok(),
        _ => None,
    }
    .ok_or_else(|| SliceError::InvalidInput("raft_layers must be an integer".to_owned()))?;
    if parsed <= RAFT_LAYERS_MAX {
        Ok(parsed)
    } else {
        Err(SliceError::InvalidInput(
            "raft_layers is out of range".to_owned(),
        ))
    }
}

fn parse_raft_expansion(value: Option<&Value>) -> Result<f64, SliceError> {
    let Some(value) = value else {
        return Ok(DEFAULT_RAFT_EXPANSION_MM);
    };
    match value {
        Value::Number(number) => number.to_string().parse().ok(),
        Value::String(text) => text.trim().parse().ok(),
        _ => None,
    }
    .filter(|value: &f64| value.is_finite() && *value >= 0.0)
    .ok_or_else(invalid_raft_expansion)
}

fn parse_raft_first_layer_expansion(value: Option<&Value>) -> Result<f64, SliceError> {
    let Some(value) = value else {
        return Ok(DEFAULT_RAFT_FIRST_LAYER_EXPANSION_MM);
    };
    match value {
        Value::Number(number) => number.to_string().parse().ok(),
        Value::String(text) => text.trim().parse().ok(),
        _ => None,
    }
    .filter(|value: &f64| value.is_finite() && *value >= 0.0)
    .ok_or_else(invalid_raft_first_layer_expansion)
}

fn parse_raft_first_layer_density(value: Option<&Value>) -> Result<f64, SliceError> {
    let Some(value) = value else {
        return Ok(DEFAULT_RAFT_FIRST_LAYER_DENSITY_PERCENT);
    };
    match value {
        Value::Number(number) => number.to_string().parse().ok(),
        Value::String(text) => text.trim().parse().ok(),
        _ => None,
    }
    .filter(|value: &f64| {
        value.is_finite()
            && (MIN_RAFT_FIRST_LAYER_DENSITY_PERCENT..=MAX_RAFT_FIRST_LAYER_DENSITY_PERCENT)
                .contains(value)
    })
    .ok_or_else(invalid_raft_first_layer_density)
}

fn invalid_raft_expansion() -> SliceError {
    SliceError::InvalidInput(format!(
        "{RAFT_EXPANSION} must be a finite non-negative number"
    ))
}

fn invalid_raft_first_layer_expansion() -> SliceError {
    SliceError::InvalidInput(format!(
        "{RAFT_FIRST_LAYER_EXPANSION} must be a finite non-negative number"
    ))
}

fn invalid_raft_first_layer_density() -> SliceError {
    SliceError::InvalidInput(format!(
        "{RAFT_FIRST_LAYER_DENSITY} must be a finite number from {MIN_RAFT_FIRST_LAYER_DENSITY_PERCENT} to {MAX_RAFT_FIRST_LAYER_DENSITY_PERCENT}"
    ))
}
