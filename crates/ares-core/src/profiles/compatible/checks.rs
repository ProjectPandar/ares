//! The compatibility decisions: `Preset.cpp:771-851` and the Orca Filament
//! Library exclusions (`Preset.cpp:3705-3736`).

use std::collections::{BTreeMap, BTreeSet};

use super::{CompatibilityDiagnostic, ORCA_FILAMENT_LIBRARY, invalid};
use crate::SliceError;
use crate::profiles::catalog::Catalog;
use crate::profiles::fragment::ProfileKind;
use crate::profiles::fragment::metadata::parse_values;
use crate::project::raw_settings::RawConfigValue;
use crate::project_slice::gcode_emit::{
    expression::evaluate,
    value::{Config, Value},
};

/// The printer at `position` as the active printer of a compatibility check.
pub(super) fn active_printer(
    catalog: &Catalog,
    position: usize,
) -> Result<ActivePrinter, SliceError> {
    let name = catalog.document(position).name.clone();
    let mut config = expression_config(catalog, position)?;
    // Preset.cpp:843-850: printer_preset plus the extruder count derived from
    // the active printer's own merged nozzle_diameter list.
    config.insert("printer_preset", Value::String(name.clone()));
    if let Some(Value::List(diameters)) = config.get("nozzle_diameter") {
        config.insert("num_extruders", Value::Integer(diameters.len() as i64));
    }
    Ok(ActivePrinter {
        name,
        is_system: !catalog.is_user_scope(position) && !catalog.is_default(position),
        inherits: catalog.leaf_inherits(position),
        config,
    })
}

pub(super) struct ActivePrinter {
    name: String,
    is_system: bool,
    /// The active printer preset's own `inherits` config value
    /// (`Preset::inherits` reads the stored option, `Preset.hpp:320-321`).
    inherits: Option<String>,
    config: Config,
}

pub(super) struct ActivePrint {
    pub(super) name: String,
    pub(super) config: Config,
}

pub(super) struct Candidate {
    kind: ProfileKind,
    position: usize,
    pub(super) name: String,
    vendor: String,
    /// `PresetBundle.cpp:4943-4944,5086-5101`: explicit `alias` field, else
    /// the bare preset name trimmed before the first `@`.
    pub(super) alias: String,
    compatible: crate::profiles::fragment::metadata::ProfileConfigMetadataPatch,
}

impl Candidate {
    pub(super) fn load(catalog: &Catalog, position: usize) -> Result<Self, SliceError> {
        let mut compatible =
            crate::profiles::fragment::metadata::ProfileConfigMetadataPatch::default();
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
pub(super) fn library_exclusions(catalog: &Catalog) -> BTreeMap<usize, BTreeSet<String>> {
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
pub(super) fn compatible_with_printer(
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
pub(super) fn compatible_with_print(
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
pub(super) fn expression_config(catalog: &Catalog, position: usize) -> Result<Config, SliceError> {
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
