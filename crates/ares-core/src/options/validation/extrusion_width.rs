use std::collections::BTreeMap;

use serde_json::Value;

use crate::SliceError;

use super::super::SliceOptions;
use super::helpers::registry_default;

const MAX_LINE_WIDTH_MULTIPLIER: f64 = 5.0;
const WIDTH_KEYS: &[&str] = &[
    "outer_wall_line_width",
    "inner_wall_line_width",
    "sparse_infill_line_width",
    "internal_solid_infill_line_width",
    "top_surface_line_width",
    "support_line_width",
    "initial_layer_line_width",
    "skin_infill_line_width",
    "skeleton_infill_line_width",
];

#[derive(Clone, Copy)]
struct FloatOrPercent {
    value: f64,
    percent: bool,
}

impl SliceOptions {
    pub fn validate_extrusion_width_options(&self) -> Result<BTreeMap<String, String>, SliceError> {
        let mut errors = BTreeMap::new();
        let max_nozzle_diameter = self.nozzle_diameters()?.into_iter().fold(0.0_f64, f64::max);
        let limit = MAX_LINE_WIDTH_MULTIPLIER * max_nozzle_diameter;

        for key in WIDTH_KEYS {
            let width = self.float_or_percent_or_default(key)?;
            let predicate_value = width.absolute_value(max_nozzle_diameter);
            if predicate_value <= limit {
                continue;
            }
            let message_value =
                self.line_width_message_value(key, width, max_nozzle_diameter, predicate_value)?;
            errors.insert(
                (*key).to_owned(),
                format!("too large line width {message_value:.6}"),
            );
        }

        Ok(errors)
    }

    fn float_or_percent_or_default(&self, key: &str) -> Result<FloatOrPercent, SliceError> {
        match self.values().get(key) {
            Some(value) => parse_float_or_percent(key, value),
            None => parse_float_or_percent(key, &Value::String(registry_default(key)?.to_owned())),
        }
    }

    fn line_width_message_value(
        &self,
        key: &str,
        width: FloatOrPercent,
        max_nozzle_diameter: f64,
        predicate_value: f64,
    ) -> Result<f64, SliceError> {
        if width.percent {
            return Ok(predicate_value);
        }
        if width.value != 0.0 || !key.ends_with("_line_width") {
            return Ok(width.value);
        }
        let line_width = self.float_or_percent_or_default("line_width")?;
        Ok(line_width.absolute_value(max_nozzle_diameter))
    }
}

impl FloatOrPercent {
    fn absolute_value(self, ratio_over: f64) -> f64 {
        if self.percent {
            ratio_over * self.value / 100.0
        } else {
            self.value
        }
    }
}

fn parse_float_or_percent(key: &str, value: &Value) -> Result<FloatOrPercent, SliceError> {
    let width = match value {
        Value::Number(number) => number.as_f64().map(|value| FloatOrPercent {
            value,
            percent: false,
        }),
        Value::String(text) => parse_float_or_percent_text(text),
        _ => None,
    }
    .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a number")))?;

    if width.value.is_finite() {
        Ok(width)
    } else {
        Err(SliceError::InvalidInput(format!(
            "{key} contains invalid value"
        )))
    }
}

fn parse_float_or_percent_text(text: &str) -> Option<FloatOrPercent> {
    let text = text.trim();
    if let Some(percent) = text.strip_suffix('%') {
        percent
            .trim()
            .parse::<f64>()
            .ok()
            .map(|value| FloatOrPercent {
                value,
                percent: true,
            })
    } else {
        text.parse::<f64>().ok().map(|value| FloatOrPercent {
            value,
            percent: false,
        })
    }
}
