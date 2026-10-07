//! Config.cpp:927-929/952-1006 maps preset JSON strings/arrays to ConfigOption wire values.
mod scalar;
#[cfg(test)]
mod tests;
use std::collections::BTreeMap;

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

/// Bare empty preset strings on vector-kind options deserialize to EMPTY
/// vectors upstream (`ConfigOptionStrings::deserialize` Config.hpp:1153-1159
/// clears then leaves the values empty because `unescape_strings_cstyle`
/// returns without appending, Config.cpp:146-149; the scalar JSON branch
/// Config.cpp:927-929 routes bare strings there). Raw views must carry the
/// same shape, so both raw entry points (catalog documents and 3MF embedded
/// settings) normalize through this function.
pub(crate) fn normalize_empty_vector_strings(values: &mut BTreeMap<String, RawConfigValue>) {
    for (key, value) in values {
        if let Some(definition) = option_definition(key)
            && is_vector(definition.kind)
            && matches!(value, RawConfigValue::Text(text) if text.is_empty())
        {
            *value = RawConfigValue::List(Vec::new());
        }
    }
}

/// `ConfigBase::load_from_json` routes every JSON string through
/// `set_deserialize` (`Config.cpp:927-929`) without checking the target
/// option's JSON shape first, so a bare scalar reaches
/// `ConfigOptionVector::deserialize` (`Config.hpp:868-887`), which splits
/// on the kind's separator and yields a single-element vector. Preset
/// documents already decode through `deserialize_preset_config`; 3MF
/// embedded settings need the same normalization before the typed
/// builders run. Non-empty scalars decode to lists; the empty-string case
/// keeps the `ConfigOptionStrings::deserialize` empty-vector result and
/// is handled by `normalize_empty_vector_strings`, which must run first.
pub(crate) fn normalize_scalar_vector_strings(
    values: &mut BTreeMap<String, RawConfigValue>,
) -> Result<(), String> {
    for (key, value) in values {
        let Some(definition) = option_definition(key) else {
            continue;
        };
        if let RawConfigValue::Text(text) = value
            && !text.is_empty()
            && is_vector(definition.kind)
        {
            *value = decode(definition.kind, text)
                .map_err(|error| format!("invalid Orca option {key}: {error}"))?;
        }
    }
    Ok(())
}

/// `Config.cpp:945-1001` joins JSON array tokens before scalar
/// deserialization; `Config.hpp:784-790` reads a numeric prefix. Keep the
/// preset presence view consistent with the typed scalar, without applying
/// preset coercion to project documents or user overrides.
pub(crate) fn normalize_numeric_scalar_arrays(
    values: &mut BTreeMap<String, RawConfigValue>,
) -> Result<(), String> {
    for (key, value) in values {
        let Some(definition) = option_definition(key) else {
            continue;
        };
        if matches!(
            definition.kind,
            OptionValueKind::Float
                | OptionValueKind::Int
                | OptionValueKind::Percent
                | OptionValueKind::FloatOrPercent
        ) && let RawConfigValue::List(items) = value
            && is_string_array(items)
        {
            *value = encode_array(definition.kind, items)
                .map_err(str::to_owned)
                .and_then(|wire| decode(definition.kind, &wire))
                .map_err(|error| format!("invalid Orca option {key}: {error}"))?;
        }
    }
    Ok(())
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

pub(crate) fn decode(kind: OptionValueKind, value: &str) -> Result<RawConfigValue, String> {
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
