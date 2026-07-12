use std::collections::BTreeSet;

mod float_or_percent;
mod multi_to_single_2;
mod second;

use serde_json::{Number, Value};

use crate::{OptionValueKind, SliceError};

use super::{SliceOptions, registry::option_definition};
use float_or_percent::{FloatOrPercentMerge, merged_float_or_percent_array};
pub use second::MultiToMulti2Update;

pub struct MultiToMultiUpdate<'a> {
    pub new_config: &'a SliceOptions,
    pub key_set: &'a [&'a str],
    pub id_name: &'a str,
    pub variant_name: &'a str,
    pub new_extruder_variants: &'a [&'a str],
}

impl SliceOptions {
    pub fn update_values_from_multi_to_multi_string_int_float_percent_bool_keys(
        &mut self,
        update: MultiToMultiUpdate<'_>,
    ) -> Result<isize, SliceError> {
        let Some(current_variant_value) = self.values().get(update.variant_name) else {
            return Ok(-1);
        };
        let Some(new_variant_value) = update.new_config.values().get(update.variant_name) else {
            return Ok(-1);
        };
        let Some(new_id_value) = update.new_config.values().get(update.id_name) else {
            return Ok(-1);
        };

        let current_variants = string_array_value(current_variant_value, update.variant_name)?;
        let new_variants = string_array_value(new_variant_value, update.variant_name)?;
        let new_ids = int_array_values(new_id_value, update.id_name)?;
        let variant_count = current_variants.len();
        let new_variant_count = new_variants.len();

        let extruder_variant_indices =
            extruder_variant_indices(update.new_extruder_variants, &current_variants);
        let new_variant_indices =
            new_variant_indices(update.new_extruder_variants, &new_variants, &new_ids);

        let keys = update.key_set.iter().copied().collect::<BTreeSet<_>>();
        let mut copied_values = Vec::new();
        for key in keys {
            let Some(definition) = option_definition(key) else {
                continue;
            };
            let Some(value) = update.new_config.values().get(key) else {
                continue;
            };
            match definition.kind {
                OptionValueKind::Strings => {
                    copied_values.push((
                        key.to_owned(),
                        Value::Array(string_array_value(value, key)?),
                    ));
                }
                OptionValueKind::Ints => {
                    copied_values
                        .push((key.to_owned(), Value::Array(int_array_value(value, key)?)));
                }
                OptionValueKind::Floats => copied_values.push((
                    key.to_owned(),
                    Value::Array(merged_float_array(FloatMerge {
                        key,
                        target: self.values.get(key),
                        source: value,
                        variant_count,
                        new_variant_count,
                        extruder_variant_indices: &extruder_variant_indices,
                        new_variant_indices: &new_variant_indices,
                    })?),
                )),
                OptionValueKind::FloatOrPercent => copied_values.push((
                    key.to_owned(),
                    Value::Array(merged_float_or_percent_array(FloatOrPercentMerge {
                        key,
                        target: self.values.get(key),
                        source: value,
                        variant_count,
                        new_variant_count,
                        extruder_variant_indices: &extruder_variant_indices,
                        new_variant_indices: &new_variant_indices,
                    })?),
                )),
                OptionValueKind::Bools => copied_values.push((
                    key.to_owned(),
                    Value::Array(merged_bool_array(BoolMerge {
                        key,
                        target: self.values.get(key),
                        source: value,
                        variant_count,
                        new_variant_count,
                        extruder_variant_indices: &extruder_variant_indices,
                        new_variant_indices: &new_variant_indices,
                    })?),
                )),
                _ => {}
            }
        }
        self.values.extend(copied_values);

        Ok(0)
    }
}

struct BoolMerge<'a> {
    key: &'a str,
    target: Option<&'a Value>,
    source: &'a Value,
    variant_count: usize,
    new_variant_count: usize,
    extruder_variant_indices: &'a [Vec<usize>],
    new_variant_indices: &'a [isize],
}

fn merged_bool_array(merge: BoolMerge<'_>) -> Result<Vec<Value>, SliceError> {
    let old_values = match merge.target {
        Some(value) => bool_array_values(value, merge.key)?,
        None => vec![default_bool_value(merge.key)?; merge.variant_count],
    };
    if old_values.len() != merge.variant_count {
        return Err(SliceError::InvalidInput(format!(
            "{} must match variant count",
            merge.key
        )));
    }

    let mut merged_values = bool_array_values(merge.source, merge.key)?;
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
            if old_values[*old_index] {
                merged_values[new_variant_index] = true;
            }
        }
    }

    Ok(merged_values.into_iter().map(Value::Bool).collect())
}

struct FloatMerge<'a> {
    key: &'a str,
    target: Option<&'a Value>,
    source: &'a Value,
    variant_count: usize,
    new_variant_count: usize,
    extruder_variant_indices: &'a [Vec<usize>],
    new_variant_indices: &'a [isize],
}

