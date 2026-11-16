mod chain;
mod cli_metadata;
mod compose;
mod defaults;
mod document;
mod filament_slots;
mod guide;
mod includes;
mod installed_printers;
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
pub use guide::{GuideFilament, GuideModel, guide_models};
pub use installed_printers::InstalledPrinters;
pub use model_info::PrinterModelInfo;
pub(crate) use model_info::VendorType;
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
    Default,
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
    /// The `AppConfig` vendors section; `None` until the caller sets one.
    installed_printers: Option<InstalledPrinters>,
    /// Set by the sparse `--load-settings` entry: imports compose as single
    /// records, matching typed files without inherits that bypass Native's
    /// resolver (`OrcaSlicer.cpp:2121-2123`). The preset-bundle/catalog entry
    /// keeps the full `PresetBundle` chain walk for its user imports.
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

    /// Parses the same `{vendors, imports}` transport used by profile slicing.
    pub fn from_json(input: &str) -> Result<Self, SliceError> {
        Ok(Self {
            catalog: Catalog::from_json(input)?,
        })
    }

    /// One vendor envelope `{id, index, files}` as JSON bytes, exactly the
    /// one-shot transport's `vendors[]` element.
    pub fn add_vendor(&mut self, vendor_json: &[u8]) -> Result<(), SliceError> {
        let vendor: VendorInput = serde_json::from_slice(super::without_utf8_bom(vendor_json))
            .map_err(|error| invalid(format!("invalid vendor profile JSON: {error}")))?;
        self.catalog.push_vendor(vendor)
    }

    /// CLI:2793–2817 refreshes resolvable system filaments without compacting slots.
    pub(crate) fn refresh_direct(
        &self,
        raw: &ProjectSettingsRaw,
    ) -> Result<ProjectSettingsRaw, SliceError> {
        super::project_refresh::direct_refresh(&self.catalog, raw)
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
    /// The installed printers (`set_installed_printers`) then hide the
    /// printers `set_visible_from_appconfig` hides. Model and variant are the
    /// resolved config values.
    pub fn printers(&self) -> Vec<SelectablePrinter> {
        self.catalog
            .printer_identities()
            .filter(|(position, model, variant)| {
                self.catalog.is_visible_printer(*position, (model, variant))
            })
            .map(|(position, printer_model, printer_variant)| {
                let document = self.catalog.document(position);
                SelectablePrinter {
                    vendor: if self.catalog.is_user_scope(position) {
                        "user".to_owned()
                    } else {
                        self.catalog.vendor_name(position).to_owned()
                    },
                    name: document.name.clone(),
                    printer_model,
                    printer_variant,
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

    /// The print a selection of `printer` holds after `PresetBundle::update_compatible` with
    /// `PresetSelectCompatibleType::Always`: `current` while compatible, else the best
    /// `PreferedPrintProfileMatch` against `current` and the printer's `default_print_profile`
    /// (`PresetBundle.cpp:8635-8639`), or the initialized collection default
    /// when no ordinary print is compatible.
    pub fn compatible_print(
        &self,
        printer: &str,
        current: Option<&str>,
    ) -> Result<Option<String>, SliceError> {
        super::compatible::compatible_print(&self.catalog, printer, current)
    }

    /// The installed filament set after `PresetBundle::load_installed_filaments`
    /// seeds it (`PresetBundle.cpp:2541-2600`).
    pub fn installed_filaments(&self, installed: &[String]) -> Result<Vec<String>, SliceError> {
        super::compatible::installed_filaments(&self.catalog, installed)
    }

    /// The filament of each slot after `PresetBundle::update_compatible(Always)`
    /// (`PresetBundle.cpp:8650-8673`) for the printer and process, with the
    /// system filaments of `installed` visible.
    pub fn select_compatible_filaments(
        &self,
        (printer, process): (&str, Option<&str>),
        filaments: &[String],
        installed: &[String],
    ) -> Result<Vec<String>, SliceError> {
        super::compatible::select_compatible_filaments(
            &self.catalog,
            (printer, process),
            filaments,
            installed,
        )
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
