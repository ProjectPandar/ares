mod user;

use super::{Catalog, ProfileDocument, Record, Scope, VendorInput, invalid};
use crate::{ProfileKind, SliceError, SourceIdentity, project::raw_settings::ProjectSettingsRaw};

/// Loaded collection configs and host identities, separate from external file inputs.
/// Identities are never part of the vendor JSON transport.
pub struct LoadedPresetSources {
    catalog: Catalog,
    bindings: Vec<(usize, SourceIdentity)>,
    loaded: Vec<Record>,
}

impl LoadedPresetSources {
    pub fn new() -> Self {
        Self {
            catalog: Catalog::empty(),
            bindings: Vec::new(),
            loaded: Vec::new(),
        }
    }

    /// A canonical config already admitted by the host's loaded user collection.
    pub fn add_loaded(&mut self, json: &str, identity: &SourceIdentity) -> Result<(), SliceError> {
        let document: ProfileDocument =
            serde_json::from_str(json).map_err(|error| invalid(error.to_string()))?;
        self.loaded.push(Record {
            kind: ProfileKind::parse(&document.kind)?,
            scope: Scope::User,
            document,
            source_identity: Some(identity.clone()),
            direct: true,
        });
        Ok(())
    }

    /// `Preset.cpp:2032–2115`: resolve one directory against previously loaded collections.
    pub fn add_user_collection(
        &mut self,
        entries: &[(String, String, SourceIdentity)],
    ) -> Result<(), SliceError> {
        let mut parsed = Vec::with_capacity(entries.len());
        for (collection_name, json, identity) in entries {
            let document: ProfileDocument =
                serde_json::from_str(json).map_err(|error| invalid(error.to_string()))?;
            let kind = ProfileKind::parse(&document.kind)?;
            parsed.push((collection_name.as_str(), kind, document, identity));
        }
        self.push_user_documents(parsed)
    }

