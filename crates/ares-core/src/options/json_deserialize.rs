//! Config.cpp:927-929/952-1006 maps preset JSON strings/arrays to ConfigOption wire values.
mod scalar;
#[cfg(test)]
mod tests;
use serde::de::DeserializeOwned;

use super::{
    config_export::value::escape_scalar_string,
    model_config_deserialize::wire::{self, WireValue},
    registry::{OptionValueKind, option_definition},
};
use crate::project::raw_settings::RawConfigValue;

pub(crate) fn deserialize_preset_config<T: DeserializeOwned>(
    input: RawConfigValue,
) -> Result<T, String> {
    let RawConfigValue::Object(mut values) = input else {
        unreachable!("preset config loader receives an option object")
    };
    for (key, value) in &mut values {
        let Some(definition) = option_definition(key) else {
            continue;
        };
        let kind = definition.kind;
        let decoded = match value {
            RawConfigValue::Text(value) if is_vector(kind) || scalar::is_scalar(kind) => {
                Some(decode(kind, value))
            }
            RawConfigValue::List(values) if is_string_array(values) => Some(
                encode_array(kind, values)
                    .map_err(str::to_owned)
                    .and_then(|wire| decode(kind, &wire)),
            ),
            // Numeric/boolean typed representations are not preset wire strings.
            _ => None,
        };
        if let Some(decoded) = decoded {
            *value = decoded.map_err(|error| format!("invalid Orca option {key}: {error}"))?;
        }
    }
    T::deserialize(RawConfigValue::Object(values)).map_err(|error| error.to_string())
}

fn is_vector(kind: OptionValueKind) -> bool {
    match kind {
        OptionValueKind::Percents
        | OptionValueKind::PercentsNullable
        | OptionValueKind::Bools
        | OptionValueKind::BoolsNullable
        | OptionValueKind::Enums
        | OptionValueKind::EnumsNullable
        | OptionValueKind::Floats
        | OptionValueKind::FloatsNullable
        | OptionValueKind::IntsNullable
        | OptionValueKind::Ints
        | OptionValueKind::Strings
        | OptionValueKind::Points
        | OptionValueKind::PointsGroups => true,
        OptionValueKind::Float
        | OptionValueKind::FloatOrPercent
        | OptionValueKind::Percent
        | OptionValueKind::Int
        | OptionValueKind::Bool
        | OptionValueKind::Enum
        | OptionValueKind::String
        | OptionValueKind::Point => false,
    }
}

fn decode(kind: OptionValueKind, value: &str) -> Result<RawConfigValue, String> {
    let value = scalar::normalize(kind, value)?;
    if scalar::is_scalar(kind) {
        wire::validate_lexical(kind, &value)?;
    }
    wire::decode(kind, &value).map(|value| match value {
        WireValue::Scalar(value) => RawConfigValue::Text(value),
        WireValue::Sequence(values) => {
            RawConfigValue::List(values.into_iter().map(RawConfigValue::Text).collect())
        }
    })
}

fn is_string_array(values: &[RawConfigValue]) -> bool {
    values.iter().all(|value| match value {
        RawConfigValue::Text(_) => true,
        RawConfigValue::List(values) => is_string_array(values),
        _ => false,
    })
}

fn encode_array(kind: OptionValueKind, values: &[RawConfigValue]) -> Result<String, &'static str> {
    let mut wire = String::new();
    let nested = matches!(values.first(), Some(RawConfigValue::List(_)));
    for (index, value) in values.iter().enumerate() {
        if index != 0 {
            wire.push(if nested || kind == OptionValueKind::PointsGroups {
                '#'
            } else if kind == OptionValueKind::Strings {
                ';'
            } else {
                ','
            });
        }
        match value {
            RawConfigValue::Text(value) if !nested => {
                if kind == OptionValueKind::Strings {
                    wire.push('"');
                    wire.push_str(&escape_scalar_string(value));
                    wire.push('"');
                } else {
                    wire.push_str(value);
                }
            }
            RawConfigValue::List(values) if nested => wire.push_str(&encode_array(kind, values)?),
            _ => return Err("inconsistent Orca JSON array element types"),
        }
    }
    Ok(wire)
}
