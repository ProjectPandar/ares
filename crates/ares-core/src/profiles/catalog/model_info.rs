//! Printer model data of the plater printer panel: vendor type, bed types,
//! default materials and nozzle diameters.

use std::collections::BTreeSet;

use crate::profiles::ProfileKind;
use crate::project::raw_settings::RawConfigValue;

/// What the sidebar reads about a `printer_model`.
#[derive(Debug, serde::Serialize)]
pub struct PrinterModelInfo {
    /// Index name of the vendor whose models list the model by name
    /// (`PresetBundle::get_current_vendor_type`, `PresetBundle.cpp:612-640`);
    /// `None` when no loaded vendor has it.
    pub vendor: Option<String>,
    /// `VendorProfile::PrinterModel::not_support_bed_types` of the model
    /// with this id (`PresetUtils::system_printer_model`, `Preset.cpp:4448-4460`).
    pub not_support_bed_types: Vec<String>,
    /// `VendorProfile::PrinterModel::default_materials` of the same model.
    pub default_materials: Vec<String>,
    /// `PresetCollection::diameters_of_selected_printer` (`Preset.cpp:3319-3328`).
    pub nozzle_diameters: Vec<String>,
}

impl super::PresetCatalogHandle {
    /// The vendor and model data of `printer_model`. Upstream's vendors are a
    /// `std::map` keyed by vendor id and `system_printer_model` searches only
    /// the preset's own vendor; the catalog has no vendor ids and searches
    /// every vendor in load order, which agrees while model names and ids are
    /// unique across vendors, as in the OrcaSlicer profiles.
    pub fn printer_model_info(
        &self,
        printer_model: &str,
    ) -> Result<PrinterModelInfo, crate::SliceError> {
        let catalog = &self.catalog;
        let vendor = catalog
            .vendor_models
            .iter()
            .position(|models| models.iter().any(|model| model.name == printer_model))
            .map(|scope| catalog.vendor_names[scope].clone());
        let model = catalog
            .vendor_models
            .iter()
            .flatten()
            .find(|model| model.id == printer_model);
        Ok(PrinterModelInfo {
            vendor,
            not_support_bed_types: model
                .map(|model| model.not_support_bed_types.clone())
                .unwrap_or_default(),
            default_materials: model
                .map(|model| model.default_materials.clone())
                .unwrap_or_default(),
            nozzle_diameters: self.diameters_of_printer_model(printer_model)?,
        })
    }

    /// `Preset.cpp:3319-3328`: the `printer_variant` of every loaded printer
    /// preset with this `printer_model`, as a sorted set. Every loaded preset
    /// counts, including non-instantiable and user ones; a preset with a
    /// missing parent is not loaded (`Preset.cpp:1685-1691`).
    fn diameters_of_printer_model(
        &self,
        printer_model: &str,
    ) -> Result<Vec<String>, crate::SliceError> {
        let catalog = &self.catalog;
        let mut diameters = BTreeSet::new();
        for position in 0..catalog.records.len() {
            if catalog.records[position].kind != ProfileKind::Machine
                || catalog.missing_parent(position).is_some()
            {
                continue;
            }
            let values =
                catalog.resolved_values(position, &["printer_model", "printer_variant"])?;
            if text(&values["printer_model"]) == printer_model {
                diameters.insert(text(&values["printer_variant"]).to_owned());
            }
        }
        Ok(diameters.into_iter().collect())
    }
}

impl super::Catalog {
    /// `PresetUtils::system_printer_model` (`Preset.cpp:4448-4460`): the
    /// model of a vendor printer whose id is its `printer_model`, from its
    /// own vendor; `None` for user printers.
    pub(in crate::profiles) fn system_printer_model(
        &self,
        position: usize,
    ) -> Result<Option<&super::vendor_models::PrinterModelProfile>, crate::SliceError> {
        let super::Scope::Vendor(scope) = self.records[position].scope else {
            return Ok(None);
        };
        let values = self.resolved_values(position, &["printer_model"])?;
        let printer_model = text(&values["printer_model"]);
        Ok(self.vendor_models[scope]
            .iter()
            .find(|model| !printer_model.is_empty() && model.id == printer_model))
    }

    /// Whether the record's vendor has printer models (`printer.vendor &&
    /// !printer.vendor->models.empty()`, `PresetBundle.cpp:2551`).
    pub(in crate::profiles) fn has_vendor_models(&self, position: usize) -> bool {
        matches!(self.records[position].scope, super::Scope::Vendor(scope) if !self.vendor_models[scope].is_empty())
    }
}

fn text(value: &RawConfigValue) -> &str {
    match value {
        RawConfigValue::Text(text) => text,
        _ => "",
    }
}
