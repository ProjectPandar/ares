use std::path::{Path, PathBuf};

use serde_json::Value;

// Beta OrcaSlicer.cpp:2118-2157 resolves inherited user inputs by the file
// identity loaded from data_dir/user/default, against data_dir/system.
pub(in super::super) fn stage(work: &Path, profiles: &Path) -> (PathBuf, PathBuf) {
    let datadir = system(work, profiles, &["OrcaFilamentLibrary", "BBL"]);
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

/// An isolated data_dir whose `system` holds the unchanged `vendors` manifests and trees, as
/// `PresetBundle.cpp:3313-3345` loads them. A system leaf inside it resolves by file identity
/// through its vendor manifest (`PresetBundle.cpp:617-645`) and beta flattens the tree itself.
pub(in super::super) fn system(work: &Path, profiles: &Path, vendors: &[&str]) -> PathBuf {
    let datadir = work.join("preset-data");
    let system = datadir.join("system");
    std::fs::create_dir_all(&system).unwrap();
    for vendor in vendors {
        std::fs::copy(
            profiles.join(format!("{vendor}.json")),
            system.join(format!("{vendor}.json")),
        )
        .unwrap();
        copy_directory(&profiles.join(vendor), &system.join(vendor));
    }
    datadir
}

pub(in super::super) struct ResolvedPresets {
    pub(in super::super) datadir: PathBuf,
    pub(in super::super) settings: [PathBuf; 2],
    pub(in super::super) filaments: Vec<PathBuf>,
}

/// Native resolves the same manifest-owned leaves supplied to `slice_with_profiles`.
pub(in super::super) fn resolved(
    work: &Path,
    profiles: &Path,
    catalog: &Value,
    selection: &Value,
) -> ResolvedPresets {
    let vendors = catalog["vendors"].as_array().unwrap();
    let names = vendors
        .iter()
        .map(|vendor| vendor["id"].as_str().unwrap())
        .collect::<Vec<_>>();
    let datadir = system(work, profiles, &names);
    let leaf = |kind: &str, name: &str| {
        let (path, mut fields) = vendors
            .iter()
            .flat_map(|vendor| {
                vendor["files"]
                    .as_object()
                    .unwrap()
                    .iter()
                    .map(move |(path, fields)| (vendor["id"].as_str().unwrap(), path, fields))
            })
            .find(|(_, _, fields)| fields["type"] == kind && fields["name"] == name)
            .map(|(vendor, relative, fields)| {
                (
                    datadir.join("system").join(vendor).join(relative),
                    fields.clone(),
                )
            })
            .unwrap();
        // CLI::load_config_file requires source metadata, not completed option values.
        fields["from"] = Value::from("system");
        super::write_json(&path, &fields);
        path
    };
    ResolvedPresets {
        settings: [
            leaf("machine", selection["machine"].as_str().unwrap()),
            leaf("process", selection["process"].as_str().unwrap()),
        ],
        filaments: selection["filaments"]
            .as_array()
            .unwrap()
            .iter()
            .map(|name| leaf("filament", name.as_str().unwrap()))
            .collect(),
        datadir,
    }
}

pub(in super::super) fn slice_stl(
    runner: &super::OrcaRunner,
    presets: &ResolvedPresets,
    input: &Path,
    output: &str,
    overrides: &Value,
) -> Vec<u8> {
    let output = runner.work.join(output);
    std::fs::create_dir(&output).unwrap();
    for (index, path) in presets
        .settings
        .iter()
        .chain(&presets.filaments)
        .enumerate()
    {
        std::fs::copy(path, output.join(format!("loaded-leaf-{index}.json"))).unwrap();
    }
    let mut arguments = vec![
        "--datadir".to_owned(),
        presets.datadir.to_str().unwrap().to_owned(),
        "--load-settings".to_owned(),
        super::super::join_paths(presets.settings.iter()),
        "--load-filaments".to_owned(),
        super::super::join_paths(presets.filaments.iter()),
        "--arrange".to_owned(),
        "0".to_owned(),
        "--slice".to_owned(),
        "0".to_owned(),
        "--outputdir".to_owned(),
        output.to_str().unwrap().to_owned(),
        input.to_str().unwrap().to_owned(),
    ];
    // OrcaSlicer.cpp:4138-4226 applies CLI overrides after loading the preset baseline.
    for (key, value) in overrides.as_object().unwrap() {
        arguments.extend([
            format!("--{}", key.replace('_', "-")),
            value.as_str().unwrap().to_owned(),
        ]);
    }
    super::stages::run(
        runner.bin(),
        arguments,
        super::stages::Stage::Slice,
        &runner.work,
    )
    .unwrap();
    std::fs::read(output.join("plate_1.gcode")).unwrap()
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
