//! `--load-settings`/`--load-filaments` composition — the single upstream
//! merge shared with the catalog/profile-entry path.
//!
//! Upstream boundary: per-file validation and the machine/process/filament
//! loops live in the `load_config_file` lambda and its callers
//! (`OrcaSlicer.cpp:2205-2452`); typed files without inherits bypass preset
//! resolution at `:2121-2123`. The merge itself is `update_full_config`
//! and the filament resize/`set_at` contract (`:2826-3337`), which
//! `Catalog::compose` already ports. This module only ingests the CLI
//! preset files as user-scope documents and selects them by name — no
//! second composition exists.

mod project;

use super::ProfileSelection;
use super::catalog::{Catalog, ComposedSelection, ProfileDocument};
use crate::{SliceError, project::raw_settings::RawConfigValue};

/// One `--load-settings`/`--load-filaments` list entry: the file label
/// (path, for upstream error text) and its JSON text. An empty JSON
/// marks an empty list slot.
#[derive(Clone, Copy, Debug)]
pub struct LoadSettingsFile<'a> {
    pub label: &'a str,
    pub json: &'a str,
}

/// The parsed halves of one list, preserving upstream ordering
/// (`OrcaSlicer.cpp:1344,:1352` read each list once, `;`-separated).
pub struct LoadSettingsRequest<'a> {
    pub settings: &'a [LoadSettingsFile<'a>],
    pub filaments: &'a [LoadSettingsFile<'a>],
}

fn invalid(message: String) -> SliceError {
    SliceError::InvalidInput(message)
}

pub(crate) fn export_settings(
    request: &LoadSettingsRequest<'_>,
) -> Result<crate::project::raw_settings::ProjectSettingsRaw, SliceError> {
    use crate::project::raw_settings::RawConfigValue;

    let mut raw = compose_load_settings(request)?.raw;
    // CLI::run inserts selected machine/process identity and the process's
    // compatibility list separately from update_full_config (:2304,:2344-2346,:3381).
    for file in request.settings {
        let (kind, document) = parse_file(file)?;
        if document
            .inherits
            .as_ref()
            .is_some_and(|value| !value.is_empty())
        {
            return Err(invalid("3MF export requires flattened presets".to_owned()));
        }
        match kind.as_str() {
            "machine" => raw.insert(
                "printer_settings_id".into(),
                RawConfigValue::Text(document.name),
            ),
            "process" => {
                raw.insert(
                    "print_settings_id".into(),
                    RawConfigValue::Text(document.name),
                );
                raw.insert(
                    "print_compatible_printers".into(),
                    document
                        .fields
                        .get("compatible_printers")
                        .cloned()
                        .unwrap_or_else(|| RawConfigValue::List(Vec::new())),
                );
            }
            _ => {
                return Err(invalid(
                    "3MF export requires one machine and one process preset".to_owned(),
                ));
            }
        }
    }
    let (_, filament) = parse_file(&request.filaments[0])?;
    raw.insert_strings(
        "filament_ids",
        &[super::catalog::document_text(&filament, "filament_id")],
    );
    raw.insert_strings("filament_settings_id", &[filament.name]);
    Ok(raw)
}

/// Parses and validates one preset file exactly like the `load_config_file`
/// lambda (`OrcaSlicer.cpp:2205-2260`): `from` must be `system|User|user`
/// and `type` must be a known kind.
fn parse_file(file: &LoadSettingsFile<'_>) -> Result<(String, ProfileDocument), SliceError> {
    let document: ProfileDocument = serde_json::from_str(file.json).map_err(|error| {
        invalid(format!(
            "Loading setting file \"{}\" failed: {error}",
            file.label
        ))
    })?;
    if let Some(from) = document.fields.get("from").and_then(|value| value.token())
        && !matches!(from.as_str(), "system" | "User" | "user")
    {
        return Err(invalid(format!(
            "CLI::run:file {}'s from {from} unsupported",
            file.label
        )));
    }
    // `ProfileKind::parse` rejects unknown `type` values with the
    // catalog's own text; the load-settings loop's message differs only
    // in phrasing (`:1999`), pinned by the error-parity controls.
    Ok((document.kind.clone(), document))
}

/// Admit resolved presets for STL export without requiring a slicing selection.
pub fn validate_stl_export_presets(request: &LoadSettingsRequest<'_>) -> Result<(), SliceError> {
    let names = load_settings_preset_names(request)?;
    let unresolved = || invalid("STL export requires resolved presets".to_owned());
    for preset in names.machine.into_iter().chain(names.process) {
        if preset.inherits.is_some_and(|parent| !parent.is_empty()) {
            return Err(unresolved());
        }
    }
    for file in request.filaments {
        let (kind, document) = parse_file(file)?;
        if kind != "filament" {
            return Err(invalid("expected a filament preset".to_owned()));
        }
        if document.inherits.is_some_and(|parent| !parent.is_empty()) {
            return Err(unresolved());
        }
    }
    Ok(())
}

