//! Repeatable acceptance artifact for `decodeConfigValue` (ARD-0108).
//!
//! Decodes every registry-known key/value pair found in
//!   - `tests/**/*.3mf`        `Metadata/project_settings.config` entries
//!   - `OrcaSlicer/resources/profiles/**/*.json`  vendor preset fields
//! and emits a JSON report on stdout:
//!   { files, entries, decoded, unknown_key, failed, failures[] }

use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

fn collect_files(root: &Path, extension: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, extension, out);
        } else if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
        {
            out.push(path);
        }
    }
}

fn read_project_settings(path: &Path) -> Vec<(String, String)> {
    let Ok(bytes) = fs::read(path) else {
        return Vec::new();
    };
    let Ok(mut archive) = zip::ZipArchive::new(Cursor::new(bytes)) else {
        return Vec::new();
    };
    let Ok(mut entry) = archive.by_name("Metadata/project_settings.config") else {
        return Vec::new();
    };
    let mut text = String::new();
    if entry.read_to_string(&mut text).is_err() {
        return Vec::new();
    }
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (key, value) = line.split_once('=')?;
            Some((key.trim().to_owned(), value.trim().to_owned()))
        })
        .collect()
}

fn profile_value_to_serialized(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Bool(b) => Some(if *b { "1".to_owned() } else { "0".to_owned() }),
        Value::Number(n) => Some(n.to_string()),
        Value::Array(items) => Some(
            items
                .iter()
                .filter_map(|item| match item {
                    Value::String(s) => Some(s.clone()),
                    Value::Bool(b) => Some(if *b { "1".to_owned() } else { "0".to_owned() }),
                    Value::Number(n) => Some(n.to_string()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(","),
        ),
        _ => None,
    }
}

fn read_profile(path: &Path) -> Vec<(String, String)> {
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(json) = serde_json::from_str::<Value>(&text) else {
        return Vec::new();
    };
    let Some(map) = json.as_object() else {
        return Vec::new();
    };
    map.iter()
        .filter_map(|(key, value)| {
            profile_value_to_serialized(value).map(|value| (key.clone(), value))
        })
        .collect()
}

fn main() {
    let mut files = 0usize;
    let mut entries = 0usize;
    let mut decoded = 0usize;
    let mut unknown_key = 0usize;
    let mut failures = Vec::new();

    let mut run_source = |source: &str, pairs: Vec<(String, String)>| {
        files += 1;
        for (key, serialized) in pairs {
            entries += 1;
            match ares_core::decode_config_value_by_key(&key, &serialized) {
                Ok(_) => decoded += 1,
                Err(error) if error.contains("unknown config option key") => unknown_key += 1,
                Err(error) => failures.push(json!({
                    "source": source,
                    "key": key,
                    "serialized": serialized,
                    "error": error,
                })),
            }
        }
    };

    let mut archives = Vec::new();
    collect_files(Path::new("tests"), "3mf", &mut archives);
    collect_files(Path::new("crates/ares-cli/tests"), "3mf", &mut archives);
    for archive in archives {
        let pairs = read_project_settings(&archive);
        run_source(&archive.to_string_lossy(), pairs);
    }

    let mut profiles = Vec::new();
    collect_files(
        Path::new("OrcaSlicer/resources/profiles"),
        "json",
        &mut profiles,
    );
    for profile in profiles {
        let pairs = read_profile(&profile);
        run_source(&profile.to_string_lossy(), pairs);
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "files": files,
            "entries": entries,
            "decoded": decoded,
            "unknown_key": unknown_key,
            "failed": failures.len(),
            "failures": failures,
            "notes": [
                "unknown_key: keys absent from the option registry (printer metadata fields, sla-only keys, etc.); decode was not attempted",
                "failures must only be inputs where upstream ConfigOption*::deserialize also fails (bad bool token, unterminated quoted string, malformed point)",
                "numeric vector elements follow C++11 [facet.num.get.virtuals] Stage 3: a failed `>>` extraction writes 0, matching upstream (e.g. 'V' or ',' decode to 0)",
            ],
        }))
        .unwrap()
    );
}
