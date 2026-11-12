pub(super) mod metadata;
pub(super) mod payload;

use std::{collections::BTreeMap, fmt};

use crate::{
    SliceError,
    options::{ProjectRuntimeOptionsBuilder, ProjectSettingsBuilder},
};

use self::{
    metadata::{ProfileConfigMetadataPatch, ProfilePresetMetadata},
    payload::ProfilePayload,
};

pub use super::inheritance::merge_profile_fragments;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ProfileKind {
    Process,
    Filament,
    Machine,
}

impl ProfileKind {
    pub(super) fn parse(value: &str) -> Result<Self, SliceError> {
        match value {
            "process" => Ok(Self::Process),
            "filament" => Ok(Self::Filament),
            "machine" => Ok(Self::Machine),
            _ => Err(SliceError::InvalidInput(
                "profile type is unsupported".to_owned(),
            )),
        }
    }
}

#[derive(Clone)]
pub struct ProfileFragment {
    preset: ProfilePresetMetadata,
    config: ProfileConfigMetadataPatch,
    payload: ProfilePayload,
}

impl ProfileFragment {
    pub fn from_json_bytes(input: impl AsRef<[u8]>) -> Result<Self, SliceError> {
        let input = input.as_ref();
        let parsed = metadata::parse(input)?;
        let payload = payload::parse(input, parsed.preset.kind)?;
        Ok(Self {
            preset: parsed.preset,
            config: parsed.config,
            payload,
        })
    }

    pub(crate) fn from_original_machine_bytes(input: &[u8]) -> Result<Self, SliceError> {
        let parsed = metadata::parse(input)?;
        // OrcaSlicer.cpp:2118–2123 separates flattened direct loads from
        // inherited resources; keep the latter's existing strict handling.
        if parsed.config.inherits.is_some() {
            return Self::from_json_bytes(input);
        }
        Self::from_flattened_config_bytes(input)
    }

    pub(crate) fn from_up_to_date_file_bytes(input: &[u8]) -> Result<Self, SliceError> {
        let parsed = metadata::parse(input)?;
        // OrcaSlicer.cpp:2539–2558 uses the same direct-file loader before
        // selecting the machine's upward vector.
        if parsed.preset.kind != ProfileKind::Machine || parsed.config.inherits.is_some() {
            return Self::from_json_bytes(input);
        }
        Self::from_flattened_config_bytes(input)
    }

    fn from_flattened_config_bytes(input: &[u8]) -> Result<Self, SliceError> {
        let mut values: BTreeMap<String, crate::project::raw_settings::RawConfigValue> =
            serde_json::from_slice(input).map_err(|error| {
                SliceError::InvalidInput(format!("profile JSON is invalid: {error}"))
            })?;
        // Config.cpp:978–1060 loads strings/arrays; other shapes leave defaults.
        values.retain(|_, value| {
            matches!(
                value,
                crate::project::raw_settings::RawConfigValue::Text(_)
                    | crate::project::raw_settings::RawConfigValue::List(_)
            )
        });
        Self::from_catalog_values(values).map(|(profile, _)| profile)
    }

    pub(super) fn from_catalog_values(
        input: BTreeMap<String, crate::project::raw_settings::RawConfigValue>,
    ) -> Result<(Self, ProjectRuntimeOptionsBuilder), SliceError> {
        let input = crate::project::raw_settings::RawConfigValue::Object(input);
        let parsed = metadata::parse_values(input.clone())?;
        // Config.cpp's preset loader uses the same typed legacy normalization as
        // project loading, before parent/child overlay; keep absence as absence.
        let builder = crate::options::json_deserialize::deserialize_preset_config::<
            ProjectSettingsBuilder,
        >(input)
        .map_err(|error| {
            SliceError::InvalidInput(format!("profile option JSON is invalid: {error}"))
        })?;
        let (printer, process, filament, runtime) = builder.into_profile_parts();
        let payload = match parsed.preset.kind {
            ProfileKind::Machine => ProfilePayload::Machine(Box::new(printer)),
            ProfileKind::Process => ProfilePayload::Process(Box::new(process)),
            ProfileKind::Filament => ProfilePayload::Filament(Box::new(filament)),
        };
        Ok((
            Self {
                preset: parsed.preset,
                config: parsed.config,
                payload,
            },
            runtime,
        ))
    }

    pub const fn kind(&self) -> ProfileKind {
        self.preset.kind
    }

    pub fn name(&self) -> &str {
        &self.preset.name
    }

    pub fn inherits(&self) -> Option<&str> {
        self.config.inherits.as_deref()
    }

    pub fn from(&self) -> Option<&str> {
        self.preset.from.as_deref()
    }

    pub fn version(&self) -> Option<&str> {
        self.preset.version.as_deref()
    }

    pub fn setting_id(&self) -> Option<&str> {
        self.preset.setting_id.as_deref()
    }

    pub fn instantiation(&self) -> Option<&str> {
        self.preset.instantiation.as_deref()
    }

    pub fn description(&self) -> Option<&str> {
        self.preset.description.as_deref()
    }

    pub fn url(&self) -> Option<&str> {
        self.preset.url.as_deref()
    }

    pub fn renamed_from(&self) -> Option<&str> {
        self.preset.renamed_from.as_deref()
    }

    pub fn filament_id(&self) -> Option<&str> {
        self.preset.filament_id.as_deref()
    }

    pub(super) fn config(&self) -> &ProfileConfigMetadataPatch {
        &self.config
    }

    pub(super) fn payload(&self) -> &ProfilePayload {
        &self.payload
    }
}

impl fmt::Debug for ProfileFragment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProfileFragment")
            .field("kind", &self.kind())
            .field("name", &self.name())
            .field("inherits", &self.inherits())
            .field("from", &self.from())
            .field("version", &self.version())
            .field("setting_id", &self.setting_id())
            .field("instantiation", &self.instantiation())
            .field("description", &self.description())
            .field("url", &self.url())
            .field("renamed_from", &self.renamed_from())
            .field("filament_id", &self.filament_id())
            .field("payload", &self.payload)
            .finish()
    }
}

impl PartialEq for ProfileFragment {
    fn eq(&self, other: &Self) -> bool {
        self.preset == other.preset && self.config == other.config && self.payload == other.payload
    }
}
