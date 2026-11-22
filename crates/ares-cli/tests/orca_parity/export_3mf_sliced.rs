use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::{Value, json};

use super::direct_stl_slice::square_stl;
use crate::{OrcaRunner, presets::VendorProfiles, runner::repo_root};

const SETTINGS: [&str; 4] = [
    "--load-settings",
    "machine.json;process.json",
    "--load-filaments",
    "filament.json",
];
const STL: &str = "3mf-export.stl";
const GCODE: &str = "Metadata/plate_1.gcode";

#[path = "export_3mf_sliced/comparator.rs"]
mod comparator;
#[path = "export_3mf_sliced/comparison_tests.rs"]
mod comparison_tests;

use comparator::{compare_members, read_members};

#[test]
fn cli_sliced_3mf_from_stl_matches_native_archive() {
    sliced_archive_matches_native(None);
}

#[test]
fn cli_sliced_3mf_from_saved_project_matches_native_archive() {
    sliced_archive_matches_native(Some(SavedShape::Single));
}

#[test]
fn cli_sliced_3mf_with_two_objects_matches_native_archive() {
    sliced_archive_matches_native(Some(SavedShape::TwoObjects));
}

#[test]
fn cli_sliced_3mf_from_arranged_stls_matches_native_archive() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = runner.work_dir();
    let native_cwd = root.join("native-arranged/cwd");
    let ares_cwd = root.join("ares-arranged/cwd");
    for dir in [&native_cwd, &ares_cwd] {
        fs::create_dir_all(dir.join("out")).unwrap();
    }
    write_inputs(&native_cwd);
    for (name, edge) in [("large.stl", 180.0), ("medium.stl", 102.0)] {
        fs::write(native_cwd.join(name), square_stl([0.0, 0.0], edge, 1.2)).unwrap();
    }
    for file in [
        "large.stl",
        "medium.stl",
        "machine.json",
        "process.json",
        "filament.json",
    ] {
        fs::copy(native_cwd.join(file), ares_cwd.join(file)).unwrap();
    }
    let argv = [
        SETTINGS.as_slice(),
        &[
            "--slice",
            "0",
            "--arrange",
            "1",
            "--export-3mf",
            "export.3mf",
            "large.stl",
            "medium.stl",
        ],
    ]
    .concat();
    assert_eq!(invoke(runner.bin(), &native_cwd, &argv, true), Some(0));
    let native = read_members(&native_cwd.join("export.3mf"));
    assert_eq!(native.len(), 17);
    for plate in [1, 2] {
        let gcode = format!("Metadata/plate_{plate}.gcode");
        let md5 = format!("{gcode}.md5");
        assert!(
            !native
                .iter()
                .find(|entry| entry.name == gcode)
                .unwrap()
                .bytes
                .is_empty()
        );
        assert_eq!(
            native
                .iter()
                .find(|entry| entry.name == md5)
                .unwrap()
                .bytes
                .len(),
            32
        );
    }
    let ares = Path::new(env!("CARGO_BIN_EXE_ares"));
    assert_eq!(
        invoke(ares, &ares_cwd, &argv, false),
        Some(0),
        "arranged STL export failed in {}",
        ares_cwd.display()
    );
    let actual = read_members(&ares_cwd.join("export.3mf"));
    let gcode_members = actual
        .iter()
        .map(|entry| entry.name.as_str())
        .filter(|name| {
            name.starts_with("Metadata/plate_")
                && (name.ends_with(".gcode.md5") || name.ends_with(".gcode"))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        gcode_members,
        [
            "Metadata/plate_1.gcode.md5",
            "Metadata/plate_2.gcode.md5",
            "Metadata/plate_1.gcode",
            "Metadata/plate_2.gcode",
        ],
        "arranged STL export must write all checksums before G-code"
    );
    compare_members(native, actual);
}

#[test]
fn cli_sliced_3mf_with_translated_model_part_matches_native_archive() {
    sliced_archive_matches_native(Some(SavedShape::TwoObjectsWithParts));
}

#[derive(Clone, Copy)]
enum SavedShape {
    Single,
    TwoObjects,
    TwoObjectsWithParts,
}

