mod cli_metadata;
mod document;
mod printer_models;
#[allow(unused_imports)]
mod raw;

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use super::{
    ComposedProfile, MergedProfile, ProfileFragment, ProfileKind, ProfileSelection,
    composition::compose_resolved_profiles,
    inheritance::{UnselectedKind, merge_direct_profile_fragment, unselected_profile},
    merge_profile_fragments,
};
use crate::{
    FilteredPresets, SliceError,
    options::ProjectRuntimeOptionsBuilder,
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

pub(super) use document::ProfileDocument;
use document::{CatalogInput, VendorInput};
pub use printer_models::CurrentPrinter;

/// `OrcaSlicer.cpp:546` gcodes key set created on the machine config by
/// `load_default_gcodes_to_config` (`:548-586`).
const PRINTER_GCODE_KEYS: [&str; 10] = [
    "change_filament_gcode",
    "layer_change_gcode",
    "machine_end_gcode",
    "machine_pause_gcode",
    "machine_start_gcode",
    "template_custom_gcode",
    "printing_by_object_gcode",
    "before_layer_change_gcode",
    "time_lapse_gcode",
    "wrapping_detection_gcode",
];

/// `OrcaSlicer.cpp:587-606` filament keys resized to one empty slot.
const FILAMENT_GCODE_KEYS: [&str; 2] = ["filament_start_gcode", "filament_end_gcode"];

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
    /// Set only by the `--load-settings` entry: every import composes
    /// as its own single record (`load_config_file` direct-load,
    /// `OrcaSlicer.cpp:1953-2010`; the chain reads are commented out,
    /// `:1987,1991`). The preset-bundle/catalog entry keeps the full
    /// `PresetBundle` chain walk for its user imports.
    direct_imports: bool,
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
            direct_imports: false,
        }
    }

    /// Marks this imports-only catalog as the `--load-settings` entry:
    /// every record composes as its own single chain (direct-load,
    /// `OrcaSlicer.cpp:1953-2010`).
    pub(super) fn enable_direct_imports(&mut self) {
        self.direct_imports = true;
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
        mut document: ProfileDocument,
    ) -> Result<(), SliceError> {
        // Bare empty preset strings on vector-kind options are EMPTY
        // vectors upstream (`ConfigOptionStrings::deserialize`,
        // Config.hpp:1153-1159); normalize at the parse boundary so the raw
        // and typed views agree on the shape.
        crate::options::json_deserialize::normalize_empty_vector_strings(&mut document.fields);
        crate::options::json_deserialize::normalize_numeric_scalar_arrays(&mut document.fields)
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
        let machine = match selection.machine() {
            Some(name) => {
                let merged = self.merge(ProfileKind::Machine, name, &mut raw, &mut runtime)?;
                // `load_default_gcodes_to_config(load_machine_config,
                // TYPE_PRINTER)` (`OrcaSlicer.cpp:2850,548-586`): every
                // printer G-code option exists on the machine config,
                // created with the option default — empty except the two
                // templates (PrintConfig.cpp:5947,2035) — before
                // `update_full_config` overlays the machine onto the
                // embedded project settings. With no machine loaded
                // there is no machine config to default (`:2009-2106`).
                insert_default_gcodes(&mut raw, &PRINTER_GCODE_KEYS);
                merged
            }
            None => unselected_profile(UnselectedKind::Machine),
        };
        let process = match selection.process() {
            Some(name) => self.merge(ProfileKind::Process, name, &mut raw, &mut runtime)?,
            None => unselected_profile(UnselectedKind::Process),
        };
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
                // `load_default_gcodes_to_config(config, TYPE_FILAMENT)`
                // (`OrcaSlicer.cpp:3150,587-606`): an empty filament
                // start/end G-code vector is resized to one empty slot.
                insert_default_gcodes(&mut filament_raw, &FILAMENT_GCODE_KEYS);
                resize_empty_filament_gcodes(&mut filament_raw);
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
        // Every loaded preset file normalizes through
        // `set_deserialize`/`handle_legacy` at load (`Config.cpp:577-579`,
        // `PrintConfig.cpp:8033`), before `update_full_config` overlays it
        // onto the embedded project (`OrcaSlicer.cpp:2750-2812`); the
        // embedded base keeps its own post-merge normalization in the
        // profile entries.
        raw.handle_legacy();
        for filament_raw in &mut filament_raws {
            filament_raw.handle_legacy();
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
        // The `--load-settings` entry never resolves `inherits`
        // (`OrcaSlicer.cpp:1953-2010` loads each file's fields directly;
        // `inherits` is a value, not a link): each import composes as
        // its own single record. The preset-bundle/catalog entry keeps
        // the full `PresetBundle` chain walk.
        if self.direct_imports {
            chain.push(position);
        } else {
            self.collect_positions(position, &mut BTreeSet::new(), &mut chain)?;
        }
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
        let mut merged = if self.direct_imports {
            merge_direct_profile_fragment(&fragments, kind, name)?
        } else {
            merge_profile_fragments(&fragments, kind, name)?
        };
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

/// OrcaSlicer.cpp:588-605, `load_default_gcodes_to_config(TYPE_FILAMENT)`:
/// resize empty start/end G-code vectors to one empty string.
fn resize_empty_filament_gcodes(raw: &mut ProjectSettingsRaw) {
    for key in FILAMENT_GCODE_KEYS {
        let empty = match raw.get(key).unwrap() {
            RawConfigValue::Text(token) => matches!(
                crate::options::json_deserialize::decode(crate::OptionValueKind::Strings, token)
                    .unwrap(),
                RawConfigValue::List(values) if values.is_empty()
            ),
            RawConfigValue::List(values) => values.is_empty(),
            _ => unreachable!("loaded filament G-code is a string vector"),
        };
        if !empty {
            continue;
        }
        raw.insert_strings(key, &[String::new()]);
    }
}

/// `option(key, true)` with the option default (`Config.hpp` create path):
/// absent keys are created from the registry default, present keys stay.
/// The pairs are filtered out of the option registry, so every inserted
/// default is a registry `OptionDefinition::default_value`.
fn insert_default_gcodes(raw: &mut ProjectSettingsRaw, keys: &[&str]) {
    for definition in crate::option_definitions() {
        if !keys.contains(&definition.key) {
            continue;
        }
        if !raw.contains_key(definition.key) {
            raw.insert(
                definition.key.to_owned(),
                RawConfigValue::Text(definition.default_value.to_owned()),
            );
        }
    }
}

fn invalid(message: impl Into<String>) -> SliceError {
    SliceError::InvalidInput(message.into())
}
