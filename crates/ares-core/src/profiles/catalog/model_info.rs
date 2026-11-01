//! Printer model data of the plater printer panel: vendor type, bed types,
//! default materials and nozzle diameters.

use std::collections::BTreeSet;

use crate::profiles::ProfileKind;
use crate::project::raw_settings::RawConfigValue;

/// What the sidebar reads about a `printer_model`.
#[derive(Debug, serde::Serialize)]
pub struct PrinterModelInfo {
    /// Id of the vendor whose models list the model by name, the `PresetBundle::vendors` key
    /// `get_current_vendor_type` compares (`PresetBundle.cpp:992-1024` at v2.5.0-beta
    /// `345efec64`); `None` when no loaded vendor has it.
    pub vendor: Option<String>,
    /// `VendorProfile::PrinterModel::not_support_bed_types` of the model
    /// with this id (`PresetUtils::system_printer_model`, `Preset.cpp:4448-4460`).
    pub not_support_bed_types: Vec<String>,
    /// `model_id` of the model with this name (`Preset::get_printer_type`,
    /// `Preset.cpp:889-905`), which `Preset::get_default_bed_type` tests
    /// (`Preset.cpp:985-990`); empty when no vendor lists the model.
    pub model_id: String,
    /// `VendorProfile::PrinterModel::default_materials` of the same model.
    pub default_materials: Vec<String>,
    /// `PresetCollection::diameters_of_selected_printer` (`Preset.cpp:3319-3328`).
    pub nozzle_diameters: Vec<String>,
}

/// `VendorType` (`Preset.hpp`), the vendor kinds `get_current_vendor_type` tells apart.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum VendorType {
    #[default]
    Unknown,
    MarlinBbl,
    KlipperQidi,
}

impl super::PresetCatalogHandle {
    /// `PresetBundle::get_current_vendor_type` (`PresetBundle.cpp:992-1024` at v2.5.0-beta
    /// `345efec64`) for `printer_model` among the installed vendors.
    pub(crate) fn current_vendor_type(&self, printer_model: &str) -> VendorType {
        match self.catalog.vendor_of_model_name(printer_model) {
            Some("BBL") => VendorType::MarlinBbl,
            Some("Qidi") => VendorType::KlipperQidi,
            _ => VendorType::Unknown,
        }
    }

    /// The vendor and model data of `printer_model`. The vendor is `get_current_vendor_type`'s
    /// lookup; `system_printer_model` searches only the preset's own vendor, while the model data
    /// here searches every vendor in load order, which agrees while model names and ids are unique
    /// across vendors, as in the OrcaSlicer profiles.
    pub fn printer_model_info(
        &self,
        printer_model: &str,
    ) -> Result<PrinterModelInfo, crate::SliceError> {
        let catalog = &self.catalog;
        let named = catalog
            .vendor_models
            .iter()
            .enumerate()
            .find_map(|(scope, models)| {
                models
                    .iter()
                    .find(|model| model.name == printer_model)
                    .map(|model| (scope, model))
            });
        let model = catalog
            .vendor_models
            .iter()
            .flatten()
            .find(|model| model.id == printer_model);
        Ok(PrinterModelInfo {
            vendor: catalog
                .vendor_of_model_name(printer_model)
                .map(str::to_owned),
            model_id: named
                .map(|(_, model)| model.model_id.clone())
                .unwrap_or_default(),
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
    /// The id of the first vendor, in `PresetBundle::vendors` order (a `std::map` by id), whose
    /// models list one named `printer_model` (`PresetBundle.cpp:1004-1014`).
    fn vendor_of_model_name(&self, printer_model: &str) -> Option<&str> {
        let mut scopes: Vec<usize> = (0..self.vendor_ids.len()).collect();
        scopes.sort_by(|&a, &b| self.vendor_ids[a].cmp(&self.vendor_ids[b]));
        scopes
            .into_iter()
            .find(|&scope| {
                self.vendor_models[scope]
                    .iter()
                    .any(|model| model.name == printer_model)
            })
            .map(|scope| self.vendor_ids[scope].as_str())
    }

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
