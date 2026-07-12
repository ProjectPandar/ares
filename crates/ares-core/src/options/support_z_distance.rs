use crate::{SliceError, SliceOptions};
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SupportZDistanceOptions {
    top_z_distance_mm: f64,
    bottom_z_distance_mm: f64,
    enforce_support_layers: u32,
}

impl SupportZDistanceOptions {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn top_z_distance_mm(&self) -> f64 {
        self.top_z_distance_mm
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn bottom_z_distance_mm(&self) -> f64 {
        self.bottom_z_distance_mm
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn enforce_support_layers(&self) -> u32 {
        self.enforce_support_layers
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn zero_top_contact(&self) -> bool {
        self.top_z_distance_mm == 0.0
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn zero_gap_interface_top(&self, top_layers: usize) -> bool {
        top_layers > 0 && self.zero_top_contact()
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn zero_gap_interface_bottom(&self, top_layers: usize, bottom_layers: isize) -> bool {
        let resolved_bottom_layers = if bottom_layers < 0 {
            top_layers
        } else {
            bottom_layers as usize
        };
        resolved_bottom_layers > 0 && (self.zero_top_contact() || self.bottom_z_distance_mm == 0.0)
    }
}

pub(crate) fn parse(options: &SliceOptions) -> Result<SupportZDistanceOptions, SliceError> {
    Ok(SupportZDistanceOptions {
        top_z_distance_mm: options.range_f64("support_top_z_distance", 0.2, 0.0, f64::INFINITY)?,
        bottom_z_distance_mm: options.range_f64(
            "support_bottom_z_distance",
            0.2,
            0.0,
            f64::INFINITY,
        )?,
        enforce_support_layers: parse_enforce_support_layers(
            options.values().get("enforce_support_layers"),
        )?,
    })
}

fn parse_enforce_support_layers(value: Option<&Value>) -> Result<u32, SliceError> {
    let Some(value) = value else {
        return Ok(0);
    };
    let parsed = match value {
        Value::Number(number) => number.as_u64().and_then(|value| u32::try_from(value).ok()),
        Value::String(text) => text.parse::<u32>().ok(),
        _ => None,
    }
    .ok_or_else(|| SliceError::InvalidInput("enforce_support_layers must be an integer".to_owned()))?;
    if parsed <= 5000 {
        Ok(parsed)
    } else {
        Err(SliceError::InvalidInput(
            "enforce_support_layers is out of range".to_owned(),
        ))
    }
}
