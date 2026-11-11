//! Printer model grouping and similar-preset resolution
//! (`PresetComboBoxes.cpp:1150, 1243-1251, 1427`, `PresetBundle.cpp:3750-3782`).

use super::{ProfileDocument, ProfileKind, document_text};
use crate::project::raw_settings::RawConfigValue;

/// One deduplicated system printer model for the grouping list
/// (`PresetComboBoxes.cpp:1243-1251`).
#[derive(Debug, serde::Serialize)]
pub struct PrinterModel {
    pub model: String,
}

/// The selected printer as the caller holds it: upstream reads
/// `printers.get_selected_preset()`, which may be a project preset outside
/// the catalog (`PresetBundle.cpp:3752, 3755, 3765`).
#[derive(Debug, Clone, Copy)]
pub struct CurrentPrinter<'a> {
    pub name: &'a str,
    pub printer_variant: &'a str,
}

impl super::PresetCatalogHandle {
    /// System printer models in "System presets" group order
    /// (`PresetComboBoxes.cpp:1243-1251`): only `is_system` presets enter the
    /// model list — user printers stay in their own name-listed section.
    /// The group is a `std::map<wxString>` (`:1150`) appended in key order
    /// (`:1427`, no re-sort for group "System" at `:1352`); wxString compares
    /// code points, matching `String`'s `Ord`. Same visibility rules as
    /// `printers()`.
    pub fn printer_models(&self) -> Vec<PrinterModel> {
        self.catalog
            .printer_identities()
            .filter(|(position, model, variant)| {
                !self.catalog.is_user_scope(*position)
                    && self.catalog.is_visible_printer(*position, (model, variant))
            })
            .map(|(_, model, _)| model)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .map(|model| PrinterModel { model })
            .collect()
    }

    /// `PresetBundle::get_similar_printer_preset` (`PresetBundle.cpp:3750-3782`):
    /// resolves a model/variant pair to a concrete preset, preferring the
    /// current preset's alias with the variant substituted, then the first
    /// candidate whose `printer_variant` matches, then the first candidate
    /// in name order (the upstream `std::map` ordering). An empty
    /// `printer_variant` restricts candidates to system presets
    /// (`:3757-3759`). A current printer outside the catalog has no alias to
    /// prefer and no model to fall back to; its passed `printer_variant` is
    /// `printer_variant_old` (`:3755`).
    pub fn similar_printer_preset(
        &self,
        printer_model: &str,
        printer_variant: &str,
        current: CurrentPrinter<'_>,
    ) -> Option<String> {
        let current_document = self
            .catalog
            .selectable_position(ProfileKind::Machine, current.name)
            .map(|position| self.catalog.document(position));
        // `:3751-3754`: an empty model falls back to the current preset's.
        let printer_model = if printer_model.is_empty() {
            &document_text(current_document?, "printer_model")
        } else {
            printer_model
        };
        if printer_model.is_empty() {
            return None;
        }
        let variant_old = current.printer_variant;

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

        if let Some(document) = current_document
            .map(|document| {
                preferred_printer(
                    document,
                    current.name,
                    printer_model,
                    printer_variant,
                    variant_old,
                )
            })
            .and_then(|prefer_printer| candidates.get(&prefer_printer))
        {
            return Some(document.name.clone());
        }

        // `:3773-3781`: first candidate with a matching variant, else the
        // first in name order.
        let printer_variant = if printer_variant.is_empty() {
            variant_old
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

/// `PresetBundle.cpp:3765-3772`: the current preset's alias with the old
/// variant replaced; the alias derives from the explicit `alias` field, else
/// the bare preset name trimmed before `@`.
fn preferred_printer(
    document: &ProfileDocument,
    name: &str,
    printer_model: &str,
    printer_variant: &str,
    variant_old: &str,
) -> String {
    let bare = name.rsplit('/').next().unwrap_or(name);
    let derived = bare.split('@').next().unwrap_or(bare).trim_end();
    let prefer = document
        .fields
        .get("alias")
        .and_then(|value| match value {
            RawConfigValue::Text(text) if !text.is_empty() => Some(text.clone()),
            _ => None,
        })
        .unwrap_or_else(|| derived.to_owned());
    if !printer_variant.is_empty() {
        prefer.replace(variant_old, printer_variant)
    } else if let Some(at) = prefer.find(variant_old) {
        format!(
            "{printer_model} {variant_old}{}",
            &prefer[at + variant_old.len()..]
        )
    } else {
        prefer
    }
}
