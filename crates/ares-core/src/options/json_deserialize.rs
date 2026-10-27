//! Config.cpp:927-929/952-1006 maps preset JSON strings/arrays to ConfigOption wire values.
mod scalar;
#[cfg(test)]
mod tests;
use std::borrow::Cow;
use std::collections::BTreeMap;

use serde::de::DeserializeOwned;

use super::{
    config_export::value::escape_scalar_string,
    model_config_deserialize::wire::{self, WireValue},
    registry::{OptionValueKind, option_definition, option_metadata},
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
/// The shared per-kind scalar deserialize for a joined array wire
/// (`Config.cpp:947-1000` → `set_deserialize`): numeric kinds keep their
/// `istringstream` prefix (`Config.hpp:784-790`), booleans their first
/// trimmed `','`-segment (`Config.hpp:1828-1850`), and every other kind the
/// whole string. Empty output keeps the original wire (a join can never
/// produce it, but `scalar::normalize` is also the no-op pass-through for
/// uncovered kinds).
pub(crate) fn scalar_token(kind: OptionValueKind, value: &str) -> Result<Cow<'_, str>, String> {
    scalar::normalize(kind, value)
}

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
/// coStrings JSON arrays take the same quote/join/decode route as presets
/// (`Config.cpp:945-1001`), retaining logical slots for the typed builder.
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
            RawConfigValue::List(values) if definition.kind == OptionValueKind::Strings => Some(
                encode_array(definition.kind, values)
                    .map_err(str::to_owned)
                    .and_then(|wire| decode(definition.kind, &wire)),
            ),
            _ => None,
        };
        if let Some(decoded) = decoded {
            *value = decoded.map_err(|error| format!("invalid Orca option {key}: {error}"))?;
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
    for (mut key, mut value) in std::mem::take(values) {
        if let Some(definition) = option_definition(&key)
            && matches!(
                definition.kind,
                OptionValueKind::Float
                    | OptionValueKind::Int
                    | OptionValueKind::Percent
                    | OptionValueKind::FloatOrPercent
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

/// `Config.cpp:654-672` substitution under the `EnableSilent` 3MF load
/// rule (`Model.cpp:259`): a scalar-bool wire that fails the "0"/"1"
/// parse (`Config.hpp:1828-1848`) retries the `looks_like_true` verdict
/// (`Config.hpp:96-99`: trim + case-insensitive "enabled"/"on"), and an
/// enum-vector wire carrying an unknown entry replaces the whole vector
/// with the option default (`Config.cpp:668-671`). Everything else keeps
/// its value — the typed builders still diagnose other failures.
pub(crate) fn normalize_unknown_wire_substitutions(values: &mut BTreeMap<String, RawConfigValue>) {
    for (key, value) in values {
        let Some(definition) = option_definition(key) else {
            continue;
        };
        match definition.kind {
            OptionValueKind::Bool => {
                let RawConfigValue::Text(text) = value else {
                    continue;
                };
                if matches!(text.split(',').next().unwrap_or_default().trim(), "0" | "1") {
                    continue;
                }
                *value = RawConfigValue::Text(if looks_like_true(text) {
                    "1".to_owned()
                } else {
                    "0".to_owned()
                });
            }
            OptionValueKind::Enum => {
                let RawConfigValue::Text(text) = value else {
                    continue;
                };
                // Scalar coEnum substitution (`Config.cpp:657-676`): a key
                // missing from the option's enum TYPE map
                // (`ConfigOptionEnum::from_string`, `Config.hpp:2032-2040`)
                // replaces the whole option with the definition default
                // under the EnableSilent 3MF load rule. Keys valid for the
                // type but outside the option's own `enum_values` whitelist
                // (e.g. `monotonic` on `sparse_infill_pattern`) pass through
                // for the `Slic3r::validate` loop to reject, like native.
                let type_keys = definition
                    .enum_type_keys
                    .or_else(|| option_metadata(key).map(|metadata| metadata.enum_values));
                if type_keys.is_none_or(|keys| keys.contains(&text.trim())) {
                    continue;
                }
                *value = RawConfigValue::Text(definition.default_value.to_owned());
            }
            OptionValueKind::Enums | OptionValueKind::EnumsNullable => {
                let wire = match value {
                    // An empty vector is a valid vector (no substitution).
                    RawConfigValue::Text(text) if !text.is_empty() => text.clone(),
                    RawConfigValue::List(items) if is_string_array(items) => {
                        match encode_array(definition.kind, items) {
                            Ok(wire) => wire,
                            Err(_) => continue,
                        }
                    }
                    _ => continue,
                };
                let Some(metadata) = option_metadata(key) else {
                    continue;
                };
                // `ConfigOptionVector` splits the wire on commas
                // (`Config.hpp:2156-2178`) and each entry goes through the
                // enum key map. `nil` is only legal on the nullable kinds —
                // on the non-nullable ones Native throws instead of
                // substituting, so the wire passes through untouched for
                // the typed builder to diagnose. `std::getline` emits no
                // trailing empty item for a final comma, so the last empty
                // segment is dropped before validation.
                let mut entries: Vec<&str> = wire.split(',').map(str::trim).collect();
                if entries.last() == Some(&"") {
                    entries.pop();
                }
                if entries.contains(&"nil") && definition.kind != OptionValueKind::EnumsNullable {
                    continue;
                }
                let type_keys = definition.enum_type_keys.unwrap_or(metadata.enum_values);
                let entries_valid = |entry: &str| type_keys.contains(&entry) || entry == "nil";
                if entries.iter().copied().all(entries_valid) {
                    continue;
                }
                *value = RawConfigValue::Text(definition.default_value.to_owned());
            }
            _ => {}
        }
    }
}

/// `ConfigHelpers::enum_looks_like_true_value` (`Config.hpp:96-99`).
fn looks_like_true(value: &str) -> bool {
    let value = value.trim();
    value.eq_ignore_ascii_case("enabled") || value.eq_ignore_ascii_case("on")
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
