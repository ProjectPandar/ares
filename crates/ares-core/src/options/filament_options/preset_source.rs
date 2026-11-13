use serde::{Serialize, Serializer, ser::SerializeMap};

use super::super::{OrcaString, OrcaStrings, option_group::declare_option_group};

declare_option_group! {
    pub struct FilamentPresetSourceOptions, FilamentPresetSourceOptionsBuilder {
        filament_plugin_config_overrides => "filament_plugin_config_overrides": OrcaString = OrcaString(String::new()),
        compatible_printers => "compatible_printers": OrcaStrings = OrcaStrings(Vec::new()),
        compatible_printers_condition => "compatible_printers_condition": OrcaString = OrcaString(String::new()),
        compatible_prints => "compatible_prints": OrcaStrings = OrcaStrings(Vec::new()),
        compatible_prints_condition => "compatible_prints_condition": OrcaString = OrcaString(String::new()),
    }
}

impl Default for FilamentPresetSourceOptions {
    fn default() -> Self {
        FilamentPresetSourceOptionsBuilder::default().resolve()
    }
}

impl FilamentPresetSourceOptions {
    pub(crate) fn restore_absent_defaults(
        &mut self,
        defaults: Self,
        present: &impl Fn(&str) -> bool,
    ) {
        if !present("filament_plugin_config_overrides") {
            self.filament_plugin_config_overrides = defaults.filament_plugin_config_overrides;
        }
        if !present("compatible_printers") {
            self.compatible_printers = defaults.compatible_printers;
        }
        if !present("compatible_printers_condition") {
            self.compatible_printers_condition = defaults.compatible_printers_condition;
        }
        if !present("compatible_prints") {
            self.compatible_prints = defaults.compatible_prints;
        }
        if !present("compatible_prints_condition") {
            self.compatible_prints_condition = defaults.compatible_prints_condition;
        }
    }
}

impl FilamentPresetSourceOptionsBuilder {
    pub(crate) fn apply_present(self, target: &mut FilamentPresetSourceOptions) {
        if let Some(value) = self.filament_plugin_config_overrides {
            target.filament_plugin_config_overrides = value;
        }
        if let Some(value) = self.compatible_printers {
            target.compatible_printers = value;
        }
        if let Some(value) = self.compatible_printers_condition {
            target.compatible_printers_condition = value;
        }
        if let Some(value) = self.compatible_prints {
            target.compatible_prints = value;
        }
        if let Some(value) = self.compatible_prints_condition {
            target.compatible_prints_condition = value;
        }
    }
}

impl Serialize for FilamentPresetSourceOptions {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(5))?;
        serialize_entries(&mut map, self)?;
        map.end()
    }
}

pub(super) fn serialize_entries<M: SerializeMap>(
    map: &mut M,
    value: &FilamentPresetSourceOptions,
) -> Result<(), M::Error> {
    map.serialize_entry(
        "filament_plugin_config_overrides",
        &value.filament_plugin_config_overrides,
    )?;
    map.serialize_entry("compatible_printers", &value.compatible_printers)?;
    map.serialize_entry(
        "compatible_printers_condition",
        &value.compatible_printers_condition,
    )?;
    map.serialize_entry("compatible_prints", &value.compatible_prints)?;
    map.serialize_entry(
        "compatible_prints_condition",
        &value.compatible_prints_condition,
    )?;
    Ok(())
}
