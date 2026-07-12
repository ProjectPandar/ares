use std::collections::BTreeSet;

use serde_json::{Number, Value};

use crate::{OptionValueKind, SliceError};

use super::{SliceOptions, option_definition};

pub struct MultiToMulti2Update<'a> {
    pub src_extruder_variants: &'a [&'a str],
    pub dst_extruder_variants: &'a [&'a str],
    pub dst_config: &'a SliceOptions,
    pub key_set: &'a [&'a str],
}

impl SliceOptions {
    pub fn update_values_from_multi_to_multi_2_float_keys(
        &mut self,
        update: MultiToMulti2Update<'_>,
    ) -> Result<isize, SliceError> {
        let same_variant_indices =
            same_variant_indices(update.src_extruder_variants, update.dst_extruder_variants);
        let key_set = update.key_set.iter().copied().collect::<BTreeSet<_>>();
        let mut copied_values = Vec::new();
        for (key, source) in self.values() {
            if !key_set.contains(key.as_str()) {
                continue;
            }
            let Some(definition) = option_definition(key) else {
                continue;
            };
            if !matches!(
                definition.kind,
                OptionValueKind::Floats
                    | OptionValueKind::FloatsNullable
                    | OptionValueKind::FloatOrPercent
                    | OptionValueKind::Bools
                    | OptionValueKind::BoolsNullable
            ) {
                continue;
            }
            let Some(destination) = update.dst_config.values().get(key) else {
                return Err(SliceError::InvalidInput(format!(
                    "{key} must exist in destination config"
                )));
            };
            let value = match definition.kind {
                OptionValueKind::FloatOrPercent => merged_float_or_percent_array(Float2Merge {
                    key,
                    source,
                    destination,
                    src_variant_count: update.src_extruder_variants.len(),
                    dst_variant_count: update.dst_extruder_variants.len(),
                    same_variant_indices: &same_variant_indices,
                })?,
                OptionValueKind::Floats | OptionValueKind::FloatsNullable => {
                    merged_float_array(Float2Merge {
                        key,
                        source,
                        destination,
                        src_variant_count: update.src_extruder_variants.len(),
                        dst_variant_count: update.dst_extruder_variants.len(),
                        same_variant_indices: &same_variant_indices,
                    })?
                }
                OptionValueKind::Bools | OptionValueKind::BoolsNullable => {
                    merged_bool_array(Float2Merge {
                        key,
                        source,
                        destination,
                        src_variant_count: update.src_extruder_variants.len(),
                        dst_variant_count: update.dst_extruder_variants.len(),
                        same_variant_indices: &same_variant_indices,
                    })?
                }
                _ => unreachable!(),
            };
            copied_values.push((key.to_owned(), Value::Array(value)));
        }
        self.values.extend(copied_values);

        Ok(0)
    }
}

struct Float2Merge<'a> {
    key: &'a str,
    source: &'a Value,
    destination: &'a Value,
    src_variant_count: usize,
    dst_variant_count: usize,
    same_variant_indices: &'a [Vec<usize>],
}

#[derive(Clone, Copy)]
enum NullableFloat {
    Nil,
    Value(f64),
}

fn merged_float_array(merge: Float2Merge<'_>) -> Result<Vec<Value>, SliceError> {
    let source = nullable_float_array(merge.source, merge.key)?;
    if source.len() != merge.src_variant_count {
        return Err(SliceError::InvalidInput(format!(
            "{} must match source variant count",
            merge.key
        )));
    }

    let mut destination = nullable_float_array(merge.destination, merge.key)?;
    if destination.len() != merge.dst_variant_count {
        return Err(SliceError::InvalidInput(format!(
            "{} must match destination variant count",
            merge.key
        )));
    }

    for (dst_index, source_indices) in merge.same_variant_indices.iter().enumerate() {
        let mut minimum = None;
        for source_index in source_indices {
            let NullableFloat::Value(value) = source[*source_index] else {
                continue;
            };
            minimum = Some(minimum.map_or(value, |minimum: f64| minimum.min(value)));
        }
        if let Some(value) = minimum {
            destination[dst_index] = NullableFloat::Value(value);
        }
    }

    destination
        .into_iter()
        .map(|value| nullable_float_value(value, merge.key))
        .collect()
}

#[derive(Clone, Copy)]
enum NullableFloatOrPercent {
    Nil,
    Value { value: f64, percent: bool },
}

fn merged_float_or_percent_array(merge: Float2Merge<'_>) -> Result<Vec<Value>, SliceError> {
    let source = nullable_float_or_percent_array(merge.source, merge.key)?;
    if source.len() != merge.src_variant_count {
        return Err(SliceError::InvalidInput(format!(
            "{} must match source variant count",
            merge.key
        )));
    }

    let mut destination = nullable_float_or_percent_array(merge.destination, merge.key)?;
    if destination.len() != merge.dst_variant_count {
        return Err(SliceError::InvalidInput(format!(
            "{} must match destination variant count",
            merge.key
        )));
    }

    for (dst_index, source_indices) in merge.same_variant_indices.iter().enumerate() {
        let mut has_value = false;
        let mut candidate = NullableFloatOrPercent::Value {
            value: 9999.0,
            percent: true,
        };
        for source_index in source_indices {
            let NullableFloatOrPercent::Value { value, percent } = source[*source_index] else {
                continue;
            };
            has_value = true;
            let NullableFloatOrPercent::Value {
                value: candidate_value,
                ..
            } = candidate
            else {
                unreachable!();
            };
            if value < candidate_value {
                candidate = NullableFloatOrPercent::Value { value, percent };
            }
        }
        if has_value {
            destination[dst_index] = candidate;
        }
    }

    destination
        .into_iter()
        .map(|value| nullable_float_or_percent_value(value, merge.key))
        .collect()
}

