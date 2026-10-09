//! `--load-settings`/`--load-filaments` composition — the single upstream
//! merge shared with the catalog/profile-entry path.
//!
//! Upstream boundary: per-file validation and the machine/process/filament
//! loops live in the `load_config_file` lambda and its callers
//! (`OrcaSlicer.cpp:1953-2215`); the merge itself is `update_full_config`
//! and the filament resize/`set_at` contract (`:2826-3337`), which
//! `Catalog::compose` already ports. This module only ingests the CLI
//! preset files as user-scope documents and selects them by name — no
//! second composition exists.

use super::ProfileSelection;
use super::catalog::{Catalog, ComposedSelection, ProfileDocument};
use crate::SliceError;

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

/// Parses and validates one preset file exactly like the `load_config_file`
/// lambda (`OrcaSlicer.cpp:1953-2024`): `from` must be `system|User|user`
/// (`:1989-1993`) and `type` must be a known kind (`:1996-2000`).
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
    let mut catalog = Catalog::empty();
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
    let mut filament_names = Vec::new();
    for file in request.filaments {
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
        filament_names.push(document.name.clone());
        catalog.push_import(document)?;
    }
    let selection = ProfileSelection::new(
        process.unwrap_or_default(),
        machine.unwrap_or_default(),
        filament_names,
    )?;
    catalog.compose(&selection)
}