    /// `Preset.cpp:2032–2115`: resolve one typed user directory directly from
    /// raw JSON strings without rewriting `"type"` through an intermediate JSON
    /// value. Non-object or malformed files are ignored like upstream's loader.
    pub fn add_user_directory(
        &mut self,
        kind: ProfileKind,
        entries: &[(String, String, SourceIdentity)],
    ) -> Result<(), SliceError> {
        #[derive(serde::Deserialize)]
        struct RawUserDocument {
            #[serde(default)]
            name: Option<String>,
            #[serde(default)]
            inherits: Option<String>,
            #[serde(default)]
            instantiation: Option<String>,
            #[serde(
                rename = "include",
                default,
                deserialize_with = "super::document::deserialize_includes"
            )]
            includes: Vec<String>,
            #[serde(rename = "type", default)]
            _ignored_type: serde::de::IgnoredAny,
            #[serde(flatten)]
            fields:
                std::collections::BTreeMap<String, crate::project::raw_settings::RawConfigValue>,
        }
        let kind_str = match kind {
            ProfileKind::Machine => "machine",
            ProfileKind::Process => "process",
            ProfileKind::Filament => "filament",
        };
        let mut parsed = Vec::with_capacity(entries.len());
        for (collection_name, json, identity) in entries {
            let Ok(raw) = serde_json::from_str::<RawUserDocument>(json) else {
                continue;
            };
            let Some(name) = raw.name else {
                return Err(invalid("missing field `name`".to_owned()));
            };
            let document = ProfileDocument {
                kind: kind_str.to_owned(),
                name,
                inherits: raw.inherits,
                instantiation: raw.instantiation,
                includes: raw.includes,
                preserve_default_thumbnails: false,
                fields: raw.fields,
            };
            parsed.push((collection_name.as_str(), kind, document, identity));
        }
        self.push_user_documents(parsed)
    }

    fn push_user_documents(
        &mut self,
        entries: Vec<(&str, ProfileKind, ProfileDocument, &SourceIdentity)>,
    ) -> Result<(), SliceError> {
        let mut documents = Vec::with_capacity(entries.len());
        for (collection_name, kind, mut document, identity) in entries {
            let authored_name = std::mem::replace(&mut document.name, collection_name.to_owned());
            if !user::valid_version(&document) {
                continue;
            }
            if self
                .catalog
                .index
                .contains_key(&(Scope::User, kind, document.name.clone()))
                || self
                    .catalog
                    .selectable
                    .contains_key(&(kind, document.name.clone()))
            {
                continue;
            }
            let record = Record {
                scope: Scope::User,
                kind,
                document,
                source_identity: Some(identity.clone()),
                direct: false,
            };
            if let Some(parent) = record
                .document
                .inherits
                .as_deref()
                .filter(|name| !name.is_empty())
                && self.catalog.parent_position(&record, parent).is_none()
            {
                continue;
            }
            documents.push((record, authored_name));
        }
        for (record, _) in &documents {
            self.catalog.push_import(record.document.clone())?;
        }
        for (mut record, authored_name) in documents {
            let position =
                self.catalog.index[&(Scope::User, record.kind, record.document.name.clone())];
            let mut raw = crate::project::raw_settings::ProjectSettingsRaw::default();
            raw.extend(super::super::project_refresh::default_preset_fields(
                record.kind,
            )?);
            for layer in self.catalog.resolved_layers(position)? {
                let mut authored = crate::project::raw_settings::ProjectSettingsRaw::default();
                authored.extend(layer.fields.clone());
                authored.handle_legacy();
                super::super::preset_normalization::replace_nil_from_defaults(&mut authored, &raw)?;
                raw.extend_raw(&authored);
                super::super::preset_normalization::extend_default_config_length(&mut raw)?;
            }
            if record.kind == ProfileKind::Machine {
                super::super::preset_normalization::normalize_machine(&mut raw)?;
            }
            record.document.fields = raw
                .entries()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect();
            record.document.name = authored_name;
            record.direct = true;
            self.loaded.push(record);
        }
        Ok(())
    }

    /// Manifest entries bind by their manifest locator, not external preset names.
    /// Admission later compares only kind and identity (`PresetBundle.cpp:582–600`).
    pub fn add_vendor(
        &mut self,
        json: &str,
        identities: &[(String, SourceIdentity)],
    ) -> Result<(), SliceError> {
        let vendor: VendorInput =
            serde_json::from_str(json).map_err(|error| invalid(error.to_string()))?;
        let identity_map: std::collections::BTreeMap<&str, &SourceIdentity> = identities
            .iter()
            .map(|(path, identity)| (path.as_str(), identity))
            .collect();
        self.push_vendor_input(vendor, &identity_map)
    }

    /// Adds one vendor from its parsed manifest and raw `(sub_path, bytes, identity)`
    /// files directly, avoiding JSON envelope re-serialization.
    pub fn add_vendor_manifest<'a>(
        &mut self,
        manifest: super::VendorManifest,
        files: impl IntoIterator<Item = (&'a str, &'a [u8], SourceIdentity)>,
    ) -> Result<(), SliceError> {
        let mut parsed = std::collections::BTreeMap::new();
        let mut owned_identities = Vec::new();
        for (sub_path, bytes, identity) in files {
            let document: ProfileDocument =
                serde_json::from_slice(crate::profiles::without_utf8_bom(bytes))
                    .map_err(|error| invalid(error.to_string()))?;
            let path = sub_path.to_owned();
            parsed.insert(path.clone(), document);
            owned_identities.push((path, identity));
        }
        let identity_map: std::collections::BTreeMap<&str, &SourceIdentity> = owned_identities
            .iter()
            .map(|(path, identity)| (path.as_str(), identity))
            .collect();
        self.push_vendor_input(
            VendorInput {
                id: manifest.id,
                index: manifest.index,
                files: parsed,
            },
            &identity_map,
        )
    }

    fn push_vendor_input(
        &mut self,
        vendor: VendorInput,
        identities: &std::collections::BTreeMap<&str, &SourceIdentity>,
    ) -> Result<(), SliceError> {
        let scope = Scope::Vendor(self.catalog.vendor_names.len());
        let entries = [
            (ProfileKind::Machine, &vendor.index.machine_list),
            (ProfileKind::Process, &vendor.index.process_list),
            (ProfileKind::Filament, &vendor.index.filament_list),
        ]
        .into_iter()
        .flat_map(|(kind, entries)| entries.iter().map(move |entry| (kind, entry)));
        let mut bindings = Vec::new();
        for (kind, entry) in entries {
            let document = vendor.files.get(&entry.sub_path).ok_or_else(|| {
                invalid(format!("missing vendor profile file {}", entry.sub_path))
            })?;
            if document.instantiation.as_deref() == Some("false") {
                continue;
            }
            if let Some(&identity) = identities.get(entry.sub_path.as_str()) {
                bindings.push((kind, document.name.clone(), identity.clone()));
            }
        }
        self.catalog.push_vendor(vendor)?;
        for (kind, name, identity) in bindings {
            let position = self.catalog.index[&(scope, kind, name)];
            if self.catalog.missing_parent(position).is_none() {
                self.bindings.push((position, identity));
            }
        }
        Ok(())
    }

    pub(in crate::profiles) fn populate(&self, target: &mut Catalog) -> Result<(), SliceError> {
        target.source_records.extend(self.loaded.iter().cloned());
        for (position, identity) in &self.bindings {
            let record = &self.catalog.records[*position];
            let mut document = record.document.clone();
            let mut fields = super::super::project_refresh::default_preset_fields(record.kind)?;
            let layers = self.catalog.resolved_layers(*position)?;
            document.preserve_default_thumbnails = record.kind == ProfileKind::Machine
                && !layers
                    .iter()
                    .any(|layer| layer.fields.contains_key("thumbnails"));
            for layer in layers {
                fields.extend(layer.fields.clone());
            }
            if record.kind == ProfileKind::Machine {
                // PresetBundle.cpp:7839–7849 completes machine vectors after all overlays.
                let mut raw = ProjectSettingsRaw::default();
                raw.extend(fields);
                super::super::preset_normalization::extend_default_config_length(&mut raw)?;
                fields = raw.into_entries();
            }
            document.fields = fields;
            target.source_records.push(Record {
                scope: record.scope,
                kind: record.kind,
                document,
                source_identity: Some(identity.clone()),
                direct: true,
            });
        }
        Ok(())
    }
}

impl Default for LoadedPresetSources {
    fn default() -> Self {
        Self::new()
    }
}
