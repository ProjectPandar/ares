use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

const ENTRIES: [(&str, &str); 3] = [
    ("machine", "WEMAKE3D"),
    ("process", "WEMAKE3D"),
    ("filament", "OrcaFilamentLibrary"),
];

pub(super) fn stage(work: &Path, case: &str, texts: &[String; 3]) -> [PathBuf; 3] {
    if case == "bypass" {
        return std::array::from_fn(|index| {
            let path = work.join(format!("{}.json", ENTRIES[index].0));
            fs::write(&path, &texts[index]).unwrap();
            path
        });
    }
    let supply = super::super::application::project_supply()
        .unwrap()
        .expect("system preset source documents for the manifest-backed fixture");
    let system = supply.native_dir.join("system");
    let profiles = work.join("profiles");
    fs::create_dir(&profiles).unwrap();
    for vendor in ["WEMAKE3D", "OrcaFilamentLibrary"] {
        let name = format!("{vendor}.json");
        fs::write(profiles.join(&name), fs::read(system.join(&name)).unwrap()).unwrap();
        copy_tree(&system.join(vendor), &profiles.join(vendor));
    }
    std::array::from_fn(|index| {
        let (kind, vendor) = ENTRIES[index];
        let dir = profiles.join(vendor);
        let manifest_path = profiles.join(format!("{vendor}.json"));
        let mut manifest: Value = read_json(&manifest_path);
        let entries = manifest[format!("{kind}_list")].as_array_mut().unwrap();
        let mut leaf: Value = serde_json::from_str(&texts[index]).unwrap();
        let path = dir.join(sub_path(
            entries
                .iter()
                .find(|entry| entry["name"] == leaf["name"])
                .unwrap(),
        ));
        fs::remove_file(&path).unwrap();
        if case == "control" {
            let parent = leaf["inherits"].as_str().unwrap();
            let alias = format!("{parent}_equivalent");
            assert!(!entries.iter().any(|entry| entry["name"] == alias));
            let index = entries
                .iter()
                .position(|entry| entry["name"] == parent)
                .unwrap();
            let mut entry = entries[index].clone();
            let source = dir.join(sub_path(&entry));
            let target = source.with_file_name(format!("{alias}.json"));
            let mut alias_document = read_json(&source);
            assert_eq!(alias_document["name"], parent);
            assert_eq!(alias_document["instantiation"], "false");
            alias_document["name"] = Value::String(alias.clone());
            fs::write(&target, serde_json::to_vec_pretty(&alias_document).unwrap()).unwrap();
            entry["name"] = Value::String(alias.clone());
            entry["sub_path"] = Value::String(
                target
                    .strip_prefix(&dir)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace('\\', "/"),
            );
            entries.insert(index + 1, entry);
            fs::write(
                &manifest_path,
                serde_json::to_vec_pretty(&manifest).unwrap(),
            )
            .unwrap();
            leaf["inherits"] = Value::String(alias);
            fs::write(&path, serde_json::to_vec_pretty(&leaf).unwrap()).unwrap();
        } else {
            fs::write(&path, &texts[index]).unwrap();
        }
        path
    })
}

fn copy_tree(source: &Path, target: &Path) {
    fs::create_dir(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let path = target.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &path);
        } else {
            fs::copy(entry.path(), path).unwrap();
        }
    }
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn sub_path(entry: &Value) -> &str {
    entry["sub_path"].as_str().unwrap()
}
