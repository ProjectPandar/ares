//! OrcaSlicer.cpp:4157–4223: changed CLI values and Preset-owned dirty columns.
use super::is_scalar;
use crate::{
    ProfileKind, SliceError,
    options::{
        OrcaStrings,
        config_export::value::serialize_config_value,
        decode_config_value, encode_config_value,
        model_config_deserialize::{WireValue, decode_wire},
        registry::{OptionValueKind, option_definition},
        unescape_strings_cstyle,
    },
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

pub(super) fn same_value(
    key: &str,
    before: Option<&RawConfigValue>,
    after: &RawConfigValue,
) -> Result<bool, SliceError> {
    let Some(before) = before else {
        return Ok(false);
    };
    let before = serialized(key, before).map_err(SliceError::InvalidInput)?;
    let after = serialized(key, after).map_err(SliceError::InvalidInput)?;
    Ok(match (before, after) {
        (WireValue::Scalar(a), WireValue::Scalar(b)) => a == b,
        (WireValue::Sequence(a), WireValue::Sequence(b)) => {
            if a.is_empty() || b.is_empty() {
                a.is_empty() && b.is_empty()
            } else {
                (0..a.len().max(b.len()))
                    .all(|i| a.get(i).unwrap_or(&a[0]) == b.get(i).unwrap_or(&b[0]))
            }
        }
        _ => false,
    })
}

fn serialized(key: &str, value: &RawConfigValue) -> Result<WireValue, String> {
    let kind = option_definition(key).unwrap().kind;
    let wire = match value {
        RawConfigValue::List(items) if !is_scalar(kind) => {
            WireValue::Sequence(items.iter().map(text).collect::<Result<_, _>>()?)
        }
        value if is_scalar(kind) => WireValue::Scalar(text(value)?),
        value => decode_wire(kind, &text(value)?)?,
    };
    Ok(match wire {
        WireValue::Scalar(token) => WireValue::Scalar(if kind == OptionValueKind::String {
            token
        } else {
            canonical(kind, &token)?
        }),
        WireValue::Sequence(tokens) => WireValue::Sequence(
            tokens
                .into_iter()
                .map(|token| {
                    if kind == OptionValueKind::Strings || token == "nil" {
                        Ok(token)
                    } else {
                        canonical(kind, &token)
                    }
                })
                .collect::<Result<_, _>>()?,
        ),
    })
}

fn canonical(kind: OptionValueKind, token: &str) -> Result<String, String> {
    encode_config_value(kind, &decode_config_value(kind, token)?)
}

fn text(value: &RawConfigValue) -> Result<String, String> {
    match value {
        RawConfigValue::Text(value) => Ok(value.clone()),
        RawConfigValue::Number(value) => Ok(value.to_string()),
        RawConfigValue::Flag(value) => Ok(u8::from(*value).to_string()),
        RawConfigValue::Null => Ok("nil".to_owned()),
        RawConfigValue::List(values) => values
            .iter()
            .map(text)
            .collect::<Result<Vec<_>, _>>()
            .map(|values| values.join(",")),
        RawConfigValue::Object(_) => Err("invalid override comparison value".to_owned()),
    }
}

pub(crate) fn record_changed(
    raw: &mut ProjectSettingsRaw,
    changed: &[&str],
) -> Result<(), SliceError> {
    if changed.is_empty() {
        return Ok(());
    }
    let count = raw
        .get("filament_settings_id")
        .map_or(0, RawConfigValue::len);
    let mut columns = match raw.get("different_settings_to_system") {
        Some(RawConfigValue::List(values)) => {
            values.iter().map(text).collect::<Result<Vec<_>, _>>()
        }
        Some(value) => text(value).and_then(|value| unescape_strings_cstyle(&value)),
        None => Ok(Vec::new()),
    }
    .map_err(SliceError::InvalidInput)?;
    columns.resize(count + 2, String::new());
    for &key in changed {
        for kind in [
            ProfileKind::Process,
            ProfileKind::Filament,
            ProfileKind::Machine,
        ] {
            if !crate::profiles::preset_owns_option(kind, key) {
                continue;
            }
            let range = match kind {
                ProfileKind::Process => 0..1,
                ProfileKind::Filament => 1..columns.len() - 1,
                ProfileKind::Machine => columns.len() - 1..columns.len(),
            };
            for column in &mut columns[range] {
                add_to_column(column, key)?;
            }
        }
    }
    raw.insert_strings("different_settings_to_system", &columns);
    Ok(())
}

fn add_to_column(column: &mut String, key: &str) -> Result<(), SliceError> {
    let mut keys = unescape_strings_cstyle(column).map_err(SliceError::InvalidInput)?;
    if !keys.iter().any(|existing| existing == key) {
        keys.push(key.to_owned());
        *column = serialize_config_value(&OrcaStrings(keys))
            .map_err(|error| SliceError::InvalidInput(error.to_string()))?
            .token;
    }
    Ok(())
}
