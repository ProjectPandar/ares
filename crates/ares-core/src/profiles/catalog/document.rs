use std::collections::BTreeMap;

use serde::Deserialize;

use crate::project::raw_settings::RawConfigValue;

#[derive(Deserialize)]
pub(in crate::profiles) struct CatalogInput {
    pub vendors: Vec<VendorInput>,
    pub imports: Vec<ImportedProfiles>,
}

#[derive(Deserialize)]
pub(in crate::profiles) struct VendorInput {
    /// `VendorProfile::id`, the vendor index file stem that keys `PresetBundle::vendors`
    /// (`PresetBundle.cpp:7951-7962`).
    pub id: String,
    pub index: VendorIndex,
    pub files: BTreeMap<String, ProfileDocument>,
}

#[derive(Deserialize)]
pub(in crate::profiles) struct VendorIndex {
    pub name: String,
    #[serde(default)]
    pub machine_model_list: Vec<VendorModelEntry>,
    #[serde(default)]
    pub machine_list: Vec<VendorEntry>,
    #[serde(default)]
    pub process_list: Vec<VendorEntry>,
    #[serde(default)]
    pub filament_list: Vec<VendorEntry>,
}

#[derive(Deserialize)]
pub(in crate::profiles) struct VendorEntry {
    pub sub_path: String,
}

/// `machine_model_list` entry: `name` is the model id
/// (`PresetBundle.cpp:4718`, `model.id = machine_model.first`).
#[derive(Deserialize)]
pub(in crate::profiles) struct VendorModelEntry {
    pub name: String,
    pub sub_path: String,
}

#[derive(Deserialize)]
pub(super) struct ImportedProfiles {
    pub profiles: Vec<ProfileDocument>,
}

/// Transport-only original option values. Typed option builders still own
/// interpretation; retaining these values also supplies config-export key/nil
/// provenance without introducing dynamic serde_json values in production.
#[derive(Clone, Deserialize)]
pub(in crate::profiles) struct ProfileDocument {
    #[serde(rename = "type", default)]
    pub kind: String,
    pub name: String,
    #[serde(default)]
    pub inherits: Option<String>,
    #[serde(default)]
    pub instantiation: Option<String>,
    #[serde(rename = "include", default, deserialize_with = "deserialize_includes")]
    pub includes: Vec<String>,
    #[serde(skip)]
    pub preserve_default_thumbnails: bool,
    #[serde(flatten)]
    pub fields: BTreeMap<String, RawConfigValue>,
}

pub(in crate::profiles) fn deserialize_includes<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Names {
        One(String),
        Many(Vec<String>),
    }
    Ok(match Names::deserialize(deserializer)? {
        Names::One(name) => vec![name],
        Names::Many(names) => names,
    })
}

/// Parsed vendor manifest (`PresetBundle.cpp:4712-4821`): carries the vendor
/// identity and list of profile `sub_path`s so hosts can read profile files
/// once from disk and feed raw bytes directly without JSON re-serialization.
pub struct VendorManifest {
    pub(in crate::profiles) id: String,
    pub(in crate::profiles) index: VendorIndex,
}

impl VendorManifest {
    /// Parses a vendor index JSON, returning `Ok(None)` for non-vendor JSON
    /// files (such as `blacklist.json`) that carry no string `"name"`.
    pub fn parse_optional(
        id: impl Into<String>,
        index_json: &[u8],
    ) -> Result<Option<Self>, crate::SliceError> {
        let value: serde_json::Value = serde_json::from_slice(crate::profiles::without_utf8_bom(
            index_json,
        ))
        .map_err(|error| {
            crate::SliceError::InvalidInput(format!("invalid vendor index JSON: {error}"))
        })?;
        if value
            .get("name")
            .and_then(serde_json::Value::as_str)
            .is_none()
        {
            return Ok(None);
        }
        let index: VendorIndex = serde_json::from_value(value).map_err(|error| {
            crate::SliceError::InvalidInput(format!("invalid vendor index JSON: {error}"))
        })?;
        Ok(Some(Self {
            id: id.into(),
            index,
        }))
    }

    /// Parses a required vendor index JSON (fails if `"name"` or any entry's
    /// `sub_path` is missing).
    pub fn parse(id: impl Into<String>, index_json: &[u8]) -> Result<Self, crate::SliceError> {
        let index: VendorIndex = serde_json::from_slice(crate::profiles::without_utf8_bom(
            index_json,
        ))
        .map_err(|error| {
            crate::SliceError::InvalidInput(format!("invalid vendor index JSON: {error}"))
        })?;
        Ok(Self {
            id: id.into(),
            index,
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn sub_paths(&self) -> impl Iterator<Item = &str> {
        self.index
            .machine_model_list
            .iter()
            .map(|entry| entry.sub_path.as_str())
            .chain(
                self.index
                    .machine_list
                    .iter()
                    .chain(&self.index.process_list)
                    .chain(&self.index.filament_list)
                    .map(|entry| entry.sub_path.as_str()),
            )
    }
}

/// Ownership scope referenced by a preset's `inherits` link
/// (`PresetBundle.cpp:614-688`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresetInheritanceSource {
    User,
    System,
}

/// Inspects a preset JSON file's `inherits` and `from` metadata without
/// materializing the full option map.
pub fn preset_inheritance_source(
    json: &str,
) -> Result<Option<PresetInheritanceSource>, crate::SliceError> {
    #[derive(Deserialize)]
    struct Header {
        #[serde(default)]
        inherits: Option<serde_json::Value>,
        #[serde(default)]
        from: Option<serde_json::Value>,
    }
    let header: Header = serde_json::from_str(json).map_err(|error| {
        crate::SliceError::InvalidInput(format!("invalid preset JSON: {error}"))
    })?;
    if header
        .inherits
        .as_ref()
        .and_then(serde_json::Value::as_str)
        .is_none_or(str::is_empty)
    {
        return Ok(None);
    }
    Ok(
        match header.from.as_ref().and_then(serde_json::Value::as_str) {
            Some("user" | "User") => Some(PresetInheritanceSource::User),
            Some("system") => Some(PresetInheritanceSource::System),
            _ => None,
        },
    )
}

impl ProfileDocument {
    pub(in crate::profiles) fn values(&self) -> BTreeMap<String, RawConfigValue> {
        let mut values = self.fields.clone();
        values.insert("type".to_owned(), RawConfigValue::Text(self.kind.clone()));
        values.insert("name".to_owned(), RawConfigValue::Text(self.name.clone()));
        for (key, value) in [
            ("inherits", &self.inherits),
            ("instantiation", &self.instantiation),
        ] {
            if let Some(value) = value {
                values.insert(key.to_owned(), RawConfigValue::Text(value.clone()));
            }
        }
        values
    }
}
