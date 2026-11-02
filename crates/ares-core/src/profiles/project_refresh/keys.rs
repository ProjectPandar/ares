mod filament;
mod machine;
mod process_first;
mod process_last;

use crate::project::raw_settings::RawConfigValue;
use crate::{OptionValueKind, ProfileKind, SliceError, option_definition};

pub(super) fn names(kind: ProfileKind) -> impl Iterator<Item = &'static str> {
    let (first, last) = match kind {
        ProfileKind::Process => (process_first::KEYS, process_last::KEYS),
        ProfileKind::Machine => (machine::KEYS, &[][..]),
        ProfileKind::Filament => (filament::KEYS, &[][..]),
    };
    first.iter().chain(last).copied()
}

pub(super) fn default_value(key: &str) -> Result<Option<RawConfigValue>, SliceError> {
    // PresetCollection's default preset applies only definitions present in PrintConfigDef.
    let Some(definition) = option_definition(key) else {
        return Ok(None);
    };
    let wire = if definition.default_is_nil {
        "nil"
    } else if definition.full_config_default {
        definition.default_wire_value
    } else {
        definition.default_value
    };
    use OptionValueKind as K;
    let value = match definition.kind {
        K::Strings => RawConfigValue::List(
            crate::options::unescape_strings_cstyle(wire)
                .map_err(SliceError::InvalidInput)?
                .into_iter()
                .map(RawConfigValue::Text)
                .collect(),
        ),
        K::Percents
        | K::PercentsNullable
        | K::Bools
        | K::BoolsNullable
        | K::Enums
        | K::EnumsNullable
        | K::Floats
        | K::FloatsNullable
        | K::FloatsOrPercents
        | K::IntsNullable
        | K::Ints
        | K::Points => RawConfigValue::List(if wire.is_empty() {
            Vec::new()
        } else {
            wire.split(',')
                .map(|token| RawConfigValue::Text(token.to_owned()))
                .collect()
        }),
        K::PointsGroups => RawConfigValue::List(if wire.is_empty() {
            Vec::new()
        } else {
            wire.split(';')
                .map(|group| {
                    RawConfigValue::List(
                        group
                            .split(',')
                            .map(|token| RawConfigValue::Text(token.to_owned()))
                            .collect(),
                    )
                })
                .collect()
        }),
        K::String => serde_json::from_str(
            &crate::decode_config_value_by_key(key, wire).map_err(SliceError::InvalidInput)?,
        )
        .map_err(|error| SliceError::InvalidInput(error.to_string()))?,
        K::Float | K::FloatOrPercent | K::Percent | K::Int | K::Bool | K::Enum | K::Point => {
            RawConfigValue::Text(wire.to_owned())
        }
    };
    Ok(Some(value))
}
