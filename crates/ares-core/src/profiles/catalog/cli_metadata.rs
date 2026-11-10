use crate::{
    MergedProfile,
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

// OrcaSlicer.cpp:3509-3517 admits this loop when the loaded-filament
// condition is true; :3715-3729 resizes each destination only when empty and
// writes the loaded filament's scalar value with set_at. `filament_index` is
// the upstream one-based slot: machine uses it directly and process uses
// `filament_index - 1`.
pub(super) fn filament_conditions(raw: &mut ProjectSettingsRaw, filaments: &[MergedProfile]) {
    let mut machine = existing_strings(raw, "compatible_machine_expression_group");
    let mut process = existing_strings(raw, "compatible_process_expression_group");
    for (index, profile) in filaments.iter().enumerate() {
        let metadata = match profile {
            MergedProfile::Machine { metadata, .. }
            | MergedProfile::Process { metadata, .. }
            | MergedProfile::Filament { metadata, .. } => metadata,
        };
        let filament_index = index + 1;
        if let Some(value) = metadata.compatible_printers_condition() {
            if machine.is_empty() {
                machine.resize(filaments.len() + 2, String::new());
            }
            set_at(&mut machine, filament_index, value);
        }
        if let Some(value) = metadata.compatible_prints_condition() {
            if process.is_empty() {
                process.resize(filaments.len(), String::new());
            }
            set_at(&mut process, filament_index - 1, value);
        }
    }
    if !machine.is_empty() {
        raw.insert_strings("compatible_machine_expression_group", &machine);
    }
    if !process.is_empty() {
        raw.insert_strings("compatible_process_expression_group", &process);
    }
}

fn set_at(values: &mut Vec<String>, index: usize, value: &str) {
    if values.len() <= index {
        let first = values.first().cloned().unwrap();
        values.resize(index + 1, first);
    }
    values[index] = value.to_owned();
}

fn existing_strings(raw: &ProjectSettingsRaw, key: &str) -> Vec<String> {
    match raw.get(key) {
        Some(RawConfigValue::List(values)) => values
            .iter()
            .map(|value| match value {
                RawConfigValue::Text(text) => text.clone(),
                _ => String::new(),
            })
            .collect(),
        Some(RawConfigValue::Text(value)) => vec![value.clone()],
        _ => Vec::new(),
    }
}
