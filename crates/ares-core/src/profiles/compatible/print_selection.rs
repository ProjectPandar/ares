//! The print preset kept or chosen for a printer: `PresetCollection::update_compatible` with
//! `PresetSelectCompatibleType::Always` (`Preset.hpp:877-885`, `Preset.cpp:3965-4010`) and
//! `first_compatible_idx` (`Preset.hpp:831-855`) under `PresetBundle::update_compatible`'s
//! `PreferedPrintProfileMatch` (`PresetBundle.cpp:8522-8572`, `:8635-8639`).

use super::checks::{Candidate, active_printer, compatible_with_printer, library_exclusions};
use super::{invalid, selectable_without_missing_parent};
use crate::SliceError;
use crate::profiles::catalog::Catalog;
use crate::profiles::fragment::ProfileKind;
use crate::project::raw_settings::RawConfigValue;

/// `PreferedPrintProfileMatch`: the previously selected print's alias and layer height, and the
/// printer's `default_print_profile`.
struct PreferedPrint {
    alias: String,
    layer_height: f64,
    default_print_profile: String,
}

impl PreferedPrint {
    /// `PreferedPrintProfileMatch::operator()` for a compatible, visible system or user print:
    /// an alias match always wins; otherwise 1 for the default profile, plus 1 and 1 for
    /// `is_visible`, times 10 for the previous layer height.
    fn quality(&self, (name, alias): (&str, &str), layer_height: f64) -> i32 {
        if !self.alias.is_empty() && self.alias == alias {
            return i32::MAX;
        }
        let quality = i32::from(name == self.default_print_profile) + 2;
        if self.layer_height > 0. && (layer_height - self.layer_height).abs() < 0.0005 {
            quality * 10
        } else {
            quality
        }
    }
}

/// `current` while it is a compatible print of `printer`, else the compatible print that
/// matches best, the first in collection order on a tie, or `None` where upstream falls back to
/// the "- default -" print. A `current` that is not a loaded print stands for that default
/// print, which every collection selects first (`Preset.cpp:1739`): no alias, and the
/// `PrintConfigDef` layer height.
pub(in crate::profiles) fn compatible_print(
    catalog: &Catalog,
    printer: &str,
    current: Option<&str>,
) -> Result<Option<String>, SliceError> {
    let printer_position =
        selectable_without_missing_parent(catalog, ProfileKind::Machine, printer)?;
    let active = active_printer(catalog, printer_position)?;
    let current = current.and_then(|name| {
        selectable_without_missing_parent(catalog, ProfileKind::Process, name).ok()
    });
    let printer_values = catalog.resolved_values(printer_position, &["default_print_profile"])?;
    let prefered = PreferedPrint {
        alias: match current {
            Some(position) => Candidate::load(catalog, position)?.alias,
            None => String::new(),
        },
        layer_height: match current {
            Some(position) => layer_height(catalog, position)?,
            None => default_layer_height()?,
        },
        default_print_profile: text(&printer_values["default_print_profile"]).to_owned(),
    };
    let excluded = library_exclusions(catalog);
    let mut best: Option<(i32, String)> = None;
    for position in catalog.selectable_positions(ProfileKind::Process) {
        if catalog.missing_parent(position).is_some() {
            continue;
        }
        let candidate = Candidate::load(catalog, position)?;
        if !compatible_with_printer(&candidate, &active, &excluded, &mut Vec::new()) {
            continue;
        }
        if current == Some(position) {
            return Ok(Some(candidate.name));
        }
        let quality = prefered.quality(
            (&candidate.name, &candidate.alias),
            layer_height(catalog, position)?,
        );
        if best.as_ref().is_none_or(|(best, _)| quality > *best) {
            best = Some((quality, candidate.name));
        }
    }
    Ok(best.map(|(_, name)| name))
}

fn layer_height(catalog: &Catalog, position: usize) -> Result<f64, SliceError> {
    number(&catalog.resolved_values(position, &["layer_height"])?["layer_height"])
}

fn default_layer_height() -> Result<f64, SliceError> {
    let definition = crate::options::registry::option_definition("layer_height")
        .ok_or_else(|| invalid("unknown option 'layer_height'"))?;
    number(&RawConfigValue::Text(definition.default_value.to_owned()))
}

fn number(value: &RawConfigValue) -> Result<f64, SliceError> {
    match value {
        RawConfigValue::Number(number) => number.as_f64(),
        RawConfigValue::Text(text) => text.trim().parse().ok(),
        _ => None,
    }
    .ok_or_else(|| invalid(format!("layer_height {value:?} is not a number")))
}

fn text(value: &RawConfigValue) -> &str {
    match value {
        RawConfigValue::Text(text) => text,
        _ => "",
    }
}
