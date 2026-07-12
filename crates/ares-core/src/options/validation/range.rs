use std::collections::BTreeMap;

use serde_json::Value;

use crate::SliceError;

use super::super::SliceOptions;
use super::helpers::registry_default;

const RANGE_EPSILON: f64 = 0.0001;
const FINITE_LINE_WIDTH_MAX: f64 = 1000.0;
const DEFAULT_LINE_WIDTH_MAX: f64 = f32::MAX as f64;
const LINE_WIDTH_RANGES: &[LineWidthRange] = &[
    LineWidthRange {
        key: "line_width",
        min: 0.0,
        max: FINITE_LINE_WIDTH_MAX,
    },
    LineWidthRange {
        key: "outer_wall_line_width",
        min: 0.0,
        max: FINITE_LINE_WIDTH_MAX,
    },
    LineWidthRange {
        key: "inner_wall_line_width",
        min: 0.0,
        max: FINITE_LINE_WIDTH_MAX,
    },
    LineWidthRange {
        key: "sparse_infill_line_width",
        min: 0.0,
        max: FINITE_LINE_WIDTH_MAX,
    },
    LineWidthRange {
        key: "internal_solid_infill_line_width",
        min: 0.0,
        max: FINITE_LINE_WIDTH_MAX,
    },
    LineWidthRange {
        key: "top_surface_line_width",
        min: 0.0,
        max: FINITE_LINE_WIDTH_MAX,
    },
    LineWidthRange {
        key: "support_line_width",
        min: 0.0,
        max: FINITE_LINE_WIDTH_MAX,
    },
    LineWidthRange {
        key: "initial_layer_line_width",
        min: 0.0,
        max: FINITE_LINE_WIDTH_MAX,
    },
    LineWidthRange {
        key: "skin_infill_line_width",
        min: 0.0,
        max: DEFAULT_LINE_WIDTH_MAX,
    },
    LineWidthRange {
        key: "skeleton_infill_line_width",
        min: 0.0,
        max: DEFAULT_LINE_WIDTH_MAX,
    },
];

struct LineWidthRange {
    key: &'static str,
    min: f64,
    max: f64,
}

struct RawFloatOrPercent {
    value: f64,
    serialized: String,
}

impl SliceOptions {
    pub fn validate_line_width_range_options(
        &self,
    ) -> Result<BTreeMap<String, String>, SliceError> {
        let mut errors = BTreeMap::new();

        for range in LINE_WIDTH_RANGES {
            let width = self.raw_float_or_percent_or_default(range.key)?;
            if is_source_range_valid(width.value, range.min, range.max) {
                continue;
            }
            errors.insert(
                range.key.to_owned(),
                format!(
                    "{} not in range [{:.6},{:.6}]",
                    width.serialized, range.min, range.max
                ),
            );
        }

        Ok(errors)
    }

    fn raw_float_or_percent_or_default(&self, key: &str) -> Result<RawFloatOrPercent, SliceError> {
        match self.values().get(key) {
            Some(value) => parse_raw_float_or_percent(key, value),
            None => {
                parse_raw_float_or_percent(key, &Value::String(registry_default(key)?.to_owned()))
            }
        }
    }
}

fn parse_raw_float_or_percent(key: &str, value: &Value) -> Result<RawFloatOrPercent, SliceError> {
    let parsed = match value {
        Value::Number(number) => number.as_f64().map(|value| RawFloatOrPercent {
            value,
            serialized: number.to_string(),
        }),
        Value::String(text) => parse_raw_float_or_percent_text(text),
        _ => None,
    }
    .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a number")))?;

    if parsed.value.is_finite() {
        Ok(parsed)
    } else {
        Err(SliceError::InvalidInput(format!(
            "{key} contains invalid value"
        )))
    }
}

fn parse_raw_float_or_percent_text(text: &str) -> Option<RawFloatOrPercent> {
    let trimmed = text.trim();
    if let Some(percent) = trimmed.strip_suffix('%') {
        percent
            .trim()
            .parse::<f64>()
            .ok()
            .map(|value| RawFloatOrPercent {
                value,
                serialized: trimmed.to_owned(),
            })
    } else {
        trimmed.parse::<f64>().ok().map(|value| RawFloatOrPercent {
            value,
            serialized: trimmed.to_owned(),
        })
    }
}

fn is_source_range_valid(value: f64, min: f64, max: f64) -> bool {
    if min == 0.0 && value < 0.0 {
        return false;
    }
    (value - min).abs() < RANGE_EPSILON
        || (value - max).abs() < RANGE_EPSILON
        || (min..=max).contains(&value)
}
