use crate::{SliceError, SliceOptions};
use serde_json::Value;

const SUPPORT_THRESHOLD_ANGLE: &str = "support_threshold_angle";
const SUPPORT_THRESHOLD_OVERLAP: &str = "support_threshold_overlap";

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SupportThresholdOptions {
    independent_layer_height: bool,
    angle_degrees: u32,
    overlap: SupportThresholdOverlap,
}

impl SupportThresholdOptions {
    pub(crate) const fn independent_layer_height(self) -> bool {
        self.independent_layer_height
    }

    pub(crate) const fn angle_degrees(self) -> u32 {
        self.angle_degrees
    }

    pub(crate) const fn overlap(self) -> SupportThresholdOverlap {
        self.overlap
    }

    pub(crate) fn consume_runtime(self) {
        let _ = (
            self.independent_layer_height(),
            self.angle_degrees(),
            self.overlap(),
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum SupportThresholdOverlap {
    AbsoluteMm(f64),
    Percent(f64),
}

impl SupportThresholdOverlap {
    pub(crate) fn abs_value(self, base_mm: f64) -> f64 {
        match self {
            Self::AbsoluteMm(value) => value,
            Self::Percent(percent) => base_mm * percent / 100.0,
        }
    }
}

impl SliceOptions {
    pub(crate) fn support_threshold_options(
        &self,
    ) -> Result<SupportThresholdOptions, SliceError> {
        Ok(SupportThresholdOptions {
            independent_layer_height: self.bool_option("independent_support_layer_height", true)?,
            angle_degrees: parse_angle_degrees(self.values().get(SUPPORT_THRESHOLD_ANGLE))?,
            overlap: parse_overlap(self.values().get(SUPPORT_THRESHOLD_OVERLAP))?,
        })
    }
}

fn parse_angle_degrees(value: Option<&Value>) -> Result<u32, SliceError> {
    let Some(value) = value else {
        return Ok(30);
    };
    let parsed = match value {
        Value::Number(number) => number
            .as_u64()
            .and_then(|value| u32::try_from(value).ok()),
        Value::String(text) => text.parse::<u32>().ok(),
        _ => None,
    }
    .ok_or_else(|| {
        SliceError::InvalidInput(format!("{SUPPORT_THRESHOLD_ANGLE} must be an integer"))
    })?;
    if parsed <= 90 {
        Ok(parsed)
    } else {
        Err(SliceError::InvalidInput(format!(
            "{SUPPORT_THRESHOLD_ANGLE} is out of range"
        )))
    }
}

fn parse_overlap(value: Option<&Value>) -> Result<SupportThresholdOverlap, SliceError> {
    let Some(value) = value else {
        return Ok(SupportThresholdOverlap::Percent(50.0));
    };
    let overlap = match value {
        Value::Number(number) => number
            .as_f64()
            .map(SupportThresholdOverlap::AbsoluteMm),
        Value::String(text) => parse_overlap_text(text),
        _ => None,
    }
    .ok_or_else(|| {
        SliceError::InvalidInput(format!("{SUPPORT_THRESHOLD_OVERLAP} must be a number"))
    })?;
    let stored_value = match overlap {
        SupportThresholdOverlap::AbsoluteMm(value) | SupportThresholdOverlap::Percent(value) => {
            value
        }
    };
    if stored_value.is_finite() && (0.0..=100.0).contains(&stored_value) {
        Ok(overlap)
    } else {
        Err(SliceError::InvalidInput(format!(
            "{SUPPORT_THRESHOLD_OVERLAP} is out of range"
        )))
    }
}

fn parse_overlap_text(text: &str) -> Option<SupportThresholdOverlap> {
    let text = text.trim();
    if let Some(percent) = text.strip_suffix('%') {
        percent
            .trim()
            .parse()
            .ok()
            .map(SupportThresholdOverlap::Percent)
    } else {
        text.parse().ok().map(SupportThresholdOverlap::AbsoluteMm)
    }
}
