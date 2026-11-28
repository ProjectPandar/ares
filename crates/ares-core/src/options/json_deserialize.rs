//! Config.cpp:927-929/952-1006 maps preset JSON strings/arrays to ConfigOption wire values.
mod scalar;
mod substitutions;

pub(crate) use substitutions::normalize_unknown_wire_substitutions;

use std::borrow::Cow;
use std::collections::BTreeMap;

use serde::{
    Deserialize,
    de::{
        DeserializeOwned,
        value::{BorrowedStrDeserializer, MapDeserializer},
    },
};

use super::{
    config_export::value::escape_scalar_string,
    model_config_deserialize::wire::{self, WireValue},
    registry::{OptionValueKind, option_definition},
};
use crate::project::raw_settings::{RawConfigValue, RawValueRef};

pub(crate) fn deserialize_preset_config<T: DeserializeOwned>(
    input: RawConfigValue,
) -> Result<T, String> {
    T::deserialize(RawConfigValue::Object(decode_preset_values(input, true)?))
        .map_err(|error| error.to_string())
}

// A refreshed project retains its loaded scalar strings but still runs the composite builder.
pub(crate) fn deserialize_refreshed_config(
    input: RawConfigValue,
) -> Result<super::ProjectSettingsBuilder, String> {
    super::ProjectSettingsBuilder::deserialize(RawConfigValue::Object(decode_preset_values(
        input, false,
    )?))
    .map_err(|error| error.to_string())
}

// Config application does not load another JSON file or rerun its composite conversion.
pub(crate) fn deserialize_config_values(
    input: RawConfigValue,
) -> Result<super::ProjectSettingsBuilder, String> {
    crate::options::project_deserialize::deserialize_config_values(RawConfigValue::Object(
        decode_preset_values(input, false)?,
    ))
}

/// Borrowing variant of `deserialize_config_values` avoiding full `BTreeMap` and key/value clones.
pub(crate) fn deserialize_config_values_ref(
    values: &BTreeMap<String, RawConfigValue>,
) -> Result<super::ProjectSettingsBuilder, String> {
    let mut entries = Vec::with_capacity(values.len());
    for (key, value) in values {
        let entry = match option_definition(key) {
            Some(definition) => decode_borrowed_value(key, definition.kind, value)?,
            None => RawValueRef::Borrowed(value),
        };
        entries.push((BorrowedStrDeserializer::new(key.as_str()), entry));
    }
    crate::options::project_deserialize::deserialize_config_map(MapDeserializer::new(
        entries.into_iter(),
    ))
}

fn decode_borrowed_value<'a>(
    key: &str,
    kind: OptionValueKind,
    value: &'a RawConfigValue,
) -> Result<RawValueRef<'a>, String> {
    match value {
        RawConfigValue::Text(text) if scalar::is_scalar(kind) => {
            let normalized = scalar::normalize(kind, text)
                .map_err(|error| format!("invalid Orca option {key}: {error}"))?;
            wire::validate_lexical(kind, &normalized)
                .map_err(|error| format!("invalid Orca option {key}: {error}"))?;
            if normalized.as_ref() == text.as_str() {
                Ok(RawValueRef::Borrowed(value))
            } else {
                Ok(RawValueRef::Owned(RawConfigValue::Text(
                    normalized.into_owned(),
                )))
            }
        }
        RawConfigValue::Text(text) if is_vector(kind) => decode(kind, text)
            .map(RawValueRef::Owned)
            .map_err(|error| format!("invalid Orca option {key}: {error}")),
        RawConfigValue::List(values) if is_string_array(values) => encode_array(kind, values)
            .map_err(str::to_owned)
            .and_then(|wire| decode(kind, &wire))
            .map(RawValueRef::Owned)
            .map_err(|error| format!("invalid Orca option {key}: {error}")),
        _ => Ok(RawValueRef::Borrowed(value)),
    }
}