fn nullable_float_or_percent_array(
    value: &Value,
    key: &str,
) -> Result<Vec<NullableFloatOrPercent>, SliceError> {
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a FloatOrPercent array")))?;
    values
        .iter()
        .map(|value| match value {
            Value::Number(number) => number
                .as_f64()
                .filter(|value| value.is_finite())
                .map(|value| NullableFloatOrPercent::Value {
                    value,
                    percent: false,
                })
                .ok_or_else(|| {
                    SliceError::InvalidInput(format!(
                        "{key} must contain finite FloatOrPercent values or nil"
                    ))
                }),
            Value::String(text) if text == "nil" => Ok(NullableFloatOrPercent::Nil),
            Value::String(text) => parse_float_or_percent(text, key),
            _ => Err(SliceError::InvalidInput(format!(
                "{key} must contain finite FloatOrPercent values or nil"
            ))),
        })
        .collect()
}

fn parse_float_or_percent(text: &str, key: &str) -> Result<NullableFloatOrPercent, SliceError> {
    let text = text.trim();
    let (text, percent) = text
        .strip_suffix('%')
        .map_or((text, false), |text| (text.trim(), true));
    let value = text.parse::<f64>().map_err(|_| {
        SliceError::InvalidInput(format!(
            "{key} must contain finite FloatOrPercent values or nil"
        ))
    })?;
    if value.is_finite() {
        Ok(NullableFloatOrPercent::Value { value, percent })
    } else {
        Err(SliceError::InvalidInput(format!(
            "{key} must contain finite FloatOrPercent values or nil"
        )))
    }
}

fn nullable_float_or_percent_value(
    value: NullableFloatOrPercent,
    key: &str,
) -> Result<Value, SliceError> {
    match value {
        NullableFloatOrPercent::Nil => Ok(Value::String("nil".to_owned())),
        NullableFloatOrPercent::Value { value, percent } => {
            if percent {
                Ok(Value::String(format!("{value}%")))
            } else {
                finite_float_value(value, key)
            }
        }
    }
}

#[derive(Clone, Copy)]
enum NullableBool {
    Nil,
    Value(bool),
}

fn merged_bool_array(merge: Float2Merge<'_>) -> Result<Vec<Value>, SliceError> {
    let source = nullable_bool_array(merge.source, merge.key)?;
    if source.len() != merge.src_variant_count {
        return Err(SliceError::InvalidInput(format!(
            "{} must match source variant count",
            merge.key
        )));
    }

    let mut destination = nullable_bool_array(merge.destination, merge.key)?;
    if destination.len() != merge.dst_variant_count {
        return Err(SliceError::InvalidInput(format!(
            "{} must match destination variant count",
            merge.key
        )));
    }

    for (dst_index, source_indices) in merge.same_variant_indices.iter().enumerate() {
        for source_index in source_indices {
            let NullableBool::Value(value) = source[*source_index] else {
                continue;
            };
            destination[dst_index] = NullableBool::Value(value);
            break;
        }
    }

    destination.into_iter().map(nullable_bool_value).collect()
}

fn nullable_bool_array(value: &Value, key: &str) -> Result<Vec<NullableBool>, SliceError> {
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a bool array")))?;
    values
        .iter()
        .map(|value| nullable_bool(value, key))
        .collect()
}

fn nullable_bool(value: &Value, key: &str) -> Result<NullableBool, SliceError> {
    match value {
        Value::Bool(value) => Ok(NullableBool::Value(*value)),
        Value::Number(number) if number.as_u64() == Some(1) => Ok(NullableBool::Value(true)),
        Value::Number(number) if number.as_u64() == Some(0) => Ok(NullableBool::Value(false)),
        Value::String(text) if text == "1" => Ok(NullableBool::Value(true)),
        Value::String(text) if text == "0" => Ok(NullableBool::Value(false)),
        Value::String(text) if text == "nil" => Ok(NullableBool::Nil),
        _ => Err(SliceError::InvalidInput(format!(
            "{key} must contain bool values or nil"
        ))),
    }
}

fn nullable_bool_value(value: NullableBool) -> Result<Value, SliceError> {
    match value {
        NullableBool::Nil => Ok(Value::String("nil".to_owned())),
        NullableBool::Value(value) => Ok(Value::Bool(value)),
    }
}

fn same_variant_indices(src_variants: &[&str], dst_variants: &[&str]) -> Vec<Vec<usize>> {
    dst_variants
        .iter()
        .map(|dst_variant| {
            src_variants
                .iter()
                .enumerate()
                .filter_map(|(index, src_variant)| (*src_variant == *dst_variant).then_some(index))
                .collect()
        })
        .collect()
}

fn nullable_float_array(value: &Value, key: &str) -> Result<Vec<NullableFloat>, SliceError> {
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a float array")))?;
    values
        .iter()
        .map(|value| match value {
            Value::Number(number) => number
                .as_f64()
                .filter(|value| value.is_finite())
                .map(NullableFloat::Value)
                .ok_or_else(|| {
                    SliceError::InvalidInput(format!("{key} must contain finite floats or nil"))
                }),
            Value::String(text) if text == "nil" => Ok(NullableFloat::Nil),
            _ => Err(SliceError::InvalidInput(format!(
                "{key} must contain finite floats or nil"
            ))),
        })
        .collect()
}

fn nullable_float_value(value: NullableFloat, key: &str) -> Result<Value, SliceError> {
    match value {
        NullableFloat::Nil => Ok(Value::String("nil".to_owned())),
        NullableFloat::Value(value) => finite_float_value(value, key),
    }
}

fn finite_float_value(value: f64, key: &str) -> Result<Value, SliceError> {
    Number::from_f64(value)
        .map(Value::Number)
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must contain finite floats")))
}
