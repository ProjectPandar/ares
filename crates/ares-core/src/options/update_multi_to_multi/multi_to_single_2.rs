use std::collections::BTreeSet;

use serde_json::{Number, Value};

use crate::{OptionValueKind, SliceError};

use super::{SliceOptions, option_definition};

impl SliceOptions {
    pub fn update_values_from_multi_to_single_2_float_keys(
        &mut self,
        key_set: &[&str],
    ) -> Result<isize, SliceError> {
        let key_set = key_set.iter().copied().collect::<BTreeSet<_>>();
        let mut collapsed_values = Vec::new();
        for (key, source) in self.values() {
            if !key_set.contains(key.as_str()) {
                continue;
            }
            let Some(definition) = option_definition(key) else {
                continue;
            };
            let value = match definition.kind {
                OptionValueKind::Floats | OptionValueKind::FloatsNullable => {
                    collapsed_float_array(source, key)?
                }
                OptionValueKind::FloatOrPercent => collapsed_float_or_percent_array(source, key)?,
                OptionValueKind::Bools | OptionValueKind::BoolsNullable => {
                    collapsed_bool_array(source, key)?
                }
                _ => continue,
            };
            collapsed_values.push((key.to_owned(), Value::Array(value)));
        }
        self.values.extend(collapsed_values);

        Ok(0)
    }
}

#[derive(Clone, Copy)]
enum NullableFloat {
    Nil,
    Value(f64),
}

fn collapsed_float_array(value: &Value, key: &str) -> Result<Vec<Value>, SliceError> {
    let source = nullable_float_array(value, key)?;
    let Some(first) = source.first().copied() else {
        return Err(SliceError::InvalidInput(format!(
            "{key} must contain at least one float"
        )));
    };

    let mut selected = None;
    let mut minimum = 9999.0;
    for value in &source {
        let NullableFloat::Value(value) = value else {
            continue;
        };
        if *value < minimum {
            minimum = *value;
            selected = Some(*value);
        }
    }

    let value = selected.map_or(first, NullableFloat::Value);
    nullable_float_value(value, key).map(|value| vec![value])
}

#[derive(Clone, Copy)]
enum NullableFloatOrPercent {
    Nil,
    Value { value: f64, percent: bool },
}

fn collapsed_float_or_percent_array(value: &Value, key: &str) -> Result<Vec<Value>, SliceError> {
    let source = nullable_float_or_percent_array(value, key)?;
    let Some(first) = source.first().copied() else {
        return Err(SliceError::InvalidInput(format!(
            "{key} must contain at least one FloatOrPercent"
        )));
    };

    let mut selected = None;
    let mut minimum = 9999.0;
    for value in &source {
        let NullableFloatOrPercent::Value { value, percent } = value else {
            continue;
        };
        if *value < minimum {
            minimum = *value;
            selected = Some(NullableFloatOrPercent::Value {
                value: *value,
                percent: *percent,
            });
        }
    }

    let value = selected.unwrap_or(first);
    nullable_float_or_percent_value(value, key).map(|value| vec![value])
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

#[derive(Clone, Copy)]
enum NullableBool {
    Nil,
    Value(bool),
}

fn collapsed_bool_array(value: &Value, key: &str) -> Result<Vec<Value>, SliceError> {
    let source = nullable_bool_array(value, key)?;
    let Some(first) = source.first().copied() else {
        return Err(SliceError::InvalidInput(format!(
            "{key} must contain at least one bool"
        )));
    };

    let selected = source.iter().find_map(|value| match value {
        NullableBool::Nil => None,
        NullableBool::Value(value) => Some(*value),
    });

    let value = selected.map_or(first, NullableBool::Value);
    Ok(vec![nullable_bool_value(value)])
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
        Value::Number(number) if number.as_i64() == Some(0) => Ok(NullableBool::Value(false)),
        Value::Number(number) if number.as_i64() == Some(1) => Ok(NullableBool::Value(true)),
        Value::String(text) if text == "nil" => Ok(NullableBool::Nil),
        Value::String(text) if text == "0" => Ok(NullableBool::Value(false)),
        Value::String(text) if text == "1" => Ok(NullableBool::Value(true)),
        _ => Err(SliceError::InvalidInput(format!(
            "{key} must contain bool values or nil"
        ))),
    }
}

fn nullable_bool_value(value: NullableBool) -> Value {
    match value {
        NullableBool::Nil => Value::String("nil".to_owned()),
        NullableBool::Value(value) => Value::Bool(value),
    }
}
