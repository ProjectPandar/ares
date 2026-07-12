use serde_json::Value;

use crate::SliceError;

use super::{float_value, option_definition};

const EPSILON: f64 = 1e-4;

#[derive(Clone, Copy)]
struct FloatOrPercent {
    value: f64,
    percent: bool,
}

pub(super) struct FloatOrPercentMerge<'a> {
    pub key: &'a str,
    pub target: Option<&'a Value>,
    pub source: &'a Value,
    pub variant_count: usize,
    pub new_variant_count: usize,
    pub extruder_variant_indices: &'a [Vec<usize>],
    pub new_variant_indices: &'a [isize],
}

pub(super) fn merged_float_or_percent_array(
    merge: FloatOrPercentMerge<'_>,
) -> Result<Vec<Value>, SliceError> {
    let old_values = match merge.target {
        Some(value) => float_or_percent_array(value, merge.key)?,
        None => vec![default_float_or_percent_value(merge.key)?; merge.variant_count],
    };
    if old_values.len() != merge.variant_count {
        return Err(SliceError::InvalidInput(format!(
            "{} must match variant count",
            merge.key
        )));
    }

    let mut merged_values = float_or_percent_array(merge.source, merge.key)?;
    if merged_values.len() != merge.new_variant_count {
        return Err(SliceError::InvalidInput(format!(
            "{} must match new variant count",
            merge.key
        )));
    }

    for (variant_indices, new_variant_index) in merge
        .extruder_variant_indices
        .iter()
        .zip(merge.new_variant_indices)
    {
        if *new_variant_index == -1 || variant_indices.is_empty() {
            continue;
        }
        let new_variant_index = *new_variant_index as usize;
        for old_index in variant_indices {
            let old = old_values[*old_index];
            if float_or_percent_less(old, merged_values[new_variant_index]) {
                merged_values[new_variant_index] = old;
            }
        }
    }

    merged_values
        .into_iter()
        .map(|value| float_or_percent_value(value, merge.key))
        .collect()
}

fn float_or_percent_less(left: FloatOrPercent, right: FloatOrPercent) -> bool {
    left.value < right.value
        || (is_approx(left.value, right.value) && !left.percent && right.percent)
}

fn is_approx(left: f64, right: f64) -> bool {
    (left - right).abs() < EPSILON
}

fn float_or_percent_array(value: &Value, key: &str) -> Result<Vec<FloatOrPercent>, SliceError> {
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a FloatOrPercent array")))?;
    values
        .iter()
        .map(|value| match value {
            Value::Number(number) => number
                .as_f64()
                .filter(|value| value.is_finite())
                .map(|value| FloatOrPercent {
                    value,
                    percent: false,
                })
                .ok_or_else(|| {
                    SliceError::InvalidInput(format!(
                        "{key} must contain finite FloatOrPercent values"
                    ))
                }),
            Value::String(text) => parse_float_or_percent(text, key),
            _ => Err(SliceError::InvalidInput(format!(
                "{key} must contain FloatOrPercent values"
            ))),
        })
        .collect()
}

fn parse_float_or_percent(text: &str, key: &str) -> Result<FloatOrPercent, SliceError> {
    let text = text.trim();
    let (text, percent) = text
        .strip_suffix('%')
        .map_or((text, false), |text| (text.trim(), true));
    let value = text.parse::<f64>().map_err(|_| {
        SliceError::InvalidInput(format!("{key} must contain finite FloatOrPercent values"))
    })?;
    if value.is_finite() {
        Ok(FloatOrPercent { value, percent })
    } else {
        Err(SliceError::InvalidInput(format!(
            "{key} must contain finite FloatOrPercent values"
        )))
    }
}

fn default_float_or_percent_value(key: &str) -> Result<FloatOrPercent, SliceError> {
    let definition = option_definition(key)
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} has no option definition")))?;
    let first = definition
        .default_value
        .split(',')
        .next()
        .unwrap_or(definition.default_value);
    parse_float_or_percent(first, key)
}

fn float_or_percent_value(value: FloatOrPercent, key: &str) -> Result<Value, SliceError> {
    if value.percent {
        Ok(Value::String(format!("{}%", value.value)))
    } else {
        float_value(value.value, key)
    }
}
