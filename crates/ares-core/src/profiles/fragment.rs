pub(super) mod metadata;
pub(super) mod payload;

use std::fmt;

use crate::SliceError;

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
