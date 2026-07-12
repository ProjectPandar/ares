use std::collections::BTreeMap;

use serde_json::Value;

use crate::SliceError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SupportIroningPattern {
    Rectilinear,
    Concentric,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SupportIroningConfig {
    flow_ratio: f64,
    spacing_mm: f64,
    pattern: SupportIroningPattern,
}

impl SupportIroningConfig {
    pub(crate) const fn flow_ratio(self) -> f64 {
        self.flow_ratio
    }

    pub(crate) const fn spacing_mm(self) -> f64 {
        self.spacing_mm
    }

    pub(crate) const fn pattern(self) -> SupportIroningPattern {
        self.pattern
    }
}

pub(super) fn parse(values: &BTreeMap<String, Value>) -> Result<f64, SliceError> {
    let fallback = parse_percent_ratio(values.get("ironing_flow"), "ironing_flow", 10.0)?;
    let Some(value) = values.get("filament_ironing_flow") else {
        return Ok(fallback);
    };
    Ok(parse_nullable_percent_ratio(value, "filament_ironing_flow")?.unwrap_or(fallback))
}

pub(crate) fn parse_support_ironing(
    values: &BTreeMap<String, Value>,
) -> Result<SupportIroningConfig, SliceError> {
    Ok(SupportIroningConfig {
        flow_ratio: parse_percent_ratio(
            values.get("support_ironing_flow"),
            "support_ironing_flow",
            10.0,
        )?,
        spacing_mm: crate::options::parsing::parse_range_f64(
            "support_ironing_spacing",
            values.get("support_ironing_spacing"),
            0.1,
            0.0,
            1.0,
        )?,
        pattern: parse_support_ironing_pattern(values)?,
    })
}

fn parse_support_ironing_pattern(
    values: &BTreeMap<String, Value>,
) -> Result<SupportIroningPattern, SliceError> {
    let Some(value) = values.get("support_ironing_pattern") else {
        return Ok(SupportIroningPattern::Rectilinear);
    };
    let Value::String(text) = value else {
        return Err(SliceError::InvalidInput(
            "support_ironing_pattern must be a string".to_owned(),
        ));
    };
    match text.as_str() {
        "rectilinear" => Ok(SupportIroningPattern::Rectilinear),
        "concentric" => Ok(SupportIroningPattern::Concentric),
        _ => Err(SliceError::InvalidInput(
            "support_ironing_pattern has invalid value".to_owned(),
        )),
    }
}

fn parse_percent_ratio(
    value: Option<&Value>,
    key: &str,
    default_percent: f64,
) -> Result<f64, SliceError> {
    let Some(value) = value else {
        return Ok(default_percent * 0.01);
    };
    percent_value(value, key, false).map(|percent| percent * 0.01)
}

fn parse_nullable_percent_ratio(value: &Value, key: &str) -> Result<Option<f64>, SliceError> {
    let value = match value {
        Value::Array(values) => values
            .first()
            .ok_or_else(|| SliceError::InvalidInput(format!("{key} must not be empty")))?,
        value => value,
    };
    if matches!(value, Value::String(text) if text.trim() == "nil") {
        return Ok(None);
    }
    percent_value(value, key, true).map(|percent| Some(percent * 0.01))
}

fn percent_value(value: &Value, key: &str, nullable: bool) -> Result<f64, SliceError> {
    let percent = match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.trim().parse::<f64>().ok(),
        _ => None,
    }
    .ok_or_else(|| {
        let expected = if nullable {
            "a number or nil"
        } else {
            "a number"
        };
        SliceError::InvalidInput(format!("{key} must be {expected}"))
    })?;
    if percent.is_finite() && (0.0..=100.0).contains(&percent) {
        Ok(percent)
    } else {
        Err(SliceError::InvalidInput(format!("{key} is out of range")))
    }
}
