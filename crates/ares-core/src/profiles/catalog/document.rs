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
#[derive(Deserialize)]
pub(in crate::profiles) struct ProfileDocument {
    #[serde(rename = "type", default)]
    pub kind: String,
    pub name: String,
    #[serde(default)]
    pub inherits: Option<String>,
    #[serde(default)]
    pub instantiation: Option<String>,
    #[serde(flatten)]
    pub fields: BTreeMap<String, RawConfigValue>,
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
