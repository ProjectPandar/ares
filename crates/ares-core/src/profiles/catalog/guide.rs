//! The setup guide's model and filament data, `GuideFrame::BuildProfileJson`
//! (`WebGuideDialog.cpp:1385-1460`).

use std::collections::BTreeMap;

use super::document::VendorInput;
use super::{PresetCatalogHandle, invalid};
use crate::SliceError;
use crate::profiles::ProfileKind;
use crate::project::raw_settings::RawConfigValue;

/// One `m_ProfileJson["model"]` entry (`:1389-1413`); the cover is not ported.
#[derive(Debug, serde::Serialize)]
pub struct GuideModel {
    /// `VendorProfile::id`.
    pub vendor: String,
    /// `PrinterModel::id`, the `printer_model` of the model's presets.
    pub model: String,
    pub name: String,
    /// The nozzle variants, in `nozzle_diameter` order.
    pub variants: Vec<String>,
    pub default_materials: Vec<String>,
}

/// One `m_ProfileJson["filament"]` entry (`:1429-1457`).
#[derive(Debug, PartialEq, Eq, serde::Serialize)]
pub struct GuideFilament {
    pub name: String,
    /// The first `filament_vendor` value.
    pub vendor: String,
    /// The first `filament_type` value.
    pub filament_type: String,
    /// `(printer_model, printer_variant)` of each `compatible_printers` entry that names a system
    /// printer with a model, the `[model++variant]` tags.
    pub models: Vec<(String, String)>,
}

/// The models of vendor envelopes `[{id, index: {name, machine_model_list}, files}]`, read as
/// `VendorProfile::PrinterModel` (`PresetBundle.cpp:4712-4821`), in vendor and list order. Only
/// the model files are needed, so the guide lists vendors whose presets are not loaded.
pub fn guide_models(vendors_json: &[u8]) -> Result<Vec<GuideModel>, SliceError> {
    let vendors: Vec<VendorInput> = serde_json::from_slice(vendors_json)
        .map_err(|error| invalid(format!("invalid vendor model JSON: {error}")))?;
    let mut models = Vec::new();
    for vendor in vendors {
        for entry in &vendor.index.machine_model_list {
            let document = vendor.files.get(&entry.sub_path).ok_or_else(|| {
                invalid(format!("missing vendor profile file {}", entry.sub_path))
            })?;
            models.extend(
                super::vendor_models::printer_model(entry, document).map(|model| GuideModel {
                    vendor: vendor.id.clone(),
                    model: model.id,
                    name: model.name,
                    variants: model.variants,
                    default_materials: model.default_materials,
                }),
            );
        }
    }
    Ok(models)
}

impl PresetCatalogHandle {
    /// The loaded system filaments in name order (the `json` object's key order), each tagged with
    /// the system printers its `compatible_printers` names.
    pub fn guide_filaments(&self) -> Vec<GuideFilament> {
        let catalog = &self.catalog;
        // `:1416-1426`: system printers with a model, by name.
        let machines: BTreeMap<String, (String, String)> = catalog
            .printer_identities()
            .filter(|(position, model, _)| !catalog.is_user_scope(*position) && !model.is_empty())
            .map(|(position, model, variant)| {
                (catalog.document(position).name.clone(), (model, variant))
            })
            .collect();
        let keys = ["filament_vendor", "filament_type", "compatible_printers"];
        let mut filaments: Vec<GuideFilament> = catalog
            .selectable_positions(ProfileKind::Filament)
            .into_iter()
            .filter(|position| {
                !catalog.is_user_scope(*position) && catalog.missing_parent(*position).is_none()
            })
            .filter_map(|position| {
                let mut values = catalog.resolved_values(position, &keys).ok()?;
                let mut strings = |key: &str| strings(values.remove(key));
                let first = |values: Vec<String>| values.into_iter().next().unwrap_or_default();
                Some(GuideFilament {
                    name: catalog.document(position).name.clone(),
                    vendor: first(strings("filament_vendor")),
                    filament_type: first(strings("filament_type")),
                    models: strings("compatible_printers")
                        .iter()
                        .filter_map(|printer| machines.get(printer).cloned())
                        .collect(),
                })
            })
            .collect();
        filaments.sort_by(|a, b| a.name.cmp(&b.name));
        filaments
    }
}

/// A resolved coStrings value: the document's list, or the option default's serialized text.
fn strings(value: Option<RawConfigValue>) -> Vec<String> {
    match value {
        Some(RawConfigValue::List(items)) => items
            .into_iter()
            .filter_map(|item| match item {
                RawConfigValue::Text(text) => Some(text),
                _ => None,
            })
            .collect(),
        Some(RawConfigValue::Text(text)) => {
            crate::options::unescape_strings_cstyle(&text).unwrap_or_default()
        }
        _ => Vec::new(),
    }
}
