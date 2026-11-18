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

use super::catalog::{Catalog, ComposedSelection, ProfileDocument};
use super::{ProfileKind, ProfileSelection};
use crate::{
    InputFormat, SliceError,
    profiles::SourceIdentity,
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

/// One `--load-settings`/`--load-filaments` list entry: the file label
/// (path, for upstream error text) and its JSON text. An empty JSON
/// marks an empty list slot.
#[derive(Clone, Copy, Debug)]
pub struct LoadSettingsFile<'a> {
    pub label: &'a str,
    pub json: &'a str,
    pub source_identity: Option<&'a SourceIdentity>,
}

/// The parsed halves of one list, preserving upstream ordering
/// (`OrcaSlicer.cpp:1344,:1352` read each list once, `;`-separated).
pub struct LoadSettingsRequest<'a> {
    pub loaded_sources: Option<&'a super::catalog::LoadedPresetSources>,
    pub settings: &'a [LoadSettingsFile<'a>],
    pub filaments: &'a [LoadSettingsFile<'a>],
}

fn invalid(message: String) -> SliceError {
    SliceError::InvalidInput(message)
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
    let catalog = loaded_catalog(request)?;
    // Binary STL can be Unknown to the signature probe; the profile loader
    // sends every non-3MF input through STL parsing before adjudication.
    let direct_stl = InputFormat::detect(input) != InputFormat::ThreeMf;
    let mut names = load_settings_preset_names(request, &catalog, direct_stl)?;
    names.embedded = project::retained_config(input, request, &names)?;
    let composed = compose_with_project(request, names.embedded.as_ref(), catalog)?;
    Ok((composed, names))
}

fn loaded_catalog(request: &LoadSettingsRequest<'_>) -> Result<Catalog, SliceError> {
    let mut catalog = Catalog::empty();
    catalog.enable_direct_imports();
    if let Some(sources) = request.loaded_sources {
        sources.populate(&mut catalog)?;
    }
    Ok(catalog)
}

fn compose_with_project(
    request: &LoadSettingsRequest<'_>,
    embedded: Option<&crate::project::raw_settings::ProjectSettingsRaw>,
    mut catalog: Catalog,
) -> Result<ComposedSelection, SliceError> {
    let mut machine: Option<String> = None;
    let mut process: Option<String> = None;
    for file in request.settings {
        let (kind, document) = parse_file(file)?;
        let profile_kind = ProfileKind::parse(&kind)?;
        let document =
            catalog.admit_document(profile_kind, document, file.label, file.source_identity)?;
        match kind.as_str() {
            "machine" => {
                if machine.is_some() {
                    return Err(invalid(format!(
                        "duplicate machine config file: {}",
                        file.label
                    )));
                }
                machine = Some(document.name.clone());
                catalog.push_import_with_identity(document, file.source_identity)?;
            }
            "process" => {
                if process.is_some() {
                    return Err(invalid(format!(
                        "duplicate process config file: {}",
                        file.label
                    )));
                }
                process = Some(document.name.clone());
                catalog.push_import_with_identity(document, file.source_identity)?;
            }
            // A filament-typed file inside `--load-settings` validates but
            // merges nothing in the upstream loop (`:2025-2106` has no
            // filament assignment; only `--load-filaments` slots merge).
            _ => {}
        }
    }
    let mut filament_slots = Vec::new();
    let mut authored_user_ids = Vec::new();
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
        let inherited_user = document
            .inherits
            .as_deref()
            .is_some_and(|name| !name.is_empty())
            && matches!(document.fields.get("from"), Some(RawConfigValue::Text(from)) if matches!(from.as_str(), "User" | "user"));
        // OrcaSlicer.cpp:2238–2255 resolves config without replacing the per-file ID.
        authored_user_ids
            .push(inherited_user.then(|| super::catalog::document_text(&document, "filament_id")));
        let document = catalog.admit_document(
            ProfileKind::Filament,
            document,
            file.label,
            file.source_identity,
        )?;
        filament_slots.push(catalog.push_filament_slot(document, slot, file.source_identity)?);
    }
    let selection = match embedded {
        Some(config) => ProfileSelection::machine_for_project(machine.unwrap(), config)?,
        None => ProfileSelection::new_partial(process, machine, filament_slots)?,
    };
    let mut composed = catalog.compose(&selection)?;
    for (id, authored) in composed
        .profile
        .settings
        .project
        .gcode
        .filament_ids
        .0
        .iter_mut()
        .zip(authored_user_ids)
    {
        if let Some(authored) = authored {
            *id = authored;
        }
    }
    Ok(composed)
}

/// Extracts the adjudication inputs of the `--load-settings` file loop
/// (`OrcaSlicer.cpp:2034-2098`): the first machine and process
/// descriptors, and the supplied machine's own options for direct STL.
/// Reuses `parse_file` so per-file validation errors are identical to
/// `compose_load_settings`; duplicate-kind lists surface the same
/// duplicate error (`:2035-2038,:2074-2078`).
fn load_settings_preset_names(
    request: &LoadSettingsRequest<'_>,
    catalog: &Catalog,
    direct_stl: bool,
) -> Result<crate::project_slice::compatibility::LoadedPresetNames, SliceError> {
    use crate::project_slice::compatibility::LoadedPresetNames;
    let mut machine = None;
    let mut printer_model = String::new();
    let mut process = None;
    let mut direct_stl_machine = None;
    let mut machine_seen = false;
    let mut process_seen = false;
    for file in request.settings {
        let (kind, document) = parse_file(file)?;
        let document = catalog.admit_document(
            ProfileKind::parse(&kind)?,
            document,
            file.label,
            file.source_identity,
        )?;
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
                if let Some(RawConfigValue::Text(model)) = document.fields.get("printer_model")
                    && let Ok(RawConfigValue::Text(model)) =
                        crate::options::json_deserialize::decode(
                            crate::OptionValueKind::String,
                            model,
                        )
                {
                    printer_model = model;
                }
                if direct_stl {
                    let mut raw = ProjectSettingsRaw::default();
                    raw.extend(document.fields);
                    raw.insert(
                        "printer_settings_id".to_owned(),
                        RawConfigValue::Text(document.name),
                    );
                    direct_stl_machine = Some(raw);
                }
            }
            "process" => {
                if process_seen {
                    return Err(invalid(format!(
                        "duplicate process config file: {}",
                        file.label
                    )));
                }
                process_seen = true;
                let mut preset = descriptor(&document);
                if direct_stl
                    && matches!(document.fields.get("compatible_printers"), Some(RawConfigValue::Text(s)) if s.is_empty())
                {
                    preset.compatible_printers.clear();
                }
                process = Some(preset);
            }
            _ => {}
        }
    }
    Ok(LoadedPresetNames {
        machine,
        printer_model,
        process,
        embedded: None,
        direct_stl_machine,
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
        compatible_printers_condition: match document.fields.get("compatible_printers_condition") {
            Some(RawConfigValue::Text(condition)) => condition.clone(),
            _ => String::new(),
        },
    }
}
