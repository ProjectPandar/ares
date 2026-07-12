use std::collections::BTreeSet;

use serde_json::{Number, Value};

use crate::{OptionValueKind, SliceError};

use super::{SliceOptions, registry::option_definition};

impl SliceOptions {
    pub fn update_values_from_single_to_multi_string_int_float_percent_bool_keys(
        &mut self,
        multi_config: &SliceOptions,
        key_set: &[&str],
        variant_name: &str,
    ) -> Result<isize, SliceError> {
        let Some(variant_value) = multi_config.values().get(variant_name) else {
            return Ok(-1);
        };
        let variant_count = string_array_value(variant_value, variant_name)?.len();

        let keys = key_set.iter().copied().collect::<BTreeSet<_>>();
        let mut copied_values = Vec::new();
        for key in keys {
            let Some(definition) = option_definition(key) else {
                continue;
            };
            let Some(value) = multi_config.values().get(key) else {
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
                OptionValueKind::Floats => {
                    copied_values.push((
                        key.to_owned(),
                        Value::Array(limited_float_array(
                            key,
                            self.values.get(key),
                            value,
                            variant_count,
                        )?),
                    ));
                }
                OptionValueKind::FloatOrPercent => {
                    copied_values.push((
                        key.to_owned(),
                        Value::Array(limited_float_or_percent_array(
                            key,
                            self.values.get(key),
                            value,
                            variant_count,
                        )?),
                    ));
                }
                OptionValueKind::Bools => {
                    copied_values.push((
                        key.to_owned(),
                        Value::Array(resized_bool_array(
                            key,
                            self.values.get(key),
                            value,
                            variant_count,
                        )?),
                    ));
                }
                _ => {}
            }
        }
        self.values.extend(copied_values);

        Ok(0)
    }
}

fn resized_bool_array(
    key: &str,
    target: Option<&Value>,
    source: &Value,
    variant_count: usize,
) -> Result<Vec<Value>, SliceError> {
    let source = bool_array_value(source, key)?;
    if source.len() != variant_count {
        return Err(SliceError::InvalidInput(format!(
            "{key} must match variant count"
        )));
    }
    let mut target = match target {
        Some(value) => bool_array_value(value, key)?,
        None => vec![Value::Bool(default_bool_value(key)?); variant_count],
    };
    resize_bool_array(&mut target, variant_count, key)?;
    Ok(target)
}

fn resize_bool_array(
    values: &mut Vec<Value>,
    variant_count: usize,
    key: &str,
) -> Result<(), SliceError> {
    if variant_count <= values.len() {
        values.truncate(variant_count);
        return Ok(());
    }

    let fill = values
        .first()
        .cloned()
        .unwrap_or(Value::Bool(default_bool_value(key)?));
    values.resize(variant_count, fill);
    Ok(())
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

#[derive(Clone, Copy)]
struct FloatOrPercent {
    value: f64,
    percent: bool,
}

fn limited_float_or_percent_array(
    key: &str,
    target: Option<&Value>,
    source: &Value,
    variant_count: usize,
) -> Result<Vec<Value>, SliceError> {
    let source = float_or_percent_array(source, key)?;
    if source.len() != variant_count {
        return Err(SliceError::InvalidInput(format!(
            "{key} must match variant count"
        )));
    }
    let mut target = match target {
        Some(value) => float_or_percent_array(value, key)?,
        None => vec![default_float_or_percent_value(key)?; variant_count],
    };
    resize_float_or_percent_array(&mut target, variant_count, key)?;

    target
        .into_iter()
        .zip(source)
        .map(|(target, source)| {
            let limited = if target.value > source.value {
                source
            } else {
                target
            };
            float_or_percent_value(limited, key)
        })
        .collect()
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

fn resize_float_or_percent_array(
    values: &mut Vec<FloatOrPercent>,
    variant_count: usize,
    key: &str,
) -> Result<(), SliceError> {
    if variant_count <= values.len() {
        values.truncate(variant_count);
        return Ok(());
    }

    let fill = values
        .first()
        .copied()
        .unwrap_or(default_float_or_percent_value(key)?);
    values.resize(variant_count, fill);
    Ok(())
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

fn limited_float_array(
    key: &str,
    target: Option<&Value>,
    source: &Value,
    variant_count: usize,
) -> Result<Vec<Value>, SliceError> {
    let source = finite_float_array(source, key)?;
    if source.len() != variant_count {
        return Err(SliceError::InvalidInput(format!(
            "{key} must match variant count"
        )));
    }
    let mut target = match target {
        Some(value) => finite_float_array(value, key)?,
        None => vec![default_float_value(key)?; variant_count],
    };
    resize_float_array(&mut target, variant_count, key)?;

    target
        .into_iter()
        .zip(source)
        .map(|(target, source)| {
            let limited = if target > source { source } else { target };
            float_value(limited, key)
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

fn resize_float_array(
    values: &mut Vec<f64>,
    variant_count: usize,
    key: &str,
) -> Result<(), SliceError> {
    if variant_count <= values.len() {
        values.truncate(variant_count);
        return Ok(());
    }

    let fill = values.first().copied().unwrap_or(default_float_value(key)?);
    values.resize(variant_count, fill);
    Ok(())
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

fn bool_array_value(value: &Value, key: &str) -> Result<Vec<Value>, SliceError> {
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a bool array")))?;
    values
        .iter()
        .map(|value| {
            value
                .as_bool()
                .map(|_| value.clone())
                .ok_or_else(|| SliceError::InvalidInput(format!("{key} must contain bools")))
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
