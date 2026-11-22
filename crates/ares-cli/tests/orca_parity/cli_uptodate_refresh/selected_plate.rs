use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

use crate::{golden::compare_ordered_bytes_generator_only, runner::OrcaRunner};

use super::{artifact_dir, project, values};

const SETTINGS: &[&str] = &["--uptodate-settings", "../machine.json;../process.json"];
const FILAMENTS: &[&str] = &["--uptodate-filaments", "../filament.json"];
const REFRESH: &[&str] = &["--uptodate"];

#[test]
fn selected_plate_refreshes_retained_machine_process_and_filament() {
    verify(
        &[REFRESH, SETTINGS, FILAMENTS].concat(),
        ["45", "8", "1.35"],
    );
}

#[test]
fn selected_plate_refreshes_retained_machine_and_process_from_files() {
    verify(&[REFRESH, SETTINGS].concat(), ["45", "8", "1.25"]);
}

#[test]
fn selected_plate_refreshes_retained_filament_from_file() {
    verify(&[REFRESH, FILAMENTS].concat(), ["40", "0", "1.35"]);
}

#[test]
fn selected_plate_refreshes_retained_values_from_installed_presets() {
    verify(REFRESH, ["40", "0", "1.25"]);
}

#[test]
fn selected_plate_keeps_retained_values_without_refresh() {
    verify(&[], ["44", "6", "1.25"]);
    verify(
        &[
            "--uptodate-settings",
            "missing-machine.json",
            "--uptodate-filaments",
            "missing-filament.json",
        ],
        ["44", "6", "1.25"],
    );
}

#[test]
fn selected_plate_rejects_mismatched_refresh_filament_count_before_reading_files() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = artifact_dir();
    project(&root);
    let args = [
        "--uptodate",
        "--uptodate-filaments",
        "missing-a.json;missing-b.json",
    ];
    let native = run(Command::new(runner.bin()), &root.join("native"), &args);
    let actual = run(ares(&root), &root.join("ares"), &args);
    assert_eq!(native.status.code(), Some(254), "{root:?}");
    assert_eq!(actual.status.code(), native.status.code(), "{root:?}");
    for name in ["native", "ares"] {
        assert!(gcode_names(&root.join(name).join("out")).is_empty());
    }
    let report = |name: &str| -> serde_json::Value {
        let path = root.join(name).join("out/result.json");
        serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
    };
    let native = report("native");
    let actual = report("ares");
    assert_eq!(native["return_code"], -2, "{root:?}");
    for key in ["return_code", "error_string", "plate_index"] {
        assert_eq!(actual[key], native[key], "{key}: {root:?}");
    }
    assert!(
        String::from_utf8_lossy(&fs::read(root.join("ares/stderr")).unwrap())
            .contains("uptodate_filaments size 2 not equal to filament_count 1"),
        "{root:?}"
    );
}

fn verify(args: &[&str], expected: [&str; 3]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = artifact_dir();
    project(&root);
    let native = run(Command::new(runner.bin()), &root.join("native"), args);
    let actual = run(ares(&root), &root.join("ares"), args);
    assert!(native.status.success(), "Native failed: {root:?}");
    assert!(actual.status.success(), "Ares failed: {root:?}");
    let load = |name: &str| {
        let out = root.join(name).join("out");
        assert_eq!(gcode_names(&out), ["plate_1.gcode"], "{root:?}");
        fs::read(out.join("plate_1.gcode")).unwrap()
    };
    let native = load("native");
    let actual = load("ares");
    values(&native, expected);
    compare_ordered_bytes_generator_only(&native, &actual)
        .unwrap_or_else(|error| panic!("{root:?}: {error}"));
}

fn ares(root: &Path) -> Command {
    let installation = root.join("installation");
    let bin_dir = installation.join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    let bin = bin_dir.join("ares");
    fs::copy(env!("CARGO_BIN_EXE_ares"), &bin).unwrap();
    std::os::unix::fs::symlink(
        crate::runner::repo_root().join("OrcaSlicer/resources"),
        installation.join("resources"),
    )
    .unwrap();
    let mut command = Command::new(bin);
    command.arg("slice");
    command
}

fn run(mut command: Command, work: &Path, args: &[&str]) -> Output {
    fs::create_dir_all(work.join("out")).unwrap();
    let output = command
        .current_dir(work)
        .args(["--slice", "1", "--outputdir", "out"])
        .args(args)
        .arg("../retained.3mf")
        .output()
        .unwrap();
    fs::write(work.join("stdout"), &output.stdout).unwrap();
    fs::write(work.join("stderr"), &output.stderr).unwrap();
    output
}

fn gcode_names(out: &Path) -> Vec<String> {
    fs::read_dir(out)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "gcode"))
        .map(|path| path.file_name().unwrap().to_str().unwrap().to_owned())
        .collect()
}
