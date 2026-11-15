//! PresetBundle loading: vendor/import insertion and scope index
//! building (`PresetCollection::load_presets`, `import_json_presets`).

use super::ProfileDocument;
use super::document::{CatalogInput, VendorInput};
use super::{Catalog, Record, Scope, invalid, invalid_json};
use crate::profiles::ProfileKind;
use std::collections::BTreeMap;

use crate::SliceError;

impl Catalog {
    pub(in crate::profiles) fn from_json(input: &str) -> Result<Self, SliceError> {
        let input = input.strip_prefix('\u{feff}').unwrap_or(input);
        let input: CatalogInput = serde_json::from_str(input).map_err(invalid_json)?;
        let mut catalog = Self::empty();
        for vendor in input.vendors {
            catalog.push_vendor(vendor)?;
        }
        for package in input.imports {
            for document in package.profiles {
                catalog.push_document(Scope::User, document)?;
            }
        }
        Ok(catalog)
    }

    pub(in crate::profiles) fn empty() -> Self {
        Self {
            records: super::defaults::records(),
            index: BTreeMap::new(),
            selectable: BTreeMap::new(),
            vendor_names: Vec::new(),
            vendor_ids: Vec::new(),
            vendor_models: Vec::new(),
            installed_printers: None,
            direct_imports: false,
        }
    }

    /// Marks this imports-only catalog as the `--load-settings` entry:
    /// every record composes as its own single chain (the sparse typed-file
    /// resolver bypass at `OrcaSlicer.cpp:2121-2123`).
    pub(in crate::profiles) fn enable_direct_imports(&mut self) {
        self.direct_imports = true;
    }

    /// `PresetBundle::load_presets` holding semantics: one vendor scope per
    /// load, appended in load order.
    pub(in crate::profiles) fn push_vendor(
        &mut self,
        vendor: VendorInput,
    ) -> Result<(), SliceError> {
        let scope = Scope::Vendor(self.vendor_names.len());
        self.vendor_names.push(vendor.index.name.clone());
        self.vendor_ids.push(vendor.id.clone());
        self.insert_vendor(scope, vendor)
    }

    /// `PresetBundle::import_json_presets` holding semantics: imports join
    /// the user scope.
    pub(in crate::profiles) fn push_import(
        &mut self,
        document: ProfileDocument,
    ) -> Result<(), SliceError> {
        self.push_document(Scope::User, document)
    }

    pub(in crate::profiles) fn push_filament_slot(
        &mut self,
        document: ProfileDocument,
        slot: usize,
    ) -> Result<String, SliceError> {
        // CLI file slots own configs independently; authored preset names stay metadata.
        let key = slot.to_string();
        self.insert(Scope::User, ProfileKind::Filament, key.clone(), document)?;
        Ok(key)
    }

    fn push_document(&mut self, scope: Scope, document: ProfileDocument) -> Result<(), SliceError> {
        let kind = ProfileKind::parse(&document.kind)?;
        self.insert(scope, kind, document.name.clone(), document)
    }

    fn insert_vendor(&mut self, scope: Scope, mut vendor: VendorInput) -> Result<(), SliceError> {
        // `PresetBundle.cpp:4712-4821`: the machine models load before the presets.
        let mut models = Vec::new();
        for entry in &vendor.index.machine_model_list {
            let document = vendor.files.remove(&entry.sub_path).ok_or_else(|| {
                invalid(format!("missing vendor profile file {}", entry.sub_path))
            })?;
            models.extend(super::vendor_models::printer_model(entry, &document));
        }
        self.vendor_models.push(models);
        let entries = [
            (ProfileKind::Machine, vendor.index.machine_list),
            (ProfileKind::Process, vendor.index.process_list),
            (ProfileKind::Filament, vendor.index.filament_list),
        ]
        .into_iter()
        .flat_map(|(kind, entries)| entries.into_iter().map(move |entry| (kind, entry)));
        for (kind, entry) in entries {
            let document = vendor.files.remove(&entry.sub_path).ok_or_else(|| {
                invalid(format!("missing vendor profile file {}", entry.sub_path))
            })?;
            self.insert(scope, kind, document.name.clone(), document)?;
        }
        Ok(())
    }

    fn insert(
        &mut self,
        scope: Scope,
        kind: ProfileKind,
        identity: String,
        mut document: ProfileDocument,
    ) -> Result<(), SliceError> {
        // Bare empty preset strings on vector-kind options are EMPTY
        // vectors upstream (`ConfigOptionStrings::deserialize`,
        // Config.hpp:1153-1159); normalize at the parse boundary so the raw
        // and typed views agree on the shape.
        crate::options::json_deserialize::normalize_empty_vector_strings(&mut document.fields);
        crate::options::json_deserialize::normalize_scalar_arrays(&mut document.fields)
            .map_err(invalid)?;
        // Old-profile percentage values for now-absolute keys are erased at
        // file load (`PrintConfig.cpp:8087-8094`, via `set_deserialize`'
        // `handle_legacy`, `Config.cpp:577-579`) — BEFORE the fragments
        // merge into the typed profile, so a `"35%"` `initial_layer_speed`
        // or `"100%"` `support_interface_speed` must never reach the typed
        // composition (the late `raw`-only pass in `compose` cannot undo
        // an already-typed value). Key renames and alias collisions stay
        // in the fragment merge, where the native-anchored alias tests
        // pin them.
        document.fields.retain(|key, value| {
            !(crate::options::legacy::erases_percentage_for_absolute_keys(key)
                && value.token().is_some_and(|token| token.contains('%')))
        });
        let position = self.records.len();
        let key = (kind, identity.clone());
        if self
            .index
            .insert((scope, kind, identity), position)
            .is_some()
        {
            return Err(invalid(format!(
                "duplicate profile {} within vendor scope",
                document.name
            )));
        }
        if document.instantiation.as_deref() != Some("false")
            && self.selectable.insert(key, position).is_some()
        {
            return Err(invalid(format!(
                "ambiguous selectable profile {}",
                document.name
            )));
        }
        self.records.push(Record {
            scope,
            kind,
            document,
        });
        Ok(())
    }

    pub(in crate::profiles) fn parent_position(
        &self,
        record: &Record,
        parent: &str,
    ) -> Option<usize> {
        // PresetBundle.cpp:4885-4918: local templates precede the base library.
        let key = (record.scope, record.kind, parent.to_owned());
        if let Some(position) = self.index.get(&key) {
            return Some(*position);
        }
        if record.scope == Scope::User {
            // `find_preset` searches the whole collection, including
            // non-instantiable library templates.
            if let Some(position) = self
                .selectable
                .get(&(record.kind, parent.to_owned()))
                .copied()
            {
                return Some(position);
            }
        }
        self.library_scope()
            .and_then(|scope| {
                self.index
                    .get(&(Scope::Vendor(scope), record.kind, parent.to_owned()))
                    .copied()
            })
            .or_else(|| {
                let position = self.default_position(record.kind);
                (self.document(position).name == parent).then_some(position)
            })
    }

    fn library_scope(&self) -> Option<usize> {
        self.vendor_names
            .iter()
            .position(|name| name == crate::profiles::compatible::ORCA_FILAMENT_LIBRARY)
    }
}