fn decode_preset_values(
    input: RawConfigValue,
    decode_scalar_strings: bool,
) -> Result<BTreeMap<String, RawConfigValue>, String> {
    let RawConfigValue::Object(mut values) = input else {
        unreachable!("preset config loader receives an option object")
    };
    for (key, value) in &mut values {
        let Some(definition) = option_definition(key) else {
            continue;
        };
        let kind = definition.kind;
        let decoded = match value {
            RawConfigValue::Text(value)
                if is_vector(kind)
                    || scalar::is_scalar(kind)
                    || (decode_scalar_strings && kind == OptionValueKind::String) =>
            {
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
    Ok(values)
}

/// Bare empty preset strings on vector-kind options deserialize to EMPTY
/// vectors upstream (`ConfigOptionStrings::deserialize` Config.hpp:1153-1159
/// clears then leaves the values empty because `unescape_strings_cstyle`
/// returns without appending, Config.cpp:146-149; the scalar JSON branch
/// Config.cpp:927-929 routes bare strings there). Raw views must carry the
/// same shape, so both raw entry points (catalog documents and 3MF embedded
/// settings) normalize through this function.
/// The shared per-kind scalar deserialize for a joined array wire
/// (`Config.cpp:947-1000` → `set_deserialize`): numeric kinds keep their
/// `istringstream` prefix (`Config.hpp:784-790`), booleans their first
/// trimmed `','`-segment (`Config.hpp:1828-1850`), and every other kind the
/// whole string (an empty join deserializes as an empty `coString`,
/// `Config.hpp:1106-1110`).
pub(crate) fn scalar_token(kind: OptionValueKind, value: &str) -> Result<Cow<'_, str>, String> {
    scalar::normalize(kind, value)
}

pub(crate) fn normalize_empty_vector_strings(values: &mut BTreeMap<String, RawConfigValue>) {
    for (key, value) in values {
        let Some(definition) = option_definition(key) else {
            continue;
        };
        if !is_vector(definition.kind)
            || !matches!(value, RawConfigValue::Text(text) if text.is_empty())
        {
            continue;
        }
        *value = match definition.kind {
            OptionValueKind::Floats | OptionValueKind::FloatsNullable => {
                RawConfigValue::List(vec![RawConfigValue::Text("0".to_owned())])
            }
            _ => RawConfigValue::List(Vec::new()),
        };
    }
}

/// Decode the catalog's authored wire scalars before consumers select vector slots.
/// JSON arrays already supply logical members and must not be split again.
pub(crate) fn normalize_preset_string_vectors(
    values: &mut BTreeMap<String, RawConfigValue>,
) -> Result<(), String> {
    for (key, value) in values {
        if let RawConfigValue::Text(text) = value
            && option_definition(key).is_some_and(|def| def.kind == OptionValueKind::Strings)
        {
            *value = decode(OptionValueKind::Strings, text)
                .map_err(|error| format!("invalid Orca option {key}: {error}"))?;
        }
    }
    Ok(())
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
/// is handled by `normalize_empty_vector_strings`, which must run first;
/// ordinary and nullable coFloats use the shared empty-wire rule and become
/// `[0]`, while coStrings remain empty. coStrings JSON arrays take the same
/// quote/join/decode route as presets (`Config.cpp:945-1001`), retaining
/// logical slots for the typed builder. Nullable coFloats arrays use their wire
/// decoder before validation and export (`Config.hpp:932-958`).
pub(crate) fn normalize_scalar_vector_strings(
    values: &mut BTreeMap<String, RawConfigValue>,
) -> Result<(), String> {
    for (key, value) in values {
        let Some(definition) = option_definition(key) else {
            continue;
        };
        let decoded = match value {
            RawConfigValue::Text(text) if !text.is_empty() && is_vector(definition.kind) => {
                Some(decode(definition.kind, text))
            }
            RawConfigValue::List(values)
                if definition.kind == OptionValueKind::Strings
                    || (matches!(
                        definition.kind,
                        OptionValueKind::Floats | OptionValueKind::FloatsNullable
                    ) && is_string_array(values)) =>
            {
                Some(
                    encode_array(definition.kind, values)
                        .map_err(str::to_owned)
                        .and_then(|wire| decode(definition.kind, &wire)),
                )
            }
            _ => None,
        };
        if let Some(decoded) = decoded {
            *value = decoded.map_err(|error| format!("invalid Orca option {key}: {error}"))?;
        }
    }
    Ok(())
}

/// `Config.cpp:1004-1060` joins typed JSON string arrays before scalar
/// deserialization: numeric kinds read their prefix, while coString keeps
/// the comma-joined text. Keep raw presence views consistent with the typed
/// scalar without coercing non-string array elements.
pub(crate) fn normalize_scalar_arrays(
    values: &mut BTreeMap<String, RawConfigValue>,
) -> Result<(), String> {
    for (mut key, mut value) in std::mem::take(values) {
        if let Some(definition) = option_definition(&key)
            && matches!(
                definition.kind,
                OptionValueKind::Float
                    | OptionValueKind::Int
                    | OptionValueKind::Percent
                    | OptionValueKind::FloatOrPercent
                    | OptionValueKind::String
            )
            && let RawConfigValue::List(items) = &value
            && is_string_array(items)
        {
            let mut wire = encode_array(definition.kind, items)
                .map_err(|error| format!("invalid Orca option {key}: {error}"))?;
            // `set_deserialize` handles the complete joined token before the
            // scalar deserializer consumes its prefix (Config.cpp:577-579).
            super::handle_legacy(&mut key, &mut wire);
            if key.is_empty() {
                continue;
            }
            value = decode(definition.kind, &wire)
                .map_err(|error| format!("invalid Orca option {key}: {error}"))?;
        }
        values.insert(key, value);
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
        | OptionValueKind::FloatsOrPercents
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

pub(crate) fn is_string_array(values: &[RawConfigValue]) -> bool {
    values.iter().all(|value| match value {
        RawConfigValue::Text(_) => true,
        RawConfigValue::List(values) => is_string_array(values),
        _ => false,
    })
}

pub(crate) fn encode_array(
    kind: OptionValueKind,
    values: &[RawConfigValue],
) -> Result<String, &'static str> {
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
