use super::{Catalog, ProfileDocument, Record, Scope, VendorInput, invalid};
use crate::{ProfileKind, SliceError, SourceIdentity};

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

    /// Manifest entries bind by their manifest locator, not external preset names.
    /// Admission later compares only kind and identity (`PresetBundle.cpp:582–600`).
    pub fn add_vendor(
        &mut self,
        json: &str,
        identities: &[(String, SourceIdentity)],
    ) -> Result<(), SliceError> {
        let vendor: VendorInput =
            serde_json::from_str(json).map_err(|error| invalid(error.to_string()))?;
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
            if let Some((_, identity)) = identities.iter().find(|(path, _)| path == &entry.sub_path)
            {
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
            for layer in self.catalog.resolved_layers(*position)? {
                fields.extend(layer.fields.clone());
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
