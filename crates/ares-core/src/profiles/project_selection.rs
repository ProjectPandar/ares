mod collection;
mod config;
mod external;
mod loaded;
mod typed;
mod variants;

use super::project_refresh::preset::{dirty_keys, strings};
use crate::{
    ProfileKind, Project, SliceError,
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};
pub use collection::{PresetCollectionState, PresetOutcome, PresetRecord};
pub use config::PresetConfig;
pub use loaded::load_preset_bundle;
pub(crate) use loaded::{LoadedCatalog, loaded_catalog};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct PresetSelection {
    pub process: PresetCollectionState,
    pub printer: PresetCollectionState,
    pub filament: PresetCollectionState,
}

// PresetBundle.cpp:6158-6365: ordinary FFF metadata split and single-filament Always.
pub(super) fn refresh(
    catalog: &LoadedCatalog,
    project: &Project,
    filename: &str,
) -> Result<(ProjectSettingsRaw, PresetSelection), SliceError> {
    let raw = &project.documents().project_settings_raw;
    let filament_count = strings(raw, "filament_colour").len();
    if filament_count == 0 {
        return Err(SliceError::InvalidInput(
            "Always selection requires at least one filament".to_owned(),
        ));
    }
    if strings(raw, "printer_technology")
        .first()
        .is_some_and(|value| value != "FFF")
    {
        return Err(SliceError::InvalidInput(
            "Always selection requires an ordinary FFF project".to_owned(),
        ));
    }
    let inherited = strings(raw, "inherits_group");
    let machine_conditions = strings(raw, "compatible_machine_expression_group");
    let process_conditions = strings(raw, "compatible_process_expression_group");
    let differences = strings(raw, "different_settings_to_system");
    let filament_ids = strings(raw, "filament_ids");
    let print_compatible = strings(raw, "print_compatible_printers");
    let mut combined = raw.clone();
    if filament_count == 1 {
        combined.insert_strings("filament_self_index", &[]);
    }
    if strings(&combined, "filament_extruder_variant").is_empty() {
        combined.insert_strings(
            "filament_extruder_variant",
            &["Direct Drive Standard".to_owned()],
        );
    }
    let version = project
        .documents()
        .preset_file_version
        .as_deref()
        .unwrap_or("0.0.0.0");
    let mut load = |kind, slot: usize| {
        for (key, values, index) in [
            ("inherits", &inherited, slot),
            ("compatible_printers_condition", &machine_conditions, slot),
            (
                "compatible_prints_condition",
                &process_conditions,
                slot.saturating_sub(1),
            ),
        ] {
            combined.insert(
                key.to_owned(),
                RawConfigValue::Text(values.get(index).cloned().unwrap_or_default()),
            );
        }
        combined.insert_strings(
            "compatible_printers",
            if kind == ProfileKind::Process {
                &print_compatible
            } else {
                &[]
            },
        );
        let mut collection = catalog.collection(kind)?.clone();
        let mut protected = dirty_keys(&differences, slot)?;
        protected.extend(
            [
                "inherits",
                "print_settings_id",
                "filament_settings_id",
                "printer_settings_id",
            ]
            .map(str::to_owned),
        );
        let config = collection.materialize(&combined, &protected)?;
        let filament_id = if kind == ProfileKind::Filament {
            filament_ids.first().map_or("", String::as_str)
        } else {
            ""
        };
        collection.select(collection::SelectionArgs {
            config,
            filename,
            project_version: version,
            filament_id,
            select: collection::LoadAndSelect::Always,
        })
    };
    let process = load(ProfileKind::Process, 0)?;
    let printer = load(ProfileKind::Machine, filament_count + 1)?;
    let filament_states = if filament_count == 1 {
        let state = load(ProfileKind::Filament, 1)?;
        drop(load);
        vec![state]
    } else {
        drop(load);
        external::load_external_filaments(external::ExternalFilamentArgs {
            catalog,
            source: &combined,
            filament_ids: &filament_ids,
            inherited: &inherited,
            machine_conditions: &machine_conditions,
            process_conditions: &process_conditions,
            differences: &differences,
            filament_count,
            filename,
            version,
        })?
    };
    let selection = PresetSelection {
        process,
        printer,
        filament: filament_states[0].clone(),
    };
    let mut result = raw.clone();
    for state in [&selection.process, &selection.printer] {
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
    compose_filaments(&mut result, &filament_states);
    for (kind, state) in [
        (ProfileKind::Process, &selection.process),
        (ProfileKind::Machine, &selection.printer),
    ] {
        result.insert(
            collection::identity(kind).to_owned(),
            RawConfigValue::Text(state.selected.name.clone()),
        );
    }
    result.insert_strings(
        "filament_settings_id",
        &filament_states
            .iter()
            .map(|state| state.selected.name.clone())
            .collect::<Vec<_>>(),
    );
    Ok((result, selection))
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
