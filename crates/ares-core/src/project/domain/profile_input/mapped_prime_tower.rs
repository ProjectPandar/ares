use crate::{
    ProjectSettings,
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

pub(super) fn disable_after_mapping(
    raw: &mut ProjectSettingsRaw,
    settings: &ProjectSettings,
    overrides: &crate::ConfigOverrides,
    filament_count: usize,
) {
    let single_preset_mapping = settings
        .project
        .preset
        .filament_settings_id
        .0
        .first()
        .is_some_and(|first| {
            settings
                .project
                .preset
                .filament_settings_id
                .0
                .iter()
                .all(|name| name == first)
        });
    let enable_timelapse = match raw.get("enable_timelapse") {
        Some(RawConfigValue::Text(value)) => value == "1",
        Some(RawConfigValue::Flag(value)) => *value,
        _ => false,
    };
    let smooth_timelapse = enable_timelapse
        && matches!(raw.get("timelapse_type"), Some(RawConfigValue::Text(value)) if value == "1");
    if !single_preset_mapping || smooth_timelapse {
        return;
    }

    let distinct_cli_colours = overrides.tokens("filament_colour").is_some_and(|colours| {
        colours
            .iter()
            .filter(|colour| !colour.is_empty())
            .any(|colour| {
                colours
                    .iter()
                    .filter(|candidate| !candidate.is_empty())
                    .any(|candidate| candidate != colour)
            })
    });
    if distinct_cli_colours {
        return;
    }

    // OrcaSlicer.cpp:4323-4342 runs after CLI override bookkeeping (:4138-4221).
    raw.insert(
        "enable_prime_tower".to_owned(),
        RawConfigValue::Text("0".to_owned()),
    );
    // full_fff_config stores no all-empty group (PresetBundle.cpp:5823-5841); the CLI resize
    // (OrcaSlicer.cpp:3202-3205) creates its process, filament and printer columns.
    let mut columns = match raw.get("different_settings_to_system") {
        Some(RawConfigValue::List(columns)) => columns.clone(),
        Some(_) => unreachable!("profile overlay constructs vector dirty columns"),
        None => vec![RawConfigValue::Text(String::new()); filament_count + 2],
    };
    let column = columns[0].token().unwrap();
    let keys = crate::options::unescape_strings_cstyle(&column).unwrap();
    if !keys.iter().any(|key| key == "enable_prime_tower") {
        columns[0] = RawConfigValue::Text(if column.is_empty() {
            "enable_prime_tower".to_owned()
        } else {
            format!("{column};enable_prime_tower")
        });
        raw.insert(
            "different_settings_to_system".to_owned(),
            RawConfigValue::List(columns),
        );
    }
}
