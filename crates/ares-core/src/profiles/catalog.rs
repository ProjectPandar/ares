mod cli_metadata;
mod document;
mod printer_models;
#[allow(unused_imports)]
mod raw;

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use super::{
    ComposedProfile, MergedProfile, ProfileFragment, ProfileKind, ProfileSelection,
    composition::compose_resolved_profiles, merge_profile_fragments,
};
use crate::{
    FilteredPresets, SliceError,
    options::ProjectRuntimeOptionsBuilder,
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

pub(super) use document::ProfileDocument;
use document::{CatalogInput, VendorInput};
pub use printer_models::CurrentPrinter;

/// `Catalog::compose` result: the fully resolved profile, the merged raw
/// presence view, and per-filament raw views for the 3MF slot overlay.
pub(super) struct ComposedSelection {
    pub(super) profile: ComposedProfile,
    pub(super) raw: ProjectSettingsRaw,
    /// One merged raw per selected filament (chain fields only).
    pub(super) filament_raws: Vec<ProjectSettingsRaw>,
}

/// Record ownership: a vendor scope per loaded vendor, plus the user scope
/// for imported presets (the trailing empty vendor name in the one-shot
/// transport).
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum Scope {
    Vendor(usize),
    User,
}

pub(super) struct Catalog {
    records: Vec<Record>,
    index: BTreeMap<(Scope, ProfileKind, String), usize>,
    selectable: BTreeMap<(ProfileKind, String), usize>,
    /// Vendor index name per vendor scope (`VendorProfile::name`).
    vendor_names: Vec<String>,
}

struct Record {
    scope: Scope,
    kind: ProfileKind,
    document: ProfileDocument,
}

impl Catalog {
    pub(super) fn from_json(input: &str) -> Result<Self, SliceError> {
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

    pub(super) fn empty() -> Self {
        Self {
            records: Vec::new(),
            index: BTreeMap::new(),
            selectable: BTreeMap::new(),
            vendor_names: Vec::new(),
        }
    }

    /// `PresetBundle::load_presets` holding semantics: one vendor scope per
    /// load, appended in load order.
    fn push_vendor(&mut self, vendor: VendorInput) -> Result<(), SliceError> {
        let scope = Scope::Vendor(self.vendor_names.len());
        self.vendor_names.push(vendor.index.name.clone());
        self.insert_vendor(scope, vendor)
    }

    /// `PresetBundle::import_json_presets` holding semantics: imports join
    /// the user scope.
    pub(super) fn push_import(&mut self, document: ProfileDocument) -> Result<(), SliceError> {
        self.push_document(Scope::User, document)
    }

    fn push_document(&mut self, scope: Scope, document: ProfileDocument) -> Result<(), SliceError> {
        let kind = ProfileKind::parse(&document.kind)?;
        self.insert(scope, kind, document)
    }

    fn insert_vendor(&mut self, scope: Scope, mut vendor: VendorInput) -> Result<(), SliceError> {
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
            self.insert(scope, kind, document)?;
        }
        Ok(())
    }

    fn insert(
        &mut self,
        scope: Scope,
        kind: ProfileKind,
        document: ProfileDocument,
    ) -> Result<(), SliceError> {
        let position = self.records.len();
        let key = (kind, document.name.clone());
        if self
            .index
            .insert((scope, kind, document.name.clone()), position)
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

    pub(super) fn compose(
        &self,
        selection: &ProfileSelection,
    ) -> Result<ComposedSelection, SliceError> {
        let mut raw = ProjectSettingsRaw::default();
        let mut runtime = ProjectRuntimeOptionsBuilder::default();
        // OrcaSlicer.cpp:2849-2867,2993-3011 applies machine, then process.
        let machine = self.merge(
            ProfileKind::Machine,
            selection.machine(),
            &mut raw,
            &mut runtime,
        )?;
        let process = self.merge(
            ProfileKind::Process,
            selection.process(),
            &mut raw,
            &mut runtime,
        )?;
        // Each filament keeps its own merged raw for the 3MF per-slot overlay
        // (`OrcaSlicer.cpp:3301` set_at per loaded filament index).
        let mut filament_raws = Vec::with_capacity(selection.filaments().len());
        let filaments = selection
            .filaments()
            .iter()
            .map(|name| {
                let mut filament_raw = ProjectSettingsRaw::default();
                let merged =
                    self.merge(ProfileKind::Filament, name, &mut filament_raw, &mut runtime)?;
                raw.extend_raw(&filament_raw);
                filament_raws.push(filament_raw);
                Ok(merged)
            })
            .collect::<Result<Vec<_>, SliceError>>()?;
        cli_metadata::filament_conditions(&mut raw, &filaments);
        let composed =
            compose_resolved_profiles(selection, machine, process, filaments, runtime.resolve());
        if let Some(group) = composed.metadata().inherits_group() {
            raw.insert_strings("inherits_group", group);
        }
        Ok(ComposedSelection {
            profile: composed,
            raw,
            filament_raws,
        })
    }

    fn merge(
        &self,
        kind: ProfileKind,
        name: &str,
        raw: &mut ProjectSettingsRaw,
        runtime: &mut ProjectRuntimeOptionsBuilder,
    ) -> Result<MergedProfile, SliceError> {
        let position = *self
            .selectable
            .get(&(kind, name.to_owned()))
            .ok_or_else(|| invalid(format!("selectable profile '{name}' was not found")))?;
        let mut chain = Vec::new();
        self.collect_positions(position, &mut BTreeSet::new(), &mut chain)?;
        let mut fragments = Vec::with_capacity(chain.len());
        for position in chain {
            let record = &self.records[position];
            let (fragment, project) =
                ProfileFragment::from_catalog_values(record.document.values())?;
            fragments.push(fragment);
            runtime.overlay(project);
            raw.extend(
                record
                    .document
                    .fields
                    .iter()
                    .filter(|(key, _)| raw::applies(kind, key))
                    .map(|(key, value)| (key.clone(), value.clone())),
            );
        }
        let mut merged = merge_profile_fragments(&fragments, kind, name)?;
        if !self.is_user_scope(position) {
            let filament_id = fragments
                .iter()
                .rev()
                .filter_map(ProfileFragment::filament_id)
                .find(|value| !value.is_empty())
                .map(str::to_owned);
            merged.apply_system_metadata(filament_id);
        }
        Ok(merged)
    }

    fn parent_position(&self, record: &Record, parent: &str) -> Option<usize> {
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
        self.library_scope().and_then(|scope| {
            self.index
                .get(&(Scope::Vendor(scope), record.kind, parent.to_owned()))
                .copied()
        })
    }

    fn library_scope(&self) -> Option<usize> {
        self.vendor_names
            .iter()
            .position(|name| name == super::compatible::ORCA_FILAMENT_LIBRARY)
    }

    pub(super) fn selectable_position(&self, kind: ProfileKind, name: &str) -> Option<usize> {
        self.selectable.get(&(kind, name.to_owned())).copied()
    }

    pub(super) fn selectable_positions(&self, kind: ProfileKind) -> Vec<usize> {
        // Upstream iterates `m_presets` in load order (`Preset.cpp:3355`); the
        // record vector preserves vendor-then-index insertion order.
        self.records
            .iter()
            .enumerate()
            .filter(|(position, record)| {
                record.kind == kind
                    && self.selectable.get(&(kind, record.document.name.clone())) == Some(position)
            })
            .map(|(position, _)| position)
            .collect()
    }

    pub(super) fn is_user_scope(&self, position: usize) -> bool {
        self.records[position].scope == Scope::User
    }

    /// The record's own `inherits` value (`Preset::inherits` reads the stored
    /// option; `Preset.hpp:320-321`).
    pub(super) fn leaf_inherits(&self, position: usize) -> Option<String> {
        self.records[position]
            .document
            .inherits
            .as_deref()
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    }

    pub(super) fn vendor_name(&self, position: usize) -> &str {
        match self.records[position].scope {
            Scope::Vendor(index) => &self.vendor_names[index],
            Scope::User => "",
        }
    }

    pub(super) fn document(&self, position: usize) -> &ProfileDocument {
        &self.records[position].document
    }

    /// The first unresolvable `inherits` link on the preset's chain, reported
    /// as (holder kind, holder name, missing parent). Upstream skips such
    /// presets at load (`Preset.cpp:1685-1691`, `PresetBundle.cpp:1521-1525`);
    /// the handle keeps them but they stay invisible to filtering until the
    /// parent arrives.
    pub(super) fn missing_parent(&self, position: usize) -> Option<MissingParentLink> {
        let mut position = position;
        let mut steps = self.records.len() + 1;
        loop {
            // Cycle guard: `collect_positions` reports inheritance cycles as
            // errors; this walk only needs the first missing link.
            steps = steps.saturating_sub(1);
            if steps == 0 {
                return None;
            }
            let record = &self.records[position];
            let parent = record
                .document
                .inherits
                .as_deref()
                .filter(|name| !name.is_empty())?;
            match self.parent_position(record, parent) {
                Some(next) => position = next,
                None => {
                    return Some(MissingParentLink {
                        kind: record.kind,
                        name: record.document.name.clone(),
                        parent: parent.to_owned(),
                    });
                }
            }
        }
    }

    /// Parent-first record positions for the preset at `position`.
    pub(super) fn chain_positions(&self, position: usize) -> Result<Vec<usize>, SliceError> {
        let mut chain = Vec::new();
        self.collect_positions(position, &mut BTreeSet::new(), &mut chain)?;
        Ok(chain)
    }

    fn collect_positions(
        &self,
        position: usize,
        visiting: &mut BTreeSet<usize>,
        chain: &mut Vec<usize>,
    ) -> Result<(), SliceError> {
        if !visiting.insert(position) {
            return Err(invalid("profile inheritance cycle"));
        }
        let record = &self.records[position];
        if let Some(parent) = record
            .document
            .inherits
            .as_deref()
            .filter(|name| !name.is_empty())
        {
            let parent_position = self.parent_position(record, parent).ok_or_else(|| {
                invalid(format!(
                    "parent profile '{parent}' was not found for '{}'",
                    record.document.name
                ))
            })?;
            self.collect_positions(parent_position, visiting, chain)?;
        }
        chain.push(position);
        Ok(())
    }
}

#[derive(Deserialize)]
struct SelectionInput {
    machine: String,
    process: String,
    filaments: Vec<String>,
}

pub(super) struct MissingParentLink {
    pub(super) kind: ProfileKind,
    pub(super) name: String,
    pub(super) parent: String,
}

/// One selectable printer for the listing API: `PresetComboBoxes.cpp:1170-1186`
/// shows name grouped by vendor; the model/variant columns come from the
/// preset's own config (`ConfigWizard` grouping), read here so TS never
/// parses preset bodies.
#[derive(Debug, serde::Serialize)]
pub struct SelectablePrinter {
    pub vendor: String,
    pub name: String,
    pub printer_model: String,
    pub printer_variant: String,
}

pub(super) fn document_text(document: &ProfileDocument, key: &str) -> String {
    match document.fields.get(key) {
        Some(RawConfigValue::Text(value)) => value.clone(),
        _ => String::new(),
    }
}

/// Stateful preset catalog (`PresetBundle` holding semantics:
/// `PresetCollection::load_presets` keeps loaded vendors in the collection;
/// `import_json_presets` adds user imports). Filtering reuses the loaded
/// records without re-parsing vendor data per call.
pub struct PresetCatalogHandle {
    catalog: Catalog,
}

impl PresetCatalogHandle {
    pub fn new() -> Self {
        Self {
            catalog: Catalog::empty(),
        }
    }

    /// One vendor envelope `{index, files}` as JSON bytes, exactly the
    /// one-shot transport's `vendors[]` element.
    pub fn add_vendor(&mut self, vendor_json: &[u8]) -> Result<(), SliceError> {
        let vendor: VendorInput = serde_json::from_slice(vendor_json)
            .map_err(|error| invalid(format!("invalid vendor profile JSON: {error}")))?;
        self.catalog.push_vendor(vendor)
    }

    /// An Orca preset JSON or `.orca_*`/`.zip` import, reusing the bundle
    /// importer (`PresetBundle.cpp:1304-1549`).
    pub fn add_import(&mut self, input: &[u8], filename: &str) -> Result<(), SliceError> {
        for document in super::import::parse_import_documents(input, filename)? {
            self.catalog.push_import(document)?;
        }
        Ok(())
    }

    /// Selectable printers in collection order (`Preset.cpp:3355`), grouped
    /// by their owning vendor. `PresetComboBoxes.cpp:1170-1186` skips presets
    /// that are not visible: load-time visibility is
    /// `instantiation != "false"` (`Preset.cpp:1663`), and printers with
    /// unresolvable parents never enter the collection (`Preset.cpp:1685-1691`).
    /// The GUI's per-model/variant install matrix (`Preset.cpp:851-864`,
    /// AppConfig state) has no Ares counterpart: the caller's `add_vendor`
    /// set is the installed set, matching the CLI which does not filter
    /// visibility.
    pub fn printers(&self) -> Vec<SelectablePrinter> {
        self.catalog
            .selectable_positions(ProfileKind::Machine)
            .into_iter()
            .filter(|position| self.catalog.missing_parent(*position).is_none())
            .map(|position| {
                let document = self.catalog.document(position);
                SelectablePrinter {
                    vendor: if self.catalog.is_user_scope(position) {
                        "user".to_owned()
                    } else {
                        self.catalog.vendor_name(position).to_owned()
                    },
                    name: document.name.clone(),
                    printer_model: document_text(document, "printer_model"),
                    printer_variant: document_text(document, "printer_variant"),
                }
            })
            .collect()
    }

    pub fn filter_compatible(
        &self,
        printer: &str,
        process: Option<&str>,
    ) -> Result<FilteredPresets, SliceError> {
        super::compatible::filter_with_catalog(&self.catalog, printer, process)
    }
}

impl Default for PresetCatalogHandle {
    fn default() -> Self {
        Self::new()
    }
}

pub(super) fn selection(input: &str) -> Result<ProfileSelection, SliceError> {
    let input: SelectionInput = serde_json::from_str(input).map_err(invalid_json)?;
    ProfileSelection::new(input.process, input.machine, input.filaments)
}

fn invalid_json(error: serde_json::Error) -> SliceError {
    invalid(format!(
        "invalid profile catalog or selection JSON: {error}"
    ))
}

fn invalid(message: impl Into<String>) -> SliceError {
    SliceError::InvalidInput(message.into())
}
