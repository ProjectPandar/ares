use super::{
    FilamentOptions, PresetMetadata, PrinterOptions, ProcessOptions, ProjectRuntimeOptions,
    filament_options::FilamentOptionsBuilder, preset_metadata::PresetMetadataBuilder,
    printer_options::PrinterOptionsBuilder, process_options::ProcessOptionsBuilder,
    project_runtime_options::ProjectRuntimeOptionsBuilder, typed_legacy::JsonDerivedEffect,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProjectSettings {
    pub printer: PrinterOptions,
    pub process: ProcessOptions,
    pub filament: FilamentOptions,
    pub project: ProjectRuntimeOptions,
    pub metadata: PresetMetadata,
}

#[derive(Default)]
pub(crate) struct ProjectSettingsBuilder {
    printer: PrinterOptionsBuilder,
    process: ProcessOptionsBuilder,
    filament: FilamentOptionsBuilder,
    project: ProjectRuntimeOptionsBuilder,
    metadata: PresetMetadataBuilder,
    derived_support_style: bool,
    derived_is_infill_first: bool,
}

impl ProjectSettingsBuilder {
    pub(crate) fn deserialize_known_field<'de, A>(
        &mut self,
        key: &str,
        map: &mut A,
    ) -> Result<bool, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        if PrinterOptionsBuilder::is_known_field(key) {
            self.printer.deserialize_known_field(key, map)
        } else if ProcessOptionsBuilder::is_known_field(key) {
            self.process.deserialize_known_field(key, map)
        } else if FilamentOptionsBuilder::is_known_field(key) {
            self.filament.deserialize_known_field(key, map)
        } else if ProjectRuntimeOptionsBuilder::is_known_field(key) {
            self.project.deserialize_known_field(key, map)
        } else if PresetMetadataBuilder::is_known_field(key) {
            self.metadata.deserialize_known_field(key, map)
        } else {
            Ok(false)
        }
    }

    pub(crate) fn deserialize_known_value<'de, D>(
        &mut self,
        key: &str,
        deserializer: D,
    ) -> Result<bool, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if PrinterOptionsBuilder::is_known_field(key) {
            self.printer.deserialize_known_value(key, deserializer)
        } else if ProcessOptionsBuilder::is_known_field(key) {
            self.process.deserialize_known_value(key, deserializer)
        } else if FilamentOptionsBuilder::is_known_field(key) {
            self.filament.deserialize_known_value(key, deserializer)
        } else if ProjectRuntimeOptionsBuilder::is_known_field(key) {
            self.project.deserialize_known_value(key, deserializer)
        } else if PresetMetadataBuilder::is_known_field(key) {
            self.metadata.deserialize_known_value(key, deserializer)
        } else {
            Ok(false)
        }
    }

    pub(crate) fn schedule_json_effect(&mut self, effect: JsonDerivedEffect) {
        match (effect.target, effect.value) {
            ("support_style", "tree_hybrid") => self.derived_support_style = true,
            ("is_infill_first", "true") => self.derived_is_infill_first = true,
            _ => unreachable!("unreviewed typed legacy JSON effect"),
        }
    }

    pub(crate) fn apply_thumbnail_composite<E>(&mut self) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        self.printer
            .normalize_present_thumbnails()
            .map_err(|error| {
                E::custom(format_args!(
                    "invalid Orca option thumbnails: {}",
                    crate::thumbnail_error_string(error)
                ))
            })
    }

    pub(crate) fn resolve(mut self) -> ProjectSettings {
        if self.derived_support_style {
            self.process.apply_derived_support_style();
        }
        if self.derived_is_infill_first {
            self.process.apply_derived_is_infill_first();
        }
        ProjectSettings {
            printer: self.printer.resolve(),
            process: self.process.resolve(),
            filament: self.filament.resolve(),
            project: self.project.resolve(),
            metadata: self.metadata.resolve(),
        }
    }
}
