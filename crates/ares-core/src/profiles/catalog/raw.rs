use crate::{
    OptionValueKind, ProfileKind, option_definition,
    options::{EXPLICIT_RULES, OBSOLETE_INPUTS},
};

// Typed config export owns canonical tokens; this raw view only retains input
// presence, nullable slots, and dynamic options not owned by a static group.

// OrcaSlicer.cpp:2800-2804 update_full_config exclusions. This is the CLI
// copy rule, not PresetBundle::full_fff_config's GUI metadata erasure.
pub(super) fn applies(kind: ProfileKind, key: &str) -> bool {
    if [
        "compatible_prints",
        "compatible_printers",
        "model_id",
        "inherits",
        "dev_model_name",
        "name",
        "from",
        "type",
        "version",
        "setting_id",
        "instantiation",
    ]
    .contains(&key)
    {
        return false;
    }
    if OBSOLETE_INPUTS.contains(&key) {
        return false;
    }
    // Legacy rule sources have no option definition, but the typed
    // resolution consumes them from the raw view (`handle_legacy` runs per
    // loaded key upstream, PrintConfig.cpp:8033); admitting them keeps
    // alias/new coexistence resolvable downstream.
    if EXPLICIT_RULES.iter().any(|rule| rule.source == key) {
        return true;
    }
    let Some(definition) = option_definition(key) else {
        return false;
    };
    if kind != ProfileKind::Filament {
        return true;
    }
    // OrcaSlicer.cpp:3257-3290 routes filament scalar conditions separately.
    match definition.kind {
        OptionValueKind::Float
        | OptionValueKind::FloatOrPercent
        | OptionValueKind::Percent
        | OptionValueKind::Int
        | OptionValueKind::Bool
        | OptionValueKind::Enum
        | OptionValueKind::String
        | OptionValueKind::Point => false,
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
        | OptionValueKind::PointsGroups => key != "filament_settings_id",
    }
}
