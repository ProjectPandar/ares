use crate::{MergedProfile, project::raw_settings::ProjectSettingsRaw};

// OrcaSlicer.cpp:3257-3271: filament scalar conditions do not replace the
// process scalar. They occupy indexed CLI groups, including reserved slots.
pub(super) fn filament_conditions(raw: &mut ProjectSettingsRaw, filaments: &[MergedProfile]) {
    let mut machine = vec![None; filaments.len() + 2];
    let mut process = vec![None; filaments.len()];
    for (index, profile) in filaments.iter().enumerate() {
        let metadata = match profile {
            MergedProfile::Machine { metadata, .. }
            | MergedProfile::Process { metadata, .. }
            | MergedProfile::Filament { metadata, .. } => metadata,
        };
        machine[index + 1] = metadata.compatible_printers_condition();
        process[index] = metadata.compatible_prints_condition();
    }
    for (key, values) in [
        ("compatible_machine_expression_group", machine),
        ("compatible_process_expression_group", process),
    ] {
        if values.iter().any(Option::is_some) {
            let values: Vec<_> = values
                .into_iter()
                .map(|value| value.unwrap_or_default().to_owned())
                .collect();
            raw.insert_strings(key, &values);
        }
    }
}
