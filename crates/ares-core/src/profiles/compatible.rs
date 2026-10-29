//! Preset compatibility filtering: a byte-oriented port of
//! `Preset.cpp:771-851` (`is_compatible_with_printer` /
//! `is_compatible_with_print`), `Preset.cpp:843-850` (`printer_preset` /
//! `num_extruders` extras) and
//! `PresetCollection::update_library_profile_excluded_from`
//! (`Preset.cpp:3705-3736`). Only the filtering decision and its
//! source-defined error-to-true diagnostics are ported; selection,
//! unselection and visibility bookkeeping are upstream UI state and stay
//! unported.

use serde::Serialize;

mod checks;
mod installed;

pub(super) use installed::installed_filaments;

use super::catalog::Catalog;
use super::fragment::ProfileKind;
use crate::SliceError;
use checks::{
    ActivePrint, Candidate, active_printer, compatible_with_print, compatible_with_printer,
    expression_config, library_exclusions,
};

pub(in crate::profiles) const ORCA_FILAMENT_LIBRARY: &str = "OrcaFilamentLibrary";

/// Filtering result serialized by the WASM adapter into the transport
/// envelope `{processes, filaments, diagnostics, missing_parents}`.
#[derive(Debug, Serialize)]
pub struct FilteredPresets {
    processes: Vec<String>,
    filaments: Vec<String>,
    diagnostics: Vec<CompatibilityDiagnostic>,
    missing_parents: Vec<MissingParent>,
}

/// A preset whose `inherits` cannot be resolved; upstream skips such
/// presets at load (`Preset.cpp:1685-1691`, `PresetBundle.cpp:1521-1525`),
/// so they never appear in the filtering results.
#[derive(Debug, Serialize)]
pub struct MissingParent {
    kind: &'static str,
    name: String,
    parent: String,
}

/// Source-defined diagnostic for `Preset.cpp:781-787,830-836`: an expression
/// that fails to evaluate keeps the preset compatible and reports here.
#[derive(Debug, Serialize)]
pub struct CompatibilityDiagnostic {
    kind: &'static str,
    name: String,
    message: String,
}

/// Filter the catalog's selectable process and filament presets against the
/// named printer (and optionally the named process), returning the upstream
/// decision plus diagnostics for expressions that failed to evaluate.
pub fn filter_compatible_presets(
    catalog_json: &str,
    printer: &str,
    process: Option<&str>,
) -> Result<FilteredPresets, SliceError> {
    let catalog = Catalog::from_json(catalog_json)?;
    filter_with_catalog(&catalog, printer, process)
}

pub(super) fn filter_with_catalog(
    catalog: &Catalog,
    printer: &str,
    process: Option<&str>,
) -> Result<FilteredPresets, SliceError> {
    let printer_position =
        selectable_without_missing_parent(catalog, ProfileKind::Machine, printer)?;
    let printer = active_printer(catalog, printer_position)?;
    let process = process
        .map(|name| {
            let position = selectable_without_missing_parent(catalog, ProfileKind::Process, name)?;
            Ok(ActivePrint {
                name: name.to_owned(),
                config: expression_config(catalog, position)?,
            })
        })
        .transpose()?;

    let excluded = library_exclusions(catalog);
    let mut result = FilteredPresets {
        processes: Vec::new(),
        filaments: Vec::new(),
        diagnostics: Vec::new(),
        missing_parents: Vec::new(),
    };
    for position in catalog.selectable_positions(ProfileKind::Process) {
        // Preset.cpp:1685-1691: an unresolvable inherits link keeps the
        // preset out of the collection, so it cannot be filtered.
        if let Some(link) = catalog.missing_parent(position) {
            push_missing_parent(&mut result.missing_parents, &link);
            continue;
        }
        let candidate = Candidate::load(catalog, position)?;
        if compatible_with_printer(&candidate, &printer, &excluded, &mut result.diagnostics) {
            result.processes.push(candidate.name);
        }
    }
    for position in catalog.selectable_positions(ProfileKind::Filament) {
        if let Some(link) = catalog.missing_parent(position) {
            push_missing_parent(&mut result.missing_parents, &link);
            continue;
        }
        let candidate = Candidate::load(catalog, position)?;
        if compatible_with_printer(&candidate, &printer, &excluded, &mut result.diagnostics)
            && compatible_with_print(&candidate, process.as_ref(), &mut result.diagnostics)
        {
            result.filaments.push(candidate.name);
        }
    }
    Ok(result)
}

/// Selected printer/process lookup: a preset whose inheritance chain has a
/// missing link is effectively absent (upstream skips it at load,
/// `Preset.cpp:1685-1691`), and the error names the missing parent.
fn selectable_without_missing_parent(
    catalog: &Catalog,
    kind: ProfileKind,
    name: &str,
) -> Result<usize, SliceError> {
    let position = catalog
        .selectable_position(kind, name)
        .ok_or_else(|| invalid(format!("{} preset '{name}' was not found", kind_str(kind))))?;
    if let Some(link) = catalog.missing_parent(position) {
        return Err(invalid(format!(
            "{} preset '{name}' is missing parent '{}'",
            kind_str(kind),
            link.parent
        )));
    }
    Ok(position)
}

fn kind_str(kind: ProfileKind) -> &'static str {
    match kind {
        ProfileKind::Process => "process",
        ProfileKind::Filament => "filament",
        ProfileKind::Machine => "printer",
    }
}

fn push_missing_parent(missing: &mut Vec<MissingParent>, link: &super::catalog::MissingParentLink) {
    let kind = match link.kind {
        ProfileKind::Process => "process",
        ProfileKind::Filament => "filament",
        ProfileKind::Machine => "machine",
    };
    if missing
        .iter()
        .any(|entry| entry.kind == kind && entry.name == link.name && entry.parent == link.parent)
    {
        return;
    }
    missing.push(MissingParent {
        kind,
        name: link.name.clone(),
        parent: link.parent.clone(),
    });
}

fn invalid(message: impl Into<String>) -> SliceError {
    SliceError::InvalidInput(message.into())
}
