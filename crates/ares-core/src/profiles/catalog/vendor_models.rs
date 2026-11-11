//! `VendorProfile::PrinterModel` loading from a vendor's
//! `machine_model_list` (`PresetBundle.cpp:4712-4821`).

use super::ProfileDocument;
use super::document::VendorModelEntry;
use crate::project::raw_settings::RawConfigValue;

/// The `VendorProfile::PrinterModel` fields the sidebar reads.
pub(in crate::profiles) struct PrinterModelProfile {
    pub(in crate::profiles) id: String,
    pub(in crate::profiles) name: String,
    /// `model_id`, the printer type of `Preset::get_printer_type`.
    pub(in crate::profiles) model_id: String,
    /// `VendorProfile::PrinterModel::variants`, the distinct `nozzle_diameter` entries.
    pub(in crate::profiles) variants: Vec<String>,
    pub(in crate::profiles) default_materials: Vec<String>,
    pub(in crate::profiles) not_support_bed_types: Vec<String>,
}

/// Parses one machine model file; `None` when the model has no id or no
/// nozzle variant, which upstream does not add (`PresetBundle.cpp:4819-4820`).
pub(super) fn printer_model(
    entry: &VendorModelEntry,
    document: &ProfileDocument,
) -> Option<PrinterModelProfile> {
    // `:4728-4813`: keys are matched with `boost::iequals`.
    let text = |key: &str| {
        document
            .fields
            .iter()
            .find(|(field, _)| field.eq_ignore_ascii_case(key))
            .and_then(|(_, value)| match value {
                RawConfigValue::Text(text) => Some(text.as_str()),
                _ => None,
            })
    };
    // `:4739-4752`: one variant per distinct `nozzle_diameter` entry.
    let mut variants: Vec<String> = Vec::new();
    for variant in strings(text("nozzle_diameter")) {
        if !variants.contains(&variant) {
            variants.push(variant);
        }
    }
    if entry.name.is_empty() || variants.is_empty() {
        return None;
    }
    Some(PrinterModelProfile {
        id: entry.name.clone(),
        // `:4731-4734`: the `name` key.
        name: document.name.clone(),
        // `:4735-4738`: the `model_id` key.
        model_id: text("model_id").unwrap_or_default().to_owned(),
        variants,
        // `:4786-4797` (`default_materials`) and `:4798-4809` (`not_support_bed_type`).
        default_materials: sorted_list(text("default_materials")),
        not_support_bed_types: sorted_list(text("not_support_bed_type")),
    })
}

/// `unescape_strings_cstyle` (`Config.cpp:146`); a malformed list is logged
/// upstream and leaves no values here.
fn strings(value: Option<&str>) -> Vec<String> {
    value
        .and_then(|value| crate::options::unescape_strings_cstyle(value).ok())
        .unwrap_or_default()
}

/// `sort_remove_duplicates` and the leading empty entry erase
/// (`PresetBundle.cpp:4790-4794`, `:4802-4806`).
fn sorted_list(value: Option<&str>) -> Vec<String> {
    let mut values = strings(value);
    values.sort();
    values.dedup();
    if values.first().is_some_and(String::is_empty) {
        values.remove(0);
    }
    values
}
