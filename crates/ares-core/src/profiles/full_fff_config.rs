use super::{PresetCollectionState, PresetRecord};
use crate::project::raw_settings::{ProjectSettingsRaw, RawConfigValue};

/// The selected process, printer and filament presets applied onto `result`.
pub(super) fn full_fff_config(
    result: &mut ProjectSettingsRaw,
    process: &PresetCollectionState,
    printer: &PresetCollectionState,
    filaments: &[PresetCollectionState],
) {
    // full_fff_config erases it (PresetBundle.cpp:5790-5794).
    result.remove("compatible_printers_condition");
    for state in [process, printer] {
        for (key, value) in state.edited.config.0.entries() {
            if matches!(
                key.as_str(),
                "inherits"
                    | "compatible_printers"
                    | "compatible_prints"
                    | "compatible_printers_condition"
                    | "compatible_prints_condition"
                    | "filament_self_index"
                    | "filament_extruder_variant"
                    | "filament_map"
            ) {
                continue;
            }
            result.insert(key.clone(), value.clone());
        }
    }
    compose_filaments(result, filaments);
}

// PresetBundle::full_fff_config(false) (PresetBundle.cpp:5621-5774), the project config the
// plater saves (full_config_secure, Plater.cpp:19779): each filament is find_preset(name, true),
// the edited copy for the selected preset; scalars come from the first filament, vectors are
// appended in filament order, filament_self_index numbers each filament's variants, and
// filament_ids are the presets' filament_id (5636, 5679, 5821).
fn compose_filaments(result: &mut ProjectSettingsRaw, states: &[PresetCollectionState]) {
    let selected = &states[0];
    let presets = states
        .iter()
        .map(|state| {
            if state.selected.name == selected.selected.name {
                &selected.edited
            } else {
                selected
                    .records
                    .iter()
                    .find(|record| record.name == state.selected.name)
                    .expect("loaded filament preset")
            }
        })
        .collect::<Vec<_>>();
    let list = |preset: &PresetRecord, key: &str| match preset.config.0.get(key) {
        Some(RawConfigValue::List(values)) => values.clone(),
        _ => Vec::new(),
    };
    for (key, value) in presets[0].config.0.entries() {
        if matches!(
            key.as_str(),
            "inherits"
                | "compatible_printers"
                | "compatible_prints"
                | "compatible_printers_condition"
                | "compatible_prints_condition"
                | "different_settings_to_system"
                | "filament_settings_id"
                | "filament_self_index"
                | "filament_map"
        ) {
            continue;
        }
        let value = match value {
            RawConfigValue::List(_) => RawConfigValue::List(
                presets
                    .iter()
                    .flat_map(|preset| list(preset, key))
                    .collect(),
            ),
            value => value.clone(),
        };
        result.insert(key.clone(), value);
    }
    result.insert_strings(
        "filament_self_index",
        &presets
            .iter()
            .enumerate()
            .flat_map(|(index, preset)| {
                vec![(index + 1).to_string(); list(preset, "filament_extruder_variant").len()]
            })
            .collect::<Vec<_>>(),
    );
    result.insert_strings(
        "filament_ids",
        &presets
            .iter()
            .map(|preset| preset.filament_id.clone())
            .collect::<Vec<_>>(),
    );
}
