use std::fmt;

use serde::{Deserialize, de::IgnoredAny};

use super::ProfileKind;
use crate::SliceError;

pub(super) fn is_metadata_key(key: &str) -> bool {
    matches!(
        key,
        "type"
            | "name"
            | "inherits"
            | "from"
            | "version"
            | "setting_id"
            | "instantiation"
            | "description"
            | "url"
            | "renamed_from"
            | "filament_id"
            | "compatible_printers"
            | "compatible_printers_condition"
            | "compatible_prints"
            | "compatible_prints_condition"
    )
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::profiles) struct ProfilePresetMetadata {
    pub(super) kind: ProfileKind,
    pub(super) name: String,
    pub(super) from: Option<String>,
    pub(super) version: Option<String>,
    pub(super) setting_id: Option<String>,
    pub(super) instantiation: Option<String>,
    pub(super) description: Option<String>,
    pub(super) url: Option<String>,
    pub(super) renamed_from: Option<String>,
    pub(super) filament_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::profiles) struct ProfileConfigMetadataPatch {
    pub(in crate::profiles) inherits: Option<String>,
    pub(in crate::profiles) compatible_printers: Option<Vec<String>>,
    pub(in crate::profiles) compatible_printers_condition: Option<String>,
    pub(in crate::profiles) compatible_prints: Option<Vec<String>>,
    pub(in crate::profiles) compatible_prints_condition: Option<String>,
}

impl ProfileConfigMetadataPatch {
    pub(in crate::profiles) fn overlay_compatibility(&mut self, child: &Self) {
        if let Some(value) = &child.compatible_printers {
            self.compatible_printers = Some(value.clone());
        }
        if let Some(value) = &child.compatible_printers_condition {
            self.compatible_printers_condition = Some(value.clone());
        }
        if let Some(value) = &child.compatible_prints {
            self.compatible_prints = Some(value.clone());
        }
        if let Some(value) = &child.compatible_prints_condition {
            self.compatible_prints_condition = Some(value.clone());
        }
    }
}

pub(super) struct ParsedMetadata {
    pub(super) preset: ProfilePresetMetadata,
    pub(super) config: ProfileConfigMetadataPatch,
}

pub(super) fn parse(input: &[u8]) -> Result<ParsedMetadata, SliceError> {
    let mut deserializer = serde_json::Deserializer::from_slice(input);
    let raw = RawMetadata::deserialize(&mut deserializer).map_err(invalid_profile)?;
    deserializer.end().map_err(invalid_profile)?;
    raw.finish()
}

fn invalid_profile(error: serde_json::Error) -> SliceError {
    SliceError::InvalidInput(format!("profile JSON is invalid: {error}"))
}

#[derive(Default)]
struct RawMetadata {
    kind: Option<String>,
    name: Option<String>,
    inherits: Option<String>,
    from: Option<String>,
    version: Option<String>,
    setting_id: Option<String>,
    instantiation: Option<String>,
    description: Option<String>,
    url: Option<String>,
    renamed_from: Option<String>,
    filament_id: Option<String>,
    compatible_printers: Option<Vec<String>>,
    compatible_printers_condition: Option<String>,
    compatible_prints: Option<Vec<String>>,
    compatible_prints_condition: Option<String>,
}

impl RawMetadata {
    fn finish(mut self) -> Result<ParsedMetadata, SliceError> {
        let kind = self
            .kind
            .as_deref()
            .ok_or_else(|| SliceError::InvalidInput("profile type is required".to_owned()))
            .and_then(ProfileKind::parse)?;
        let name = self
            .name
            .take()
            .ok_or_else(|| SliceError::InvalidInput("profile name is required".to_owned()))?;
        if name.is_empty() {
            return Err(SliceError::InvalidInput(
                "profile name must not be empty".to_owned(),
            ));
        }
        self.validate_ownership(kind)?;
        if self.inherits.as_deref() == Some("") {
            self.inherits = None;
        }

        Ok(ParsedMetadata {
            preset: ProfilePresetMetadata {
                kind,
                name,
                from: self.from,
                version: self.version,
                setting_id: self.setting_id,
                instantiation: self.instantiation,
                description: self.description,
                url: self.url,
                renamed_from: self.renamed_from,
                filament_id: self.filament_id,
            },
            config: ProfileConfigMetadataPatch {
                inherits: self.inherits,
                compatible_printers: self.compatible_printers,
                compatible_printers_condition: self.compatible_printers_condition,
                compatible_prints: self.compatible_prints,
                compatible_prints_condition: self.compatible_prints_condition,
            },
        })
    }

    fn validate_ownership(&self, kind: ProfileKind) -> Result<(), SliceError> {
        if kind != ProfileKind::Filament && self.filament_id.is_some() {
            return Err(SliceError::InvalidInput(
                "profile filament_id belongs only to filament profiles".to_owned(),
            ));
        }
        if kind == ProfileKind::Machine
            && (self.compatible_printers.is_some()
                || self.compatible_printers_condition.is_some()
                || self.compatible_prints.is_some()
                || self.compatible_prints_condition.is_some())
        {
            return Err(SliceError::InvalidInput(
                "profile compatibility metadata is not valid for machine profiles".to_owned(),
            ));
        }
        if kind == ProfileKind::Process
            && (self.compatible_prints.is_some() || self.compatible_prints_condition.is_some())
        {
            return Err(SliceError::InvalidInput(
                "profile print compatibility belongs only to filament profiles".to_owned(),
            ));
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for RawMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(MetadataVisitor)
    }
}

struct MetadataVisitor;

impl<'de> serde::de::Visitor<'de> for MetadataVisitor {
    type Value = RawMetadata;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an Orca profile object")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut raw = RawMetadata::default();
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "type" => read_once(&mut raw.kind, &key, &mut map)?,
                "name" => read_once(&mut raw.name, &key, &mut map)?,
                "inherits" => read_once(&mut raw.inherits, &key, &mut map)?,
                "from" => read_once(&mut raw.from, &key, &mut map)?,
                "version" => read_once(&mut raw.version, &key, &mut map)?,
                "setting_id" => read_once(&mut raw.setting_id, &key, &mut map)?,
                "instantiation" => read_once(&mut raw.instantiation, &key, &mut map)?,
                "description" => read_once(&mut raw.description, &key, &mut map)?,
                "url" => read_once(&mut raw.url, &key, &mut map)?,
                "renamed_from" => read_once(&mut raw.renamed_from, &key, &mut map)?,
                "filament_id" => read_once(&mut raw.filament_id, &key, &mut map)?,
                "compatible_printers" => read_once(&mut raw.compatible_printers, &key, &mut map)?,
                "compatible_printers_condition" => {
                    read_once(&mut raw.compatible_printers_condition, &key, &mut map)?
                }
                "compatible_prints" => read_once(&mut raw.compatible_prints, &key, &mut map)?,
                "compatible_prints_condition" => {
                    read_once(&mut raw.compatible_prints_condition, &key, &mut map)?
                }
                _ => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok(raw)
    }
}

fn read_once<'de, A, T>(slot: &mut Option<T>, key: &str, map: &mut A) -> Result<(), A::Error>
where
    A: serde::de::MapAccess<'de>,
    T: Deserialize<'de>,
{
    if slot.is_some() {
        return Err(serde::de::Error::custom(format_args!(
            "duplicate profile field {key}"
        )));
    }
    *slot = Some(map.next_value().map_err(|error| {
        serde::de::Error::custom(format_args!("invalid profile field {key}: {error}"))
    })?);
    Ok(())
}
