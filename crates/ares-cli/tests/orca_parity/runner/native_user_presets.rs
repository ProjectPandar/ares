use std::path::{Path, PathBuf};

use serde_json::Value;

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
    for vendor in vendors {
        let id = vendor["id"].as_str().unwrap();
        let original = super::vendor(profiles, id);
        for (relative, fields) in vendor["files"].as_object().unwrap() {
            if fields != &original["files"][relative] {
                super::write_json(&datadir.join("system").join(id).join(relative), fields);
            }
        }
    }
    let leaf = |kind: &str, name: &str| {
        let (path, fields) = vendors
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
                let mut fields = fields.clone();
                fields["from"] = Value::from("system");
                (datadir.join("system").join(vendor).join(relative), fields)
            })
            .unwrap_or_else(|| {
                let fields = catalog["imports"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .flat_map(|group| group["profiles"].as_array().unwrap())
                    .find(|fields| fields["type"] == kind && fields["name"] == name)
                    .unwrap();
                let parent = fields["inherits"].as_str().unwrap();
                let vendor = vendors
                    .iter()
                    .find(|vendor| {
                        vendor["files"]
                            .as_object()
                            .unwrap()
                            .values()
                            .any(|fields| fields["type"] == kind && fields["name"] == parent)
                    })
                    .unwrap();
                let mut fields = fields.clone();
                fields["from"] = Value::from("user");
                // Preset.cpp:1860-1867 requires a version for directory-loaded users.
                fields["version"] = vendor["index"]["version"].clone();
                let directory = datadir.join("user/default").join(kind);
                std::fs::create_dir_all(&directory).unwrap();
                (directory.join(format!("{name}.json")), fields)
            });
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
