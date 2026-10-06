//! Printer model grouping and similar-preset resolution
//! (`PresetComboBoxes.cpp:1243-1251`, `PresetBundle.cpp:3750-3782`).

use super::{ProfileKind, document_text};
use crate::project::raw_settings::RawConfigValue;

/// One deduplicated system printer model for the grouping list
/// (`PresetComboBoxes.cpp:1243-1251`).
#[derive(Debug, serde::Serialize)]
pub struct PrinterModel {
    pub model: String,
}

impl super::PresetCatalogHandle {
    /// System printer models deduplicated by first occurrence
    /// (`PresetComboBoxes.cpp:1243-1251`): only `is_system` presets enter the
    /// model list — user printers stay in their own name-listed section.
    /// Collection order, same visibility rules as `printers()`.
    pub fn printer_models(&self) -> Vec<PrinterModel> {
        let mut seen = std::collections::BTreeSet::new();
        self.catalog
            .selectable_positions(ProfileKind::Machine)
            .into_iter()
            .filter(|position| {
                !self.catalog.is_user_scope(*position)
                    && self.catalog.missing_parent(*position).is_none()
            })
            .filter_map(|position| {
                let model = document_text(self.catalog.document(position), "printer_model");
                seen.insert(model.clone()).then_some(PrinterModel { model })
            })
            .collect()
    }

    /// `PresetBundle::get_similar_printer_preset` (`PresetBundle.cpp:3750-3782`):
    /// resolves a model/variant pair to a concrete preset, preferring the
    /// current preset's alias with the variant substituted, then the first
    /// candidate whose `printer_variant` matches, then the first candidate
    /// in name order (the upstream `std::map` ordering). An empty
    /// `printer_variant` restricts candidates to system presets
    /// (`:3757-3759`).
    pub fn similar_printer_preset(
        &self,
        printer_model: &str,
        printer_variant: &str,
        current: &str,
    ) -> Option<String> {
        let current_position = self
            .catalog
            .selectable_position(ProfileKind::Machine, current)?;
        let current_document = self.catalog.document(current_position);
        // `:3751-3753`: an empty model falls back to the current preset's.
        let printer_model = if printer_model.is_empty() {
            &document_text(current_document, "printer_model")
        } else {
            printer_model
        };
        if printer_model.is_empty() {
            return None;
        }
        let variant_old = document_text(current_document, "printer_variant");

        // `:3758-3763`: candidates keyed by name (std::map ordering).
        let mut candidates = std::collections::BTreeMap::new();
        for position in self.catalog.selectable_positions(ProfileKind::Machine) {
            if printer_variant.is_empty() && self.catalog.is_user_scope(position) {
                continue;
            }
            let document = self.catalog.document(position);
            if document_text(document, "printer_model") == *printer_model {
                candidates.insert(document.name.clone(), document);
            }
        }
        if candidates.is_empty() {
            return None;
        }

        // `:3765-3772`: prefer the current preset's alias with the old
        // variant replaced; the alias derives from the explicit `alias`
        // field, else the bare preset name trimmed before `@`.
        let bare = current.rsplit('/').next().unwrap_or(current);
        let derived = bare.split('@').next().unwrap_or(bare).trim_end();
        let prefer = current_document
            .fields
            .get("alias")
            .and_then(|value| match value {
                RawConfigValue::Text(text) if !text.is_empty() => Some(text.clone()),
                _ => None,
            })
            .unwrap_or_else(|| derived.to_owned());
        let prefer_printer = if !printer_variant.is_empty() {
            prefer.replace(&variant_old, printer_variant)
        } else if let Some(at) = prefer.find(&variant_old) {
            format!(
                "{printer_model} {variant_old}{}",
                &prefer[at + variant_old.len()..]
            )
        } else {
            prefer
        };
        if let Some(document) = candidates.get(&prefer_printer) {
            return Some(document.name.clone());
        }

        // `:3773-3781`: first candidate with a matching variant, else the
        // first in name order.
        let printer_variant = if printer_variant.is_empty() {
            &variant_old
        } else {
            printer_variant
        };
        candidates
            .values()
            .find(|document| document_text(document, "printer_variant") == printer_variant)
            .map(|document| document.name.clone())
            .or_else(|| candidates.values().next().map(|d| d.name.clone()))
    }
}
