use std::fmt;

use serde::{Deserialize, Deserializer, de::Visitor};

use super::{
    ProjectSettings, filament_options::FilamentOptionsBuilder,
    preset_metadata::PresetMetadataBuilder, printer_options::PrinterOptionsBuilder,
    process_options::ProcessOptionsBuilder, project_runtime_options::ProjectRuntimeOptionsBuilder,
};

impl<'de> Deserialize<'de> for ProjectSettings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(ProjectSettingsVisitor)
    }
}

struct ProjectSettingsVisitor;

impl<'de> Visitor<'de> for ProjectSettingsVisitor {
    type Value = ProjectSettings;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Orca project settings")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut printer = PrinterOptionsBuilder::default();
        let mut process = ProcessOptionsBuilder::default();
        let mut filament = FilamentOptionsBuilder::default();
        let mut project = ProjectRuntimeOptionsBuilder::default();
        let mut metadata = PresetMetadataBuilder::default();

        while let Some(key) = map.next_key::<String>()? {
            if printer.deserialize_known_field(&key, &mut map)?
                || process.deserialize_known_field(&key, &mut map)?
                || filament.deserialize_known_field(&key, &mut map)?
                || project.deserialize_known_field(&key, &mut map)?
                || metadata.deserialize_known_field(&key, &mut map)?
            {
                continue;
            }

            return Err(serde::de::Error::custom(format!(
                "unknown Orca project option {key}"
            )));
        }

        Ok(ProjectSettings {
            printer: printer.resolve(),
            process: process.resolve(),
            filament: filament.resolve(),
            project: project.resolve(),
            metadata: metadata.resolve(),
        })
    }
}
