//! The `AppConfig` vendors section and printer visibility: `Preset::set_visible_from_appconfig`
//! (`Preset.cpp:967-995`) applied to every printer by `PresetBundle::load_installed_printers`
//! (`PresetBundle.cpp:3685-3690`), and
//! `PrinterPresetCollection::find_system_preset_by_model_and_variant` (`Preset.cpp:4569-4581`).

use std::collections::{BTreeMap, BTreeSet};

use super::{Catalog, PresetCatalogHandle};
use crate::SliceError;
use crate::profiles::ProfileKind;
use crate::project::raw_settings::RawConfigValue;

/// `AppConfig::VendorMap`: vendor id → printer model id → installed nozzle variants.
pub type InstalledPrinters = BTreeMap<String, BTreeMap<String, BTreeSet<String>>>;

impl Catalog {
    /// The printer's resolved `printer_model` and `printer_variant` (`preset.config`).
    pub(in crate::profiles) fn printer_identity(
        &self,
        position: usize,
    ) -> Result<(String, String), SliceError> {
        let mut values = self.resolved_values(position, &["printer_model", "printer_variant"])?;
        let mut text = |key: &str| match values.remove(key) {
            Some(RawConfigValue::Text(text)) => text,
            _ => String::new(),
        };
        Ok((text("printer_model"), text("printer_variant")))
    }

    /// `Preset::set_visible_from_appconfig` for a printer: a user preset (no vendor, `:971`) or a
    /// printer with an empty model or variant (`:976-977`) keeps its load-time visibility; any
    /// other is visible exactly when `AppConfig::get_variant(vendor->id, model, variant)` is set.
    /// Without a vendors section every printer stays visible.
    pub(in crate::profiles) fn is_visible_printer(
        &self,
        position: usize,
        (model, variant): (&str, &str),
    ) -> bool {
        let Some(installed) = &self.installed_printers else {
            return true;
        };
        let super::Scope::Vendor(scope) = self.records[position].scope else {
            return true;
        };
        model.is_empty()
            || variant.is_empty()
            || installed
                .get(&self.vendor_ids[scope])
                .and_then(|models| models.get(model))
                .is_some_and(|variants| variants.contains(variant))
    }

    /// Loaded printers in collection order with their resolved model and variant. Printers whose
    /// parent is missing never enter the collection (`Preset.cpp:1685-1691`).
    pub(in crate::profiles) fn printer_identities(
        &self,
    ) -> impl Iterator<Item = (usize, String, String)> + '_ {
        self.selectable_positions(ProfileKind::Machine)
            .into_iter()
            .filter(|position| self.missing_parent(*position).is_none())
            .filter_map(|position| {
                let (model, variant) = self.printer_identity(position).ok()?;
                Some((position, model, variant))
            })
    }
}

impl PresetCatalogHandle {
    /// Stores the `AppConfig` vendors section; the printer listings then follow
    /// `load_installed_printers`, and so does `load_installed_filaments`.
    pub fn set_installed_printers(&mut self, installed: InstalledPrinters) {
        self.catalog.installed_printers = Some(installed);
    }

    /// `find_system_preset_by_model_and_variant`: the first system printer of `model` whose
    /// variant is `variant`, or any variant when `variant` is empty. Visibility is not checked.
    pub fn system_printer(&self, model: &str, variant: &str) -> Option<String> {
        if model.is_empty() {
            return None;
        }
        self.catalog
            .printer_identities()
            .find(|(position, printer_model, printer_variant)| {
                !self.catalog.is_user_scope(*position)
                    && printer_model == model
                    && (variant.is_empty() || printer_variant == variant)
            })
            .map(|(position, ..)| self.catalog.document(position).name.clone())
    }
}
