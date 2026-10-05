//! Byte-oriented preset import: `PresetBundle.cpp:1304-1447`
//! (`import_presets` archive extraction) and `:1450-1549`
//! (`import_json_presets` per-record parsing). Temp folders, overwrite
//! prompts, `.info` persistence and bundle bookkeeping are upstream
//! filesystem/UI behavior and stay unported; the returned envelope keeps each
//! record's original option values and `inherits` without flattening.

use std::collections::BTreeMap;
use std::io::{Cursor, Read};

use serde::Serialize;
use zip::ZipArchive;

use crate::SliceError;

const BUNDLE_STRUCTURE_JSON: &str = "bundle_structure.json";

#[derive(Serialize)]
struct ParsedBundle {
    profiles: Vec<BTreeMap<String, serde_json::Value>>,
    bundle: Option<BundleMetadata>,
}

/// `BundleMetadata::load_from_json` (`PresetBundle.cpp:5717-5750`): every
/// field is optional; absent `name` falls back to `bundle_id`, absent
/// `description` to `bundle_type` (backwards compatibility).
#[derive(Default, Serialize)]
struct BundleMetadata {
    #[serde(skip_serializing_if = "String::is_empty")]
    id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    version: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    description: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    author: String,
    #[serde(default)]
    imported_time: i64,
    #[serde(default)]
    updated_time: i64,
}

/// Parse a preset JSON file or an `.orca_printer` / `.orca_filament` /
/// `.orca_bundle` / `.zip` archive into the transport envelope that
/// `catalog_json.imports` consumes.
pub fn parse_preset_bundle(input: impl AsRef<[u8]>, filename: &str) -> Result<String, SliceError> {
    let input = input.as_ref();
    if input.is_empty() {
        return Err(SliceError::EmptyInput);
    }
    let mut bundle = ParsedBundle {
        profiles: Vec::new(),
        bundle: None,
    };
    if is_archive(input, filename) {
        import_archive(input, &mut bundle)?;
    } else {
        import_json(input, &mut bundle.profiles)?;
    }
    serde_json::to_string(&bundle).map_err(|error| {
        SliceError::InvalidInput(format!("preset bundle result cannot serialize: {error}"))
    })
}

fn is_archive(input: &[u8], filename: &str) -> bool {
    // PresetBundle.cpp:1320 dispatches on the extension; a bare JSON check
    // follows. The byte API additionally accepts ZIP magic so callers do not
    // have to normalize extensions.
    input.starts_with(b"PK\x03\x04")
        || [".orca_printer", ".orca_filament", ".orca_bundle", ".zip"]
            .iter()
            .any(|suffix| filename.to_lowercase().ends_with(suffix))
}

fn import_archive(input: &[u8], bundle: &mut ParsedBundle) -> Result<(), SliceError> {
    let mut archive = ZipArchive::new(Cursor::new(input))
        .map_err(|error| invalid(format!("preset archive is malformed: {error}")))?;
    let mut members: Vec<String> = (0..archive.len())
        .filter_map(|index| {
            archive
                .by_index_raw(index)
                .ok()
                .and_then(|file| file.name().rsplit('/').next().map(str::to_owned))
        })
        .collect();
    // Upstream extracts in ZIP order; index_raw order equals central order.
    members = members.into_iter().collect();
    for (index, name) in members.iter().enumerate() {
        if name == BUNDLE_STRUCTURE_JSON {
            let bytes = read_member(&mut archive, index)?;
            bundle.bundle = Some(parse_bundle_metadata(&bytes)?);
            continue;
        }
        // PresetBundle.cpp:1390-1406 feeds every non-structure member to the
        // JSON importer; non-JSON payloads are skipped there. Only `.json`
        // members are preset records; skip the rest without copying the
        // upstream soft-failure into the byte API.
        if !name.to_lowercase().ends_with(".json") {
            continue;
        }
        let bytes = read_member(&mut archive, index)?;
        import_json(&bytes, &mut bundle.profiles)?;
    }
    Ok(())
}

/// `mz_zip_reader_extract_file_to_file`-equivalent bounded read.
fn read_member(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    index: usize,
) -> Result<Vec<u8>, SliceError> {
    let file = archive
        .by_index(index)
        .map_err(|error| invalid(format!("preset archive member is unreadable: {error}")))?;
    let limit = file.size().saturating_add(1);
    let mut bytes = Vec::new();
    file.take(limit)
        .read_to_end(&mut bytes)
        .map_err(|error| invalid(format!("preset archive member failed to expand: {error}")))?;
    Ok(bytes)
}

fn parse_bundle_metadata(input: &[u8]) -> Result<BundleMetadata, SliceError> {
    let raw: serde_json::Value =
        serde_json::from_slice(input).map_err(|error| invalid_json("bundle metadata", error))?;
    let get = |key: &str| raw.get(key).and_then(serde_json::Value::as_str);
    Ok(BundleMetadata {
        id: get("id").unwrap_or_default().to_owned(),
        name: get("name")
            .or_else(|| get("bundle_id"))
            .unwrap_or_default()
            .to_owned(),
        version: get("version").unwrap_or_default().to_owned(),
        description: get("description")
            .or_else(|| get("bundle_type"))
            .unwrap_or_default()
            .to_owned(),
        author: get("author").unwrap_or_default().to_owned(),
        imported_time: raw
            .get("imported_time")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or_default(),
        updated_time: raw
            .get("updated_time")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or_default(),
    })
}

/// `import_json_presets` (`PresetBundle.cpp:1450-1549`) reduced to the
/// byte-oriented contract: parse one record, derive its kind from the
/// settings-ID keys (`:1469-1482`), keep its `inherits` reference and skip
/// records without a parseable `version` or any settings-ID (`:1466-1483`).
fn import_json(
    input: &[u8],
    profiles: &mut Vec<BTreeMap<String, serde_json::Value>>,
) -> Result<(), SliceError> {
    let mut object: BTreeMap<String, serde_json::Value> =
        serde_json::from_slice(input).map_err(|error| invalid_json("preset", error))?;
    let version = object
        .get("version")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    if version.is_empty() {
        // Upstream `Semver::parse` failure skips the record.
        return Ok(());
    }
    let kind = if object.contains_key("printer_settings_id") {
        Some("machine")
    } else if object.contains_key("print_settings_id") {
        Some("process")
    } else if object.contains_key("filament_settings_id") {
        Some("filament")
    } else {
        object
            .get("type")
            .and_then(serde_json::Value::as_str)
            .filter(|kind| matches!(*kind, "machine" | "process" | "filament"))
    };
    let Some(kind) = kind else {
        // PresetBundle.cpp:1479-1483: unknown preset type, not loaded.
        return Ok(());
    };
    if !object.contains_key("type") {
        object.insert(
            "type".to_owned(),
            serde_json::Value::String(kind.to_owned()),
        );
    }
    profiles.push(object);
    Ok(())
}

fn invalid_json(context: &str, error: serde_json::Error) -> SliceError {
    invalid(format!("invalid {context} JSON: {error}"))
}

fn invalid(message: impl Into<String>) -> SliceError {
    SliceError::InvalidInput(message.into())
}
