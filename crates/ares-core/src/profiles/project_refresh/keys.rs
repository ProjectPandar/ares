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

pub(crate) fn default_value(key: &str) -> Result<Option<RawConfigValue>, SliceError> {
    // PresetCollection's default preset applies only definitions present in PrintConfigDef.
    let Some(definition) = option_definition(key) else {
        return Ok(None);
    };
    // PresetBundle.cpp:508-518 nils every nullable filament override key of the default filament preset.
    let wire = if definition.default_is_nil
        || (is_filament_extruder_override_key(key)
            && matches!(
                definition.kind,
                OptionValueKind::BoolsNullable
                    | OptionValueKind::EnumsNullable
                    | OptionValueKind::FloatsNullable
                    | OptionValueKind::IntsNullable
                    | OptionValueKind::PercentsNullable
            )) {
        "nil"
    } else {
        definition.default_wire_value
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

/// `is_filament_extruder_override_key` (`PrintConfig.cpp:130-139`) over
/// `filament_extruder_override_keys` (`PrintConfig.cpp:97-122`).
fn is_filament_extruder_override_key(key: &str) -> bool {
    matches!(
        key,
        "filament_retraction_length"
            | "filament_z_hop"
            | "filament_z_hop_types"
            | "filament_retract_lift_above"
            | "filament_retract_lift_below"
            | "filament_retract_lift_enforce"
            | "filament_retraction_speed"
            | "filament_deretraction_speed"
            | "filament_retract_restart_extra"
            | "filament_retraction_minimum_travel"
            | "filament_retract_length_toolchange"
            | "filament_retract_restart_extra_toolchange"
            | "filament_wipe_distance"
            | "filament_retract_when_changing_layer"
            | "filament_wipe"
            | "filament_retract_before_wipe"
            | "filament_retract_after_wipe"
            | "filament_long_retractions_when_cut"
            | "filament_retraction_distances_when_cut"
            | "filament_retract_length_nc"
            | "filament_ironing_flow"
            | "filament_ironing_spacing"
            | "filament_ironing_inset"
            | "filament_ironing_speed"
    )
}
