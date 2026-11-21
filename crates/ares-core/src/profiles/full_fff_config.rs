use super::{
    PresetCollectionState, PresetRecord,
    project_selection::{
        FullFffGroups, Selected, dirty_options, filament_diff, filament_parent, selected_parent,
    },
};
use crate::{
    SliceError,
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

/// The selected process, printer and filament presets applied onto `result`.
pub(super) fn full_fff_config(
    result: &mut ProjectSettingsRaw,
    process: &PresetCollectionState,
    printer: &PresetCollectionState,
    filaments: &[PresetCollectionState],
) -> Result<(), SliceError> {
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
    let presets = filament_presets(filaments);
    compose_filaments(result, &presets);
    groups(process, printer, filaments, &presets)?.apply(result);
    Ok(())
}

/// The groups of the selected states (PresetBundle.cpp:5571-5595, 5628-5712, 5778-5788).
fn groups(
    process: &PresetCollectionState,
    printer: &PresetCollectionState,
    filaments: &[PresetCollectionState],
    presets: &[&PresetRecord],
) -> Result<FullFffGroups, SliceError> {
    let selected = |state: &PresetCollectionState| {
        let parent = selected_parent(&state.selected, &state.edited, &state.records, |name| {
            state.records.iter().find(|record| record.name == name)
        });
        dirty_options(parent.map(|parent| &parent.config), &state.edited.config)
    };
    let first = &filaments[0];
    let filament_groups = if let [only] = filaments {
        vec![Selected {
            preset: &only.edited,
            different: selected(only)?,
        }]
    } else {
        presets
            .iter()
            .map(|preset| {
                let parent = filament_parent(preset, &first.selected, |name| {
                    first.records.iter().find(|record| record.name == name)
                });
                Ok(Selected {
                    preset,
                    different: filament_diff(parent.map(|parent| &parent.config), &preset.config)?,
                })
            })
            .collect::<Result<Vec<_>, SliceError>>()?
    };
    Ok(FullFffGroups::collect(
        &Selected {
            preset: &process.edited,
            different: selected(process)?,
        },
        &filament_groups,
        &Selected {
            preset: &printer.edited,
            different: selected(printer)?,
        },
    ))
}

/// Each filament as find_preset(name, true) returns it: the edited copy for the selected preset.
fn filament_presets(states: &[PresetCollectionState]) -> Vec<&PresetRecord> {
    let selected = &states[0];
    states
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
        .collect()
}

// PresetBundle::full_fff_config(false) (PresetBundle.cpp:5621-5774), the project config the
// plater saves (full_config_secure, Plater.cpp:19779): each filament is find_preset(name, true),
// the edited copy for the selected preset; scalars come from the first filament, vectors are
// appended in filament order, filament_self_index numbers each filament's variants, and
// filament_ids are the presets' filament_id (5636, 5679, 5821).
fn compose_filaments(result: &mut ProjectSettingsRaw, presets: &[&PresetRecord]) {
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
