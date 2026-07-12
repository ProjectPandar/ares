use serde_json::{Number, Value};

use super::registry;
use crate::{OptionValueKind, SliceError};

pub(super) fn resize_array(values: &mut Vec<Value>, target_size: usize, default: Value) {
    if target_size <= values.len() {
        values.truncate(target_size);
        return;
    }

    let fill = values.first().cloned().unwrap_or(default);
    values.resize(target_size, fill);
}

pub(super) fn default_array_member(key: &str) -> Result<Value, SliceError> {
    let definition = registry::option_definition(key)
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} has no option definition")))?;

    match definition.kind {
        OptionValueKind::Float
        | OptionValueKind::FloatOrPercent
        | OptionValueKind::Percent
        | OptionValueKind::Percents
        | OptionValueKind::PercentsNullable
        | OptionValueKind::Int
        | OptionValueKind::Ints
        | OptionValueKind::IntsNullable
        | OptionValueKind::Floats
        | OptionValueKind::FloatsNullable => default_number(definition.default_value, key),
        OptionValueKind::Bool | OptionValueKind::Bools | OptionValueKind::BoolsNullable => {
            default_bool(definition.default_value, key)
        }
        OptionValueKind::Enum
        | OptionValueKind::Enums
        | OptionValueKind::EnumsNullable
        | OptionValueKind::Strings
        | OptionValueKind::String
        | OptionValueKind::Point
        | OptionValueKind::Points
        | OptionValueKind::PointsGroups => Ok(Value::String(definition.default_value.to_owned())),
    }
}

fn default_number(default_value: &str, key: &str) -> Result<Value, SliceError> {
    let first = default_value
        .split(',')
        .next()
        .unwrap_or(default_value)
        .trim();
    let value = first.parse::<f64>().map_err(|_| {
        SliceError::InvalidInput(format!("{key} default must be a numeric array member"))
    })?;
    Number::from_f64(value).map(Value::Number).ok_or_else(|| {
        SliceError::InvalidInput(format!(
            "{key} default must be a finite numeric array member"
        ))
    })
}

fn default_bool(default_value: &str, key: &str) -> Result<Value, SliceError> {
    match default_value {
        "true" => Ok(Value::Bool(true)),
        "false" => Ok(Value::Bool(false)),
        _ => Err(SliceError::InvalidInput(format!(
            "{key} default must be a boolean array member"
        ))),
    }
}
