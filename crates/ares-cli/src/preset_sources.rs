//! Host half of PresetBundle.cpp:614–650, 654–688 source-manifest loading.
use ares_core::{LoadSettingsFile, LoadedPresetSources};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs, io,
    path::{Path, PathBuf},
};

pub(super) fn load<'a>(
    files: impl Iterator<Item = LoadSettingsFile<'a>>,
) -> Result<LoadedPresetSources, Box<dyn std::error::Error>> {
    let mut sources = LoadedPresetSources::new();
    let mut loaded = BTreeSet::new();
    for file in files {
        if file.json.is_empty() {
            continue;
        }
        let document: Value = serde_json::from_str(file.json)?;
        if document["inherits"].as_str().is_none_or(str::is_empty) {
            continue;
        }
        if matches!(document["from"].as_str(), Some("user" | "User") | None) {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("CLI user preset source is unresolved (datadir/AppConfig producer is deferred): {}", file.label),
            ).into());
        }
        if document["from"].as_str() != Some("system") {
            continue;
        }
        let mut source = PathBuf::new();
        for component in std::path::absolute(file.label)?.components() {
            match component {
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir => {
                    source.pop();
                }
                component => source.push(component),
            }
        }
        for vendor_dir in source.parent().into_iter().flat_map(Path::ancestors) {
            let Some(id) = vendor_dir.file_name().and_then(|id| id.to_str()) else {
                continue;
            };
            let Some(root) = vendor_dir.parent() else {
                continue;
            };
            if !root.join(format!("{id}.json")).is_file() {
                continue;
            }
            let library = "OrcaFilamentLibrary";
            if id != library && root.join(format!("{library}.json")).is_file() {
                add_vendor(&mut sources, &mut loaded, root, library)?;
            }
            add_vendor(&mut sources, &mut loaded, root, id)?;
            break;
        }
    }
    Ok(sources)
}

fn add_vendor(
    sources: &mut LoadedPresetSources,
    loaded: &mut BTreeSet<PathBuf>,
    root: &Path,
    id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest = root.join(format!("{id}.json"));
    if !loaded.insert(manifest.clone()) {
        return Ok(());
    }
    let index: Value = serde_json::from_slice(&fs::read(manifest)?)?;
    let mut files = serde_json::Map::new();
    let mut identities = Vec::new();
    for key in [
        "machine_model_list",
        "machine_list",
        "process_list",
        "filament_list",
    ] {
        let Some(entries) = index[key].as_array() else {
            continue;
        };
        for entry in entries {
            let sub_path = entry["sub_path"].as_str().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "vendor manifest sub_path is missing",
                )
            })?;
            let path = root.join(id).join(sub_path);
            let (json, identity) = crate::read_preset_source(&path)?;
            let document: Value = serde_json::from_str(&json)?;
            identities.push((sub_path.to_owned(), identity));
            files.insert(sub_path.to_owned(), document);
        }
    }
    sources.add_vendor(
        &json!({"id": id, "index": index, "files": files}).to_string(),
        &identities,
    )?;
    Ok(())
}
