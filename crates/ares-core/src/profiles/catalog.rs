mod chain;
mod cli_metadata;
mod compose;
mod document;
mod model_info;
mod preset_values;
mod printer_models;
#[allow(unused_imports)]
mod raw;
mod store;
mod vendor_models;

use std::collections::BTreeMap;

use serde::Deserialize;

use super::ProfileKind;
use super::composition::ComposedProfile;
use super::composition::ProfileSelection;
use crate::{
    FilteredPresets, SliceError,
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

pub(in crate::profiles) use chain::MissingParentLink;
pub(super) use document::ProfileDocument;
use document::VendorInput;
pub use model_info::PrinterModelInfo;
pub use preset_values::PresetValues;
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
    /// Vendor id per vendor scope (`VendorProfile::id`, the `PresetBundle::vendors` key).
    vendor_ids: Vec<String>,
    /// `VendorProfile::models` per vendor scope.
    vendor_models: Vec<Vec<vendor_models::PrinterModelProfile>>,
    /// Set only by the `--load-settings` entry: every import composes
    /// as its own single record (`load_config_file` direct-load,
    /// `OrcaSlicer.cpp:1953-2010`; the chain reads are commented out,
    /// `:1987,1991`). The preset-bundle/catalog entry keeps the full
    /// `PresetBundle` chain walk for its user imports.
    direct_imports: bool,
}

pub(in crate::profiles) struct Record {
    scope: Scope,
    kind: ProfileKind,
    document: ProfileDocument,
}

#[derive(Deserialize)]
struct SelectionInput {
    machine: String,
    process: String,
    filaments: Vec<String>,
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

    /// One vendor envelope `{id, index, files}` as JSON bytes, exactly the
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

    /// The installed filament set after `PresetBundle::load_installed_filaments`
    /// seeds it (`PresetBundle.cpp:2541-2600`).
    pub fn installed_filaments(&self, installed: &[String]) -> Result<Vec<String>, SliceError> {
        super::compatible::installed_filaments(&self.catalog, installed)
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