fn sliced_archive_matches_native(shape: Option<SavedShape>) {
    let saved = shape.is_some();
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = runner.work_dir();
    let native_cwd = root.join("native/cwd");
    let ares_cwd = root.join("ares/cwd");
    for dir in [&native_cwd, &ares_cwd] {
        fs::create_dir_all(dir.join("out")).unwrap();
    }
    if let Some(shape) = shape {
        let source = root.join("source/cwd");
        fs::create_dir_all(source.join("out")).unwrap();
        write_saved_input(&runner, &source);
        let mut input = fs::read(source.join("saved.3mf")).unwrap();
        if !matches!(shape, SavedShape::Single) {
            input = super::export_3mf_sliced_rejection::multiobject_project(
                &input,
                matches!(shape, SavedShape::TwoObjectsWithParts),
            );
        }
        for dir in [&native_cwd, &ares_cwd] {
            fs::write(dir.join("saved.3mf"), &input).unwrap();
        }
    } else {
        write_inputs(&native_cwd);
        for file in [STL, "machine.json", "process.json", "filament.json"] {
            fs::copy(native_cwd.join(file), ares_cwd.join(file)).unwrap();
        }
    }
    let mut argv = Vec::new();
    if !saved {
        argv.extend(SETTINGS);
    }
    argv.extend([
        "--slice",
        "1",
        "--export-3mf",
        "export.3mf",
        if saved { "saved.3mf" } else { STL },
    ]);
    assert_eq!(invoke(runner.bin(), &native_cwd, &argv, true), Some(0));
    let native = read_members(&native_cwd.join("export.3mf"));
    let split_files = 1 + usize::from(saved && !matches!(shape, Some(SavedShape::Single)));
    assert_eq!(native.len(), 12 + split_files);
    let ares = Path::new(env!("CARGO_BIN_EXE_ares"));
    assert_eq!(
        invoke(ares, &ares_cwd, &argv, false),
        Some(0),
        "slice and export failed in {}",
        ares_cwd.display()
    );
    let actual = read_members(&ares_cwd.join("export.3mf"));
    let gcode = &actual
        .iter()
        .find(|entry| entry.name == GCODE)
        .unwrap()
        .bytes;
    assert_eq!(
        std::str::from_utf8(gcode)
            .unwrap()
            .split('\n')
            .filter(|line| line.starts_with("; generated by "))
            .collect::<Vec<_>>(),
        [concat!(
            "; generated by OrcaSlicer Ares-",
            env!("CARGO_PKG_VERSION"),
            " on 1980-01-01 at 00:00:00"
        )]
    );
    compare_members(native, actual);
    for cwd in [&native_cwd, &ares_cwd] {
        assert!(!cwd.join("plate_1.gcode").exists());
        assert_eq!(fs::read_dir(cwd.join("out")).unwrap().count(), 0);
    }
}

pub(super) fn write_saved_input(runner: &OrcaRunner, dir: &Path) {
    write_inputs(dir);
    let argv = [SETTINGS.as_slice(), &["--export-3mf", "saved.3mf", STL]].concat();
    assert_eq!(invoke(runner.bin(), dir, &argv, true), Some(0));
}

fn write_inputs(dir: &Path) {
    let printer = "Bambu Lab X1 Carbon 0.4 nozzle";
    let profiles =
        VendorProfiles::load(&repo_root().join("OrcaSlicer/resources/profiles"), "BBL").unwrap();
    let process = profiles.compatible_process(printer).unwrap();
    let filament = profiles.compatible_filament(printer).unwrap();
    for (kind, mut fields) in [
        ("machine", profiles.machine(printer).unwrap()),
        ("process", profiles.process(&process).unwrap()),
        ("filament", profiles.filament(&filament).unwrap()),
    ] {
        fields.extend([
            ("type".into(), json!(kind)),
            ("from".into(), json!("system")),
            ("instantiation".into(), json!("true")),
        ]);
        fs::write(
            dir.join(format!("{kind}.json")),
            serde_json::to_vec(&Value::Object(fields)).unwrap(),
        )
        .unwrap();
    }
    fs::copy(repo_root().join("tests/parity/cube10.stl"), dir.join(STL)).unwrap();
}

pub(super) fn invoke(bin: &Path, cwd: &Path, argv: &[&str], native: bool) -> Option<i32> {
    let artifacts = cwd.parent().unwrap();
    let tmp = artifacts.join("tmp");
    fs::create_dir(&tmp).unwrap();
    fs::write(artifacts.join("argv.nul"), argv.join("\0") + "\0").unwrap();
    fs::write(artifacts.join("cwd.txt"), cwd.display().to_string()).unwrap();
    fs::write(artifacts.join("tmpdir.txt"), tmp.display().to_string()).unwrap();
    let mut command = Command::new(bin);
    if !native {
        command.arg("slice");
    }
    let output = command
        .current_dir(cwd)
        .env("TMPDIR", &tmp)
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .args(argv)
        .output()
        .unwrap();
    let code = output.status.code();
    fs::write(artifacts.join("exit"), format!("{code:?}\n")).unwrap();
    fs::write(artifacts.join("stdout.log"), output.stdout).unwrap();
    fs::write(artifacts.join("stderr.log"), output.stderr).unwrap();
    let mut paths = Vec::new();
    inventory(cwd, artifacts, &mut paths);
    inventory(&tmp, artifacts, &mut paths);
    paths.sort_unstable();
    fs::write(artifacts.join("inventory.txt"), paths.join("\n") + "\n").unwrap();
    code
}

fn inventory(dir: &Path, root: &Path, paths: &mut Vec<String>) {
    for item in fs::read_dir(dir).unwrap() {
        let item = item.unwrap();
        let path = item.path();
        let name = path.strip_prefix(root).unwrap().display();
        if item.file_type().unwrap().is_dir() {
            paths.push(format!("{name}/"));
            inventory(&path, root, paths);
        } else {
            paths.push(format!("{name}\t{}", item.metadata().unwrap().len()));
        }
    }
}
