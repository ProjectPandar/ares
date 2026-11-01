use std::path::{Path, PathBuf};

use serde_json::Value;

// Beta OrcaSlicer.cpp:2118-2157 resolves inherited user inputs by the file
// identity loaded from data_dir/user/default, against data_dir/system.
pub(in super::super) fn stage(work: &Path, profiles: &Path) -> (PathBuf, PathBuf) {
    let datadir = work.join("preset-data");
    let system = datadir.join("system");
    std::fs::create_dir_all(&system).unwrap();
    for vendor in ["OrcaFilamentLibrary", "BBL"] {
        std::fs::copy(
            profiles.join(format!("{vendor}.json")),
            system.join(format!("{vendor}.json")),
        )
        .unwrap();
        copy_directory(&profiles.join(vendor), &system.join(vendor));
    }
    let index: Value =
        serde_json::from_slice(&std::fs::read(profiles.join("BBL.json")).unwrap()).unwrap();
    let mut process: Value =
        serde_json::from_slice(&std::fs::read(work.join("process.json")).unwrap()).unwrap();
    // Preset.cpp:1860-1867 requires a valid version for directory-loaded users.
    process["version"] = index["version"].clone();
    let user = datadir.join("user/default/process");
    std::fs::create_dir_all(&user).unwrap();
    let path = user.join(format!("{}.json", process["name"].as_str().unwrap()));
    super::write_json(&path, &process);
    (datadir, path)
}

fn copy_directory(source: &Path, destination: &Path) {
    std::fs::create_dir(destination).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_directory(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}
