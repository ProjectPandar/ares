//! Preset compatibility filtering: a byte-oriented port of
//! `Preset.cpp:771-851` (`is_compatible_with_printer` /
//! `is_compatible_with_print`), `Preset.cpp:843-850` (`printer_preset` /
//! `num_extruders` extras) and
//! `PresetCollection::update_library_profile_excluded_from`
//! (`Preset.cpp:3705-3736`). Only the filtering decision and its
//! source-defined error-to-true diagnostics are ported; selection,
//! unselection and visibility bookkeeping are upstream UI state and stay
//! unported.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::catalog::Catalog;
use super::fragment::ProfileKind;
use super::fragment::metadata::parse_values;
use crate::SliceError;
use crate::project::raw_settings::RawConfigValue;
use crate::project_slice::gcode_emit::{
    expression::evaluate,
    value::{Config, Value},
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
    let mut printer_config = expression_config(catalog, printer_position)?;
    // Preset.cpp:843-850: printer_preset plus the extruder count derived from
    // the active printer's own merged nozzle_diameter list.
    printer_config.insert("printer_preset", Value::String(printer.to_owned()));
    if let Some(Value::List(diameters)) = printer_config.get("nozzle_diameter") {
        printer_config.insert("num_extruders", Value::Integer(diameters.len() as i64));
    }
    let printer = ActivePrinter {
        name: printer.to_owned(),
        is_system: !catalog.is_user_scope(printer_position),
        inherits: catalog.leaf_inherits(printer_position),
        config: printer_config,
    };
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

struct ActivePrinter {
    name: String,
    is_system: bool,
    /// The active printer preset's own `inherits` config value
    /// (`Preset::inherits` reads the stored option, `Preset.hpp:320-321`).
    inherits: Option<String>,
    config: Config,
}

struct ActivePrint {
    name: String,
    config: Config,
}

struct Candidate {
    kind: ProfileKind,
    position: usize,
    name: String,
    vendor: String,
    /// `PresetBundle.cpp:4943-4944,5086-5101`: explicit `alias` field, else
    /// the bare preset name trimmed before the first `@`.
    alias: String,
    compatible: super::fragment::metadata::ProfileConfigMetadataPatch,
}

impl Candidate {
    fn load(catalog: &Catalog, position: usize) -> Result<Self, SliceError> {
        let mut compatible = super::fragment::metadata::ProfileConfigMetadataPatch::default();
        let mut leaf = None;
        let mut alias = None;
        for member in catalog.chain_positions(position)? {
            let document = catalog.document(member);
            let parsed =
                parse_values(RawConfigValue::Object(document.values())).map_err(|error| {
                    invalid(format!(
                        "profile '{}' metadata is invalid: {error}",
                        document.name
                    ))
                })?;
            compatible.overlay_compatibility(&parsed.config);
            // The alias option follows the merged config, not only the leaf.
            if let Some(value) = document
                .fields
                .get("alias")
                .and_then(raw_text)
                .filter(|value| !value.is_empty())
            {
                alias = Some(value.to_owned());
            }
            leaf = Some((member, parsed));
        }
        let (member, parsed) = leaf.expect("chain always ends with its selected record");
        let alias = alias.or_else(|| {
            let bare = parsed
                .preset
                .name
                .rsplit('/')
                .next()
                .unwrap_or(&parsed.preset.name);
            let derived = bare.split('@').next().unwrap_or(bare).trim_end();
            Some(if derived.is_empty() {
                bare.to_owned()
            } else {
                derived.to_owned()
            })
        });
        Ok(Self {
            kind: parsed.preset.kind,
            position,
            name: parsed.preset.name.clone(),
            vendor: catalog.vendor_name(member).to_owned(),
            alias: alias.unwrap_or_default(),
            compatible,
        })
    }
}

/// `Preset.cpp:3705-3736`: Orca Filament Library presets with an empty
/// `compatible_printers` list are excluded from the printers claimed by a
/// same-alias preset in a concrete vendor.
fn library_exclusions(catalog: &Catalog) -> BTreeMap<usize, BTreeSet<String>> {
    let mut by_alias: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for position in catalog.selectable_positions(ProfileKind::Filament) {
        let Ok(candidate) = Candidate::load(catalog, position) else {
            continue;
        };
        if candidate.vendor == ORCA_FILAMENT_LIBRARY
            && candidate
                .compatible
                .compatible_printers
                .as_ref()
                .is_none_or(|values| values.is_empty())
        {
            by_alias
                .entry(candidate.alias.clone())
                .or_default()
                .push(position);
        }
    }
    let mut excluded: BTreeMap<usize, BTreeSet<String>> = BTreeMap::new();
    for position in catalog.selectable_positions(ProfileKind::Filament) {
        let Ok(candidate) = Candidate::load(catalog, position) else {
            continue;
        };
        if candidate.vendor == ORCA_FILAMENT_LIBRARY {
            continue;
        }
        let Some(printers) = candidate
            .compatible
            .compatible_printers
            .as_ref()
            .filter(|values| !values.is_empty())
        else {
            continue;
        };
        if let Some(targets) = by_alias.get(&candidate.alias) {
            for target in targets {
                excluded
                    .entry(*target)
                    .or_default()
                    .extend(printers.iter().cloned());
            }
        }
    }
    excluded
}

/// `Preset.cpp:809-840` `is_compatible_with_printer`.
fn compatible_with_printer(
    candidate: &Candidate,
    printer: &ActivePrinter,
    excluded: &BTreeMap<usize, BTreeSet<String>>,
    diagnostics: &mut Vec<CompatibilityDiagnostic>,
) -> bool {
    // Preset.cpp:814-821: filament-library exclusions check the active
    // printer's name and its immediate inherits parent.
    if candidate.kind == ProfileKind::Filament
        && candidate.vendor == ORCA_FILAMENT_LIBRARY
        && let Some(printers) = excluded.get(&candidate.position)
    {
        let parent = printer.inherits.as_deref();
        if printers.contains(&printer.name)
            || parent.is_some_and(|parent| printers.contains(parent))
        {
            return false;
        }
    }
    let compatible_printers = candidate.compatible.compatible_printers.as_ref();
    let has_list = compatible_printers.is_some_and(|values| !values.is_empty());
    let condition = candidate
        .compatible
        .compatible_printers_condition
        .as_deref()
        .unwrap_or_default();
    if !has_list && !condition.is_empty() {
        // Preset.cpp:830-836: a parse/eval error keeps the preset compatible.
        return match evaluate(condition, &printer.config, 0) {
            Ok(value) => value.as_bool(),
            Err(message) => {
                diagnostics.push(CompatibilityDiagnostic {
                    kind: kind_name(candidate.kind),
                    name: candidate.name.clone(),
                    message,
                });
                true
            }
        };
    }
    // Preset.cpp:837-840: catalog presets are never collection defaults and
    // the active printer name is never empty, so the upstream
    // `is_default || name.empty()` cases cannot fire here. Non-system active
    // printers also match against their immediate parent (`:799-806`).
    !has_list
        || compatible_printers.is_some_and(|values| {
            values.iter().any(|value| value == &printer.name)
                || (!printer.is_system
                    && printer
                        .inherits
                        .as_deref()
                        .is_some_and(|parent| values.iter().any(|value| value == parent)))
        })
}

/// `Preset.cpp:771-792` `is_compatible_with_print`; skipped without an active
/// process (`Preset.cpp:3364-3365`).
fn compatible_with_print(
    candidate: &Candidate,
    process: Option<&ActivePrint>,
    diagnostics: &mut Vec<CompatibilityDiagnostic>,
) -> bool {
    let Some(process) = process else {
        return true;
    };
    let compatible_prints = candidate.compatible.compatible_prints.as_ref();
    let has_list = compatible_prints.is_some_and(|values| !values.is_empty());
    let condition = candidate
        .compatible
        .compatible_prints_condition
        .as_deref()
        .unwrap_or_default();
    if !has_list && !condition.is_empty() {
        return match evaluate(condition, &process.config, 0) {
            Ok(value) => value.as_bool(),
            Err(message) => {
                diagnostics.push(CompatibilityDiagnostic {
                    kind: kind_name(candidate.kind),
                    name: candidate.name.clone(),
                    message,
                });
                true
            }
        };
    }
    !has_list
        || compatible_prints.is_some_and(|values| values.iter().any(|value| value == &process.name))
}

const fn kind_name(kind: ProfileKind) -> &'static str {
    match kind {
        ProfileKind::Process => "process",
        ProfileKind::Filament => "filament",
        ProfileKind::Machine => "machine",
    }
}

/// The active preset's merged fields as an expression `Config`.
fn expression_config(catalog: &Catalog, position: usize) -> Result<Config, SliceError> {
    let mut config = Config::default();
    for member in catalog.chain_positions(position)? {
        for (key, value) in catalog.document(member).values() {
            if let Some(value) = expression_value(&value) {
                config.insert(key, value);
            }
        }
    }
    Ok(config)
}

fn expression_value(value: &RawConfigValue) -> Option<Value> {
    match value {
        RawConfigValue::Text(text) => Some(Value::String(text.clone())),
        RawConfigValue::Number(number) => number
            .as_i64()
            .map(Value::Integer)
            .or_else(|| number.as_f64().map(Value::Number)),
        RawConfigValue::Flag(flag) => Some(Value::OptionBool(*flag)),
        RawConfigValue::List(values) => Some(Value::List(
            values.iter().filter_map(expression_value).collect(),
        )),
        RawConfigValue::Object(_) | RawConfigValue::Null => None,
    }
}

fn raw_text(value: &RawConfigValue) -> Option<&str> {
    match value {
        RawConfigValue::Text(text) => Some(text),
        _ => None,
    }
}

fn invalid(message: impl Into<String>) -> SliceError {
    SliceError::InvalidInput(message.into())
}
