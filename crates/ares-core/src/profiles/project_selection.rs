mod collection;
mod config;
mod external;
mod typed;
mod variants;

use super::{
    catalog::Catalog,
    project_refresh::preset::{dirty_keys, strings},
};
use crate::{
    ProfileKind, Project, SliceError,
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};
pub use collection::{PresetCollectionState, PresetOutcome, PresetRecord};
pub use config::PresetConfig;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize)]
pub struct PresetSelection {
    pub process: PresetCollectionState,
    pub printer: PresetCollectionState,
    pub filament: PresetCollectionState,
}

// PresetBundle.cpp:6158-6365: ordinary FFF metadata split and single-filament Always.
pub(super) fn refresh(
    catalog: &Catalog,
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
        let mut collection = collection::Collection::load(catalog, kind)?;
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
    let mut filament_values = BTreeMap::new();
    for state in &filament_states {
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
            if matches!(value, RawConfigValue::List(_)) {
                continue;
            }
            filament_values.insert(key.clone(), value.clone());
        }
    }
    for (key, value) in filament_values {
        result.insert(key, value);
    }
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
