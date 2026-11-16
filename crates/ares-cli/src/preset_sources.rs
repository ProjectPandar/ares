//! Host half of PresetBundle.cpp:614–650, 654–688 source-manifest loading.
use ares_core::{LoadSettingsFile, LoadedPresetSources};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs, io,
    path::{Path, PathBuf},
};

pub(super) fn load<'a>(
    datadir: Option<&Path>,
    files: impl Iterator<Item = LoadSettingsFile<'a>>,
) -> Result<LoadedPresetSources, Box<dyn std::error::Error>> {
    let mut sources = LoadedPresetSources::new();
    let mut users_loaded = false;
    let mut loaded = BTreeSet::new();
    for file in files {
        if file.json.is_empty() {
            continue;
        }
        let document: Value = serde_json::from_str(file.json)?;
        if document["inherits"].as_str().is_none_or(str::is_empty) {
            continue;
        }
        if matches!(document["from"].as_str(), Some("user" | "User")) {
            if !users_loaded && let Some(datadir) = datadir {
                load_user_presets(datadir, &mut sources)?;
                users_loaded = true;
            }
            continue;
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

fn load_user_presets(
    datadir: &Path,
    sources: &mut LoadedPresetSources,
) -> Result<(), Box<dyn std::error::Error>> {
    let system = datadir.join("system");
    let mut vendors = fs::read_dir(&system)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    vendors.retain(|path| path.extension().and_then(|value| value.to_str()) == Some("json"));
    vendors.sort_by_key(|path| {
        (
            path.file_stem().unwrap() != "OrcaFilamentLibrary",
            path.clone(),
        )
    });
    let mut loaded = BTreeSet::new();
    for path in vendors {
        add_vendor(
            sources,
            &mut loaded,
            &system,
            path.file_stem().unwrap().to_str().unwrap(),
        )?;
    }
    let app_config = datadir.join("OrcaSlicer.conf");
    let user = if app_config.is_file() {
        let document = match serde_json::from_slice::<Value>(&fs::read(app_config)?) {
            Ok(document) => document,
            Err(error) => {
                eprintln!("Ignoring invalid app config during CLI preset resolution: {error}");
                Value::Null
            }
        };
        document
            .get("app")
            .and_then(|presets| presets.get("preset_folder"))
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .unwrap_or("default")
            .to_owned()
    } else {
        "default".to_owned()
    };
    let root = datadir.join("user").join(user);
    for (kind, directory) in [
        ("machine", "machine"),
        ("process", "process"),
        ("filament", "filament"),
    ] {
        let path = root.join(directory);
        if !path.is_dir() {
            continue;
        }
        load_user_directory(&path, kind, sources)?;
    }
    Ok(())
}

fn load_user_directory(
    directory: &Path,
    kind: &str,
    sources: &mut LoadedPresetSources,
) -> Result<(), Box<dyn std::error::Error>> {
    let base = directory.join("base");
    if base.is_dir() {
        load_user_directory(&base, kind, sources)?;
    }
    let mut entries = Vec::new();
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let Ok((json, identity)) = crate::read_preset_source(&path) else {
            continue;
        };
        let Ok(mut document) = serde_json::from_str::<Value>(&json) else {
            continue;
        };
        if !document.is_object() {
            continue;
        }
        document["type"] = Value::String(kind.to_owned());
        entries.push((
            path.file_stem().unwrap().to_str().unwrap().to_owned(),
            document.to_string(),
            identity,
        ));
    }
    sources.add_user_collection(&entries)?;
    Ok(())
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
