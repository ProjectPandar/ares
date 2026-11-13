mod collection;
mod config;
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
        let collection = collection::Collection::load(catalog, kind)?;
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
        collection.select(config, filename, version, filament_id)
    };
    let process = load(ProfileKind::Process, 0)?;
    let printer = load(ProfileKind::Machine, filament_count + 1)?;
    let filament_states = if filament_count == 1 {
        let state = load(ProfileKind::Filament, 1)?;
        drop(load);
        vec![state]
    } else {
        drop(load);
        load_external_filaments(
            catalog,
            &combined,
            &filament_ids,
            &inherited,
            &machine_conditions,
            &process_conditions,
            &differences,
            filament_count,
            filename,
            version,
        )?
    };
    let selection = PresetSelection {
        process,
        printer,
        filament: filament_states[0].clone(),
    };
    let mut result = raw.clone();
    for state in [&selection.process, &selection.printer]
        .into_iter()
        .chain(filament_states.iter())
    {
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

fn load_external_filaments(
    catalog: &Catalog,
    source: &ProjectSettingsRaw,
    filament_ids: &[String],
    inherited: &[String],
    machine_conditions: &[String],
    process_conditions: &[String],
    differences: &[String],
    filament_count: usize,
    filename: &str,
    version: &str,
) -> Result<Vec<PresetCollectionState>, SliceError> {
    let variant_values = strings(source, "filament_extruder_variant");
    let self_indices = strings(source, "filament_self_index")
        .into_iter()
        .map(|value| value.parse::<usize>().unwrap_or_default())
        .collect::<Vec<_>>();
    let process_multi_extruder =
        source.contains_key("extruder_variant_list") && variant_values.len() > filament_count;
    let extruder_variant_count = variant_values.len().max(filament_count);
    let variant_indices = (0..filament_count)
        .map(|filament| {
            self_indices
                .iter()
                .position(|index| *index == filament + 1)
                .unwrap_or(filament)
        })
        .collect::<Vec<_>>();
    let variant_keys = crate::options::registry::filament_options_with_variant();
    let defaults = PresetConfig::defaults(ProfileKind::Filament)?.0;
    let mut configs = Vec::with_capacity(filament_count);
    for filament in 0..filament_count {
        let mut config = PresetConfig(defaults.clone());
        for (key, default) in defaults.entries() {
            let Some(value) = source.get(key) else {
                continue;
            };
            match value {
                RawConfigValue::List(values)
                    if key != "compatible_printers" && key != "compatible_prints" =>
                {
                    let values = if process_multi_extruder
                        && (key == "filament_extruder_variant"
                            || variant_keys.contains(&key.as_str()))
                    {
                        let mut values = values.clone();
                        values.resize(
                            extruder_variant_count,
                            values.first().cloned().unwrap_or_else(|| {
                                default_list_value(default)
                                    .unwrap_or_else(|| RawConfigValue::Text("0".to_owned()))
                            }),
                        );
                        let start = variant_indices[filament].min(values.len());
                        let end = variant_indices
                            .get(filament + 1)
                            .copied()
                            .unwrap_or(extruder_variant_count)
                            .min(values.len());
                        RawConfigValue::List(values[start..end].to_vec())
                    } else {
                        values.get(filament).cloned().map_or_else(
                            || RawConfigValue::List(Vec::new()),
                            |value| RawConfigValue::List(vec![value]),
                        )
                    };
                    config.0.insert(key.clone(), values);
                }
                RawConfigValue::List(_) => {}
                value => {
                    config.0.insert(key.clone(), value.clone());
                }
            }
        }
        config.set_string(
            "compatible_printers_condition",
            machine_conditions
                .get(filament + 1)
                .cloned()
                .unwrap_or_default(),
        );
        config.set_string(
            "compatible_prints_condition",
            process_conditions
                .get(filament)
                .cloned()
                .unwrap_or_default(),
        );
        let mut parent = inherited.get(filament + 1).cloned().unwrap_or_default();
        if !parent.is_empty() {
            parent = convert_filament_preset_name(source, parent);
        }
        config.set_string("inherits", parent.clone());
        config.set_string(
            "different_settings_to_system",
            differences.get(filament + 1).cloned().unwrap_or_default(),
        );
        let mut name = strings(source, "filament_settings_id")
            .get(filament)
            .cloned()
            .unwrap_or_default();
        if parent.is_empty() {
            name = convert_filament_preset_name(source, name);
        }
        config.set_string("filament_settings_id", name);
        config.set_string(
            "filament_self_index",
            self_indices
                .get(variant_indices[filament])
                .copied()
                .unwrap_or(filament + 1)
                .to_string(),
        );
        configs.push(config);
    }

    let collection = || collection::Collection::load(catalog, ProfileKind::Filament);
    let mut states = Vec::with_capacity(filament_count);
    for filament in (0..filament_count).rev() {
        let collection = collection()?;
        let mut protected = dirty_keys(differences, filament + 1)?;
        protected.extend(
            [
                "inherits",
                "print_settings_id",
                "filament_settings_id",
                "printer_settings_id",
            ]
            .map(str::to_owned),
        );
        let config = collection.materialize(&configs[filament].0, &protected)?;
        states.push(collection.select(
            config,
            filename,
            version,
            filament_ids.get(filament).map(String::as_str).unwrap_or(""),
        )?);
    }
    states.reverse();
    Ok(states)
}

fn default_list_value(value: &RawConfigValue) -> Option<RawConfigValue> {
    match value {
        RawConfigValue::List(values) => values.first().cloned(),
        _ => None,
    }
}

fn convert_filament_preset_name(source: &ProjectSettingsRaw, mut name: String) -> String {
    let machine = strings(source, "printer_settings_id")
        .into_iter()
        .next()
        .unwrap_or_default();
    let replacement = match machine.as_str() {
        "Bambu Lab H2D 0.6 nozzle" => match name.as_str() {
            "Bambu PLA Basic @BBL H2D" => Some("Bambu PLA Basic @BBL H2D 0.6 nozzle"),
            "Bambu PLA Matte @BBL H2D" => Some("Bambu PLA Matte @BBL H2D 0.6 nozzle"),
            "Bambu ABS @BBL H2D" => Some("Bambu ABS @BBL H2D 0.6 nozzle"),
            _ => None,
        },
        "Bambu Lab H2D 0.8 nozzle" => match name.as_str() {
            "Bambu PETG HF @BBL H2D 0.6 nozzle" => Some("Bambu PETG HF @BBL H2D 0.8 nozzle"),
            "Bambu ASA @BBL H2D 0.6 nozzle" => Some("Bambu ASA @BBL H2D 0.8 nozzle"),
            _ => None,
        },
        _ => None,
    };
    if let Some(replacement) = replacement {
        name = replacement.to_owned();
    }
    name
}
