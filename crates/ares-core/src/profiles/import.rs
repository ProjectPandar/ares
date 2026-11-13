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
use crate::project::raw_settings::RawConfigValue;

const BUNDLE_STRUCTURE_JSON: &str = "bundle_structure.json";

/// Import result serialized by the WASM adapter into the transport envelope
/// that `catalog_json.imports` consumes.
#[derive(Debug, Serialize)]
pub struct ParsedPresetBundle {
    profiles: Vec<BTreeMap<String, RawConfigValue>>,
    bundle: Option<BundleMetadata>,
}

/// `BundleMetadata::load_from_json` (`PresetBundle.cpp:5717-5750`): every
/// field is optional; absent `name` falls back to `bundle_id`, absent
/// `description` to `bundle_type` (backwards compatibility).
#[derive(Debug, Default, serde::Deserialize, Serialize)]
pub struct BundleMetadata {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    version: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    description: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    author: String,
    #[serde(default)]
    imported_time: i64,
    #[serde(default)]
    updated_time: i64,
}

/// Parse a preset JSON file or an `.orca_printer` / `.orca_filament` /
/// `.orca_bundle` / `.zip` archive into the typed import envelope.
pub fn parse_preset_bundle(
    input: impl AsRef<[u8]>,
    filename: &str,
) -> Result<ParsedPresetBundle, SliceError> {
    let input = input.as_ref();
    if input.is_empty() {
        return Err(SliceError::EmptyInput);
    }
    let mut bundle = ParsedPresetBundle {
        profiles: Vec::new(),
        bundle: None,
    };
    if is_archive(input, filename) {
        import_archive(input, &mut bundle)?;
    } else {
        import_json(input, &mut bundle.profiles)?;
    }
    Ok(bundle)
}

/// Catalog-handle import path: the importer's normalized records converted
/// to catalog documents (the one-shot transport feeds the same records from
/// `ParsedPresetBundle`).
pub(super) fn parse_import_documents(
    input: &[u8],
    filename: &str,
) -> Result<Vec<super::catalog::ProfileDocument>, SliceError> {
    let bundle = parse_preset_bundle(input, filename)?;
    Ok(bundle
        .profiles
        .into_iter()
        .map(|mut fields| {
            let mut text = |key: &str| match fields.remove(key) {
                Some(RawConfigValue::Text(value)) => Some(value),
                _ => None,
            };
            super::catalog::ProfileDocument {
                kind: text("type").unwrap_or_default(),
                name: text("name").unwrap_or_default(),
                inherits: text("inherits"),
                instantiation: text("instantiation"),
                includes: Vec::new(),
                fields,
            }
        })
        .collect())
}

fn is_archive(input: &[u8], filename: &str) -> bool {
    // PresetBundle.cpp:1320 dispatches on the extension; the byte API
    // additionally accepts ZIP magic so callers do not have to normalize
    // extensions.
    input.starts_with(b"PK\x03\x04")
        || [".orca_printer", ".orca_filament", ".orca_bundle", ".zip"]
            .iter()
            .any(|suffix| filename.to_lowercase().ends_with(suffix))
}

fn import_archive(input: &[u8], bundle: &mut ParsedPresetBundle) -> Result<(), SliceError> {
    let mut archive = ZipArchive::new(Cursor::new(input))
        .map_err(|error| invalid(format!("preset archive is malformed: {error}")))?;
    let names: Vec<String> = (0..archive.len())
        .filter_map(|index| {
            archive.by_index_raw(index).ok().and_then(|file| {
                let name = file.name();
                name.rsplit('/').next().map(str::to_owned)
            })
        })
        .collect();
    for (index, name) in names.iter().enumerate() {
        if name == BUNDLE_STRUCTURE_JSON {
            let bytes = read_member(&mut archive, index)?;
            bundle.bundle = Some(parse_bundle_metadata(&bytes)?);
            continue;
        }
        // PresetBundle.cpp:1390-1406 feeds every non-structure member to the
        // JSON importer; only `.json` members carry preset records.
        if !name.to_lowercase().ends_with(".json") {
            continue;
        }
        let bytes = read_member(&mut archive, index)?;
        import_json(&bytes, &mut bundle.profiles)?;
    }
    Ok(())
}

/// Bounded member expansion (`mz_zip_reader_extract_to_mem`-equivalent).
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
    // Deserialize directly; the `name`/`description` fallbacks run after.
    #[derive(Default, serde::Deserialize)]
    struct Raw {
        id: Option<String>,
        name: Option<String>,
        bundle_id: Option<String>,
        version: Option<String>,
        description: Option<String>,
        bundle_type: Option<String>,
        author: Option<String>,
        imported_time: Option<i64>,
        updated_time: Option<i64>,
    }
    let raw: Raw = serde_json::from_slice(super::without_utf8_bom(input))
        .map_err(|error| invalid_json("bundle metadata", error))?;
    Ok(BundleMetadata {
        id: raw.id.unwrap_or_default(),
        name: raw.name.or(raw.bundle_id).unwrap_or_default(),
        version: raw.version.unwrap_or_default(),
        description: raw.description.or(raw.bundle_type).unwrap_or_default(),
        author: raw.author.unwrap_or_default(),
        imported_time: raw.imported_time.unwrap_or_default(),
        updated_time: raw.updated_time.unwrap_or_default(),
    })
}

/// `import_json_presets` (`PresetBundle.cpp:1450-1549`) reduced to the
/// byte-oriented contract: parse one record, derive its kind from the
/// settings-ID keys (`:1469-1482`), keep its `inherits` reference, and skip
/// records without a parseable `version` or any settings-ID (`:1466-1483`).
fn import_json(
    input: &[u8],
    profiles: &mut Vec<BTreeMap<String, RawConfigValue>>,
) -> Result<(), SliceError> {
    let mut object: BTreeMap<String, RawConfigValue> =
        serde_json::from_slice(super::without_utf8_bom(input))
            .map_err(|error| invalid_json("preset", error))?;
    let version = object.get("version").and_then(raw_text).unwrap_or_default();
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
            .and_then(raw_text)
            .filter(|kind| matches!(*kind, "machine" | "process" | "filament"))
    };
    let Some(kind) = kind else {
        // PresetBundle.cpp:1479-1483: unknown preset type, not loaded.
        return Ok(());
    };
    if !object.contains_key("type") {
        object.insert("type".to_owned(), RawConfigValue::Text(kind.to_owned()));
    }
    profiles.push(object);
    Ok(())
}

fn raw_text(value: &RawConfigValue) -> Option<&str> {
    match value {
        RawConfigValue::Text(text) => Some(text),
        _ => None,
    }
}

fn invalid_json(context: &str, error: serde_json::Error) -> SliceError {
    invalid(format!("invalid {context} JSON: {error}"))
}

fn invalid(message: impl Into<String>) -> SliceError {
    SliceError::InvalidInput(message.into())
}
