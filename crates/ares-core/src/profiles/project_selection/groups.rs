//! The group vectors `PresetBundle::full_fff_config` collects over the selected presets
//! (PresetBundle.cpp:5571-5595, 5628-5647, 5673-5712, 5778-5788) and stores only when some entry is
//! non-empty (5823-5841), with the keys it erases (5791-5797).

use super::{PresetRecord, config::PresetConfig};
use crate::{SliceError, project::raw_settings::ProjectSettingsRaw};

/// PresetBundle.cpp:5515-5518 `ignore_settings_list`.
const IGNORED: [&str; 4] = [
    "inherits",
    "print_settings_id",
    "filament_settings_id",
    "printer_settings_id",
];

/// PresetBundle.cpp:5791-5797: the keys the print and filament presets clash on, and
/// `different_settings_to_system`, which the collected group replaces.
const ERASED: [&str; 6] = [
    "compatible_prints",
    "compatible_prints_condition",
    "compatible_printers",
    "compatible_printers_condition",
    "inherits",
    "different_settings_to_system",
];

/// One preset full_fff_config reads, with its `different_settings_to_system` entry.
pub(in crate::profiles) struct Selected<'a> {
    pub(in crate::profiles) preset: &'a PresetRecord,
    pub(in crate::profiles) different: String,
}

/// The collected groups; `apply` writes them as full_fff_config's result holds them.
#[derive(Debug)]
pub(crate) struct FullFffGroups([(&'static str, Vec<String>); 5]);

impl FullFffGroups {
    pub(in crate::profiles) fn collect(
        process: &Selected<'_>,
        filaments: &[Selected<'_>],
        printer: &Selected<'_>,
    ) -> Self {
        let text = |preset: &PresetRecord, key: &str| preset.config.string(key);
        let mut machine_conditions = vec![text(process.preset, "compatible_printers_condition")];
        let mut process_conditions = Vec::with_capacity(filaments.len());
        let mut inherits = vec![text(process.preset, "inherits")];
        let mut different = vec![process.different.clone()];
        for filament in filaments {
            machine_conditions.push(text(filament.preset, "compatible_printers_condition"));
            process_conditions.push(text(filament.preset, "compatible_prints_condition"));
            inherits.push(text(filament.preset, "inherits"));
            different.push(filament.different.clone());
        }
        // Printers have no compatible_printers_condition entry (5778-5780).
        inherits.push(text(printer.preset, "inherits"));
        different.push(printer.different.clone());
        let print_compatible = super::strings(&process.preset.config.0, "compatible_printers");
        Self([
            ("compatible_machine_expression_group", machine_conditions),
            ("compatible_process_expression_group", process_conditions),
            ("inherits_group", inherits),
            ("different_settings_to_system", different),
            ("print_compatible_printers", print_compatible),
        ])
    }

    pub(crate) fn apply(&self, raw: &mut ProjectSettingsRaw) {
        for key in ERASED {
            raw.remove(key);
        }
        for (key, values) in &self.0 {
            raw.remove(key);
            if values.iter().any(|value| !value.is_empty()) {
                raw.insert_strings(key, values);
            }
        }
    }
}

/// `PresetCollection::get_selected_preset_parent` (Preset.cpp:3770-3793, 3815-3824): the
/// selected preset itself when it is its own base, else the preset the edited copy inherits, else
/// the default preset unless the selected one is external.
pub(in crate::profiles) fn selected_parent<'a>(
    selected: &'a PresetRecord,
    edited: &PresetRecord,
    records: &'a [PresetRecord],
    find: impl Fn(&str) -> Option<&'a PresetRecord>,
) -> Option<&'a PresetRecord> {
    if selected.system || selected.default || selected.config.string("inherits").is_empty() {
        return Some(selected);
    }
    match edited.config.string("inherits").as_str() {
        "" if selected.external => None,
        "" => records.first(),
        inherits => find(inherits),
    }
}

/// The parent of a filament slot when several are selected (PresetBundle.cpp:5684-5694): a system
/// or default preset is its own, read unedited when it is the selected one; another preset's is
/// the one it inherits.
pub(in crate::profiles) fn filament_parent<'a>(
    preset: &'a PresetRecord,
    selected: &'a PresetRecord,
    find: impl Fn(&str) -> Option<&'a PresetRecord>,
) -> Option<&'a PresetRecord> {
    if preset.system || preset.default {
        return Some(if preset.name == selected.name {
            selected
        } else {
            preset
        });
    }
    match preset.config.string("inherits").as_str() {
        "" => None,
        inherits => find(inherits),
    }
}

/// `dirty_options_without_option_list(edited, parent, ignore_settings_list)` as one
/// `different_settings_to_system` entry (Preset.cpp:4242-4268, PresetBundle.cpp:5590-5594).
pub(in crate::profiles) fn dirty_options(
    parent: Option<&PresetConfig>,
    edited: &PresetConfig,
) -> Result<String, SliceError> {
    let Some(parent) = parent else {
        return Ok(String::new());
    };
    Ok(entry(parent.dirty(edited)?.1))
}

/// A filament slot's entry when several are selected: `cfg.diff(parent)` without the
/// compatible-key presence check (PresetBundle.cpp:5692-5708).
pub(in crate::profiles) fn filament_diff(
    parent: Option<&PresetConfig>,
    preset: &PresetConfig,
) -> Result<String, SliceError> {
    let Some(parent) = parent else {
        return Ok(String::new());
    };
    Ok(entry(preset.diff(parent)?))
}

fn entry(mut keys: Vec<String>) -> String {
    keys.retain(|key| !IGNORED.contains(&key.as_str()));
    keys.iter()
        .map(|key| crate::options::escape_string_vector_element(key, keys.len()))
        .collect::<Vec<_>>()
        .join(";")
}