/// Composes the loaded files through the single catalog composition
/// (`Catalog::compose`), mirroring the upstream merge order: machine,
/// default printer G-codes, process, per-slot filaments with their own
/// raws (`OrcaSlicer.cpp:2849-2867,2993-3011,3150`).
///
/// Duplicate machine/process files are rejected like
/// `OrcaSlicer.cpp:2036-2040,2073-2077`. Empty filament slots are
/// dropped (`:2202-2215` with `use_first_fila_as_default` false, the
/// CLI default at `:1438,:1474-1476`).
pub fn compose_load_settings(
    request: &LoadSettingsRequest<'_>,
) -> Result<ComposedSelection, SliceError> {
    compose_with_project(request, None)
}

pub(crate) fn compose_project_load_settings(
    input: &[u8],
    request: &LoadSettingsRequest<'_>,
) -> Result<
    (
        ComposedSelection,
        crate::project_slice::compatibility::LoadedPresetNames,
    ),
    SliceError,
> {
    let mut names = load_settings_preset_names(request)?;
    names.embedded = project::retained_config(input, request, &names)?;
    let composed = match names.embedded.as_ref() {
        Some(embedded) => compose_with_project(request, Some(embedded))?,
        None => compose_load_settings(request)?,
    };
    Ok((composed, names))
}

fn compose_with_project(
    request: &LoadSettingsRequest<'_>,
    embedded: Option<&crate::project::raw_settings::ProjectSettingsRaw>,
) -> Result<ComposedSelection, SliceError> {
    let mut catalog = Catalog::empty();
    catalog.enable_direct_imports();
    let mut machine: Option<String> = None;
    let mut process: Option<String> = None;
    for file in request.settings {
        let (kind, document) = parse_file(file)?;
        match kind.as_str() {
            "machine" => {
                if machine.is_some() {
                    return Err(invalid(format!(
                        "duplicate machine config file: {}",
                        file.label
                    )));
                }
                machine = Some(document.name.clone());
                catalog.push_import(document)?;
            }
            "process" => {
                if process.is_some() {
                    return Err(invalid(format!(
                        "duplicate process config file: {}",
                        file.label
                    )));
                }
                process = Some(document.name.clone());
                catalog.push_import(document)?;
            }
            // A filament-typed file inside `--load-settings` validates but
            // merges nothing in the upstream loop (`:2025-2106` has no
            // filament assignment; only `--load-filaments` slots merge).
            _ => {}
        }
    }
    let mut filament_slots = Vec::new();
    for (slot, file) in request.filaments.iter().enumerate() {
        if file.json.is_empty() {
            continue;
        }
        let (kind, document) = parse_file(file)?;
        if kind != "filament" {
            return Err(invalid(format!(
                "CLI::run: unknown config type {kind} of file {} in load-filaments",
                file.label
            )));
        }
        filament_slots.push(catalog.push_filament_slot(document, slot)?);
    }
    let selection = match embedded {
        Some(config) => ProfileSelection::machine_for_project(machine.unwrap(), config)?,
        None => ProfileSelection::new_partial(process, machine, filament_slots)?,
    };
    catalog.compose(&selection)
}

/// Extracts the adjudication inputs of the `--load-settings` file loop
/// (`OrcaSlicer.cpp:2034-2098`): the first machine and process
/// descriptors (`config_name`/`from`/`inherits`/`compatible_printers`).
/// Reuses `parse_file` so per-file validation errors are identical to
/// `compose_load_settings`; duplicate-kind lists surface the same
/// duplicate error (`:2035-2038,:2074-2078`).
pub(crate) fn load_settings_preset_names(
    request: &LoadSettingsRequest<'_>,
) -> Result<crate::project_slice::compatibility::LoadedPresetNames, SliceError> {
    use crate::project_slice::compatibility::LoadedPresetNames;
    let mut machine = None;
    let mut process = None;
    let mut machine_seen = false;
    let mut process_seen = false;
    for file in request.settings {
        let (kind, document) = parse_file(file)?;
        match kind.as_str() {
            "machine" => {
                if machine_seen {
                    return Err(invalid(format!(
                        "duplicate machine config file: {}",
                        file.label
                    )));
                }
                machine_seen = true;
                machine = Some(descriptor(&document));
            }
            "process" => {
                if process_seen {
                    return Err(invalid(format!(
                        "duplicate process config file: {}",
                        file.label
                    )));
                }
                process_seen = true;
                process = Some(descriptor(&document));
            }
            _ => {}
        }
    }
    Ok(LoadedPresetNames {
        machine,
        process,
        embedded: None,
    })
}

fn descriptor(document: &ProfileDocument) -> crate::project_slice::compatibility::PresetDescriptor {
    crate::project_slice::compatibility::PresetDescriptor {
        name: document.name.clone(),
        from: document
            .fields
            .get("from")
            .and_then(|value| value.token())
            .unwrap_or_default(),
        inherits: document.inherits.clone(),
        compatible_printers: document
            .fields
            .get("compatible_printers")
            .map(|value| match value {
                RawConfigValue::List(values) => values
                    .iter()
                    .map(|item| match item {
                        RawConfigValue::Text(text) => text.clone(),
                        _ => String::new(),
                    })
                    .collect(),
                RawConfigValue::Text(text) => vec![text.clone()],
                _ => Vec::new(),
            })
            .unwrap_or_default(),
    }
}