fn merged_float_array(merge: FloatMerge<'_>) -> Result<Vec<Value>, SliceError> {
    let old_values = match merge.target {
        Some(value) => finite_float_array(value, merge.key)?,
        None => vec![default_float_value(merge.key)?; merge.variant_count],
    };
    if old_values.len() != merge.variant_count {
        return Err(SliceError::InvalidInput(format!(
            "{} must match variant count",
            merge.key
        )));
    }

    let mut merged_values = finite_float_array(merge.source, merge.key)?;
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
            if old_values[*old_index] < merged_values[new_variant_index] {
                merged_values[new_variant_index] = old_values[*old_index];
            }
        }
    }

    merged_values
        .into_iter()
        .map(|value| float_value(value, merge.key))
        .collect()
}

fn extruder_variant_indices(
    new_extruder_variants: &[&str],
    current_variants: &[Value],
) -> Vec<Vec<usize>> {
    new_extruder_variants
        .iter()
        .map(|variant| {
            let indices = current_variants
                .iter()
                .enumerate()
                .filter_map(|(index, value)| (value.as_str() == Some(*variant)).then_some(index))
                .collect::<Vec<_>>();
            if indices.is_empty() {
                (0..current_variants.len()).collect()
            } else {
                indices
            }
        })
        .collect()
}

fn new_variant_indices(
    new_extruder_variants: &[&str],
    new_variants: &[Value],
    new_ids: &[i32],
) -> Vec<isize> {
    new_extruder_variants
        .iter()
        .enumerate()
        .map(|(extruder_index, variant)| {
            new_variants
                .iter()
                .zip(new_ids)
                .position(|(new_variant, new_id)| {
                    *new_id == extruder_index as i32 + 1 && new_variant.as_str() == Some(*variant)
                })
                .map_or(-1, |index| index as isize)
        })
        .collect()
}

fn finite_float_array(value: &Value, key: &str) -> Result<Vec<f64>, SliceError> {
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a float array")))?;
    values
        .iter()
        .map(|value| {
            value
                .as_f64()
                .filter(|value| value.is_finite())
                .ok_or_else(|| {
                    SliceError::InvalidInput(format!("{key} must contain finite floats"))
                })
        })
        .collect()
}

fn default_float_value(key: &str) -> Result<f64, SliceError> {
    let definition = option_definition(key)
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} has no option definition")))?;
    let first = definition
        .default_value
        .split(',')
        .next()
        .unwrap_or(definition.default_value)
        .trim();
    let value = first
        .parse::<f64>()
        .map_err(|_| SliceError::InvalidInput(format!("{key} default must be a float")))?;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(SliceError::InvalidInput(format!(
            "{key} default must be finite"
        )))
    }
}

fn float_value(value: f64, key: &str) -> Result<Value, SliceError> {
    Number::from_f64(value)
        .map(Value::Number)
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must contain finite floats")))
}

fn string_array_value(value: &Value, key: &str) -> Result<Vec<Value>, SliceError> {
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a string array")))?;
    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(|_| value.clone())
                .ok_or_else(|| SliceError::InvalidInput(format!("{key} must contain strings")))
        })
        .collect()
}

fn bool_array_values(value: &Value, key: &str) -> Result<Vec<bool>, SliceError> {
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a bool array")))?;
    values
        .iter()
        .map(|value| {
            value
                .as_bool()
                .ok_or_else(|| SliceError::InvalidInput(format!("{key} must contain bools")))
        })
        .collect()
}

fn default_bool_value(key: &str) -> Result<bool, SliceError> {
    let definition = option_definition(key)
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} has no option definition")))?;
    let first = definition
        .default_value
        .split(',')
        .next()
        .unwrap_or(definition.default_value)
        .trim();
    match first {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(SliceError::InvalidInput(format!(
            "{key} default must be a bool"
        ))),
    }
}

fn int_array_values(value: &Value, key: &str) -> Result<Vec<i32>, SliceError> {
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be an integer array")))?;
    values
        .iter()
        .map(|value| {
            value
                .as_i64()
                .and_then(|value| i32::try_from(value).ok())
                .ok_or_else(|| SliceError::InvalidInput(format!("{key} must contain i32 integers")))
        })
        .collect()
}

fn int_array_value(value: &Value, key: &str) -> Result<Vec<Value>, SliceError> {
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be an integer array")))?;
    values
        .iter()
        .map(|value| {
            value
                .as_i64()
                .and_then(|value| i32::try_from(value).ok())
                .map(|_| value.clone())
                .ok_or_else(|| SliceError::InvalidInput(format!("{key} must contain i32 integers")))
        })
        .collect()
}
