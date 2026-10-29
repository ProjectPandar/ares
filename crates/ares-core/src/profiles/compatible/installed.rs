//! `PresetBundle::load_installed_filaments` (`PresetBundle.cpp:2541-2600`):
//! the installed filament set, seeded from printer model default materials.

use std::collections::BTreeSet;

use super::{Candidate, Catalog, active_printer, compatible_with_printer, library_exclusions};
use crate::SliceError;
use crate::profiles::ProfileKind;

/// `installed` (the AppConfig `filaments` section) plus, for every vendor
/// printer with models that no installed filament is compatible with, the
/// system presets named by its printer model's `default_materials`
/// (`:2550-2590`), sorted as the section's `std::map`. All vendor printers
/// count as visible: the catalog's vendors are the installed set. Names
/// that are not loaded are kept, as the section keeps them.
pub(in crate::profiles) fn installed_filaments(
    catalog: &Catalog,
    installed: &[String],
) -> Result<Vec<String>, SliceError> {
    let excluded = library_exclusions(catalog);
    // `:2557-2558`: `find_preset` of each installed name; unloaded names never match.
    let candidates = installed
        .iter()
        .filter_map(|name| catalog.selectable_position(ProfileKind::Filament, name))
        .filter(|position| catalog.missing_parent(*position).is_none())
        .map(|position| Candidate::load(catalog, position))
        .collect::<Result<Vec<_>, _>>()?;
    let mut section: BTreeSet<String> = installed.iter().cloned().collect();
    for position in catalog.selectable_positions(ProfileKind::Machine) {
        if catalog.missing_parent(position).is_some() || !catalog.has_vendor_models(position) {
            continue;
        }
        let printer = active_printer(catalog, position)?;
        // `:2559`: `is_compatible_with_printer`; its diagnostics are not reported here.
        let mut diagnostics = Vec::new();
        if candidates.iter().any(|candidate| {
            compatible_with_printer(candidate, &printer, &excluded, &mut diagnostics)
        }) {
            continue;
        }
        let Some(model) = catalog.system_printer_model(position)? else {
            continue;
        };
        // `:2579-2584`: only system presets of the default materials are added.
        section.extend(
            model
                .default_materials
                .iter()
                .filter(|name| {
                    catalog
                        .selectable_position(ProfileKind::Filament, name)
                        .is_some_and(|filament| !catalog.is_user_scope(filament))
                })
                .cloned(),
        );
    }
    Ok(section.into_iter().collect())
}
