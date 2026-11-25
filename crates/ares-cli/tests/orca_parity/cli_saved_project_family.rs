#![cfg(target_os = "linux")]

use std::{fs, os::unix::fs::symlink, path::Path, process::Command};

use super::cli_non_bbl_3mf::without_generator_tags;
use crate::{
    artifacts,
    load_settings_cli::banner_normalized,
    runner::{
        OrcaRunner,
        stages::{self, Stage},
    },
};

const CUBE: &[u8] = include_bytes!("../../../../tests/parity/cube10.stl");
const PRESETS: [(&str, &[u8]); 4] = [
    (
        "machine.json",
        include_bytes!("../../../../tests/parity/cli-saved-project-family/machine.json"),
    ),
    (
        "process.json",
        include_bytes!("../../../../tests/parity/cli-saved-project-family/process.json"),
    ),
    (
        "pla.json",
        include_bytes!("../../../../tests/parity/cli-saved-project-family/pla.json"),
    ),
    (
        "petg.json",
        include_bytes!("../../../../tests/parity/cli-saved-project-family/petg.json"),
    ),
];

#[test]
fn saved_cli_bbl_project_composes_retained_filament_slots() {
    compare_saved_project(true);
}

#[test]
fn saved_cli_non_bbl_project_keeps_embedded_filament_slots() {
    compare_saved_project(false);
}

fn compare_saved_project(bbl_family: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("cli-saved-project-family-")
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let project = export_cube(&root, &runner);
    let project = if bbl_family {
        project
    } else {
        without_generator_tags(&project)
    };
    fs::write(root.join("project.3mf"), project).unwrap();

    let installation = root.join("installation");
    let ares = installation.join("bin/ares");
    fs::create_dir_all(ares.parent().unwrap()).unwrap();
    fs::copy(env!("CARGO_BIN_EXE_ares"), &ares).unwrap();
    symlink(
        crate::runner::repo_root().join("OrcaSlicer/resources"),
        installation.join("resources"),
    )
    .unwrap();

    let expected = slice(runner.bin(), &root, "native");
    let actual = slice(&ares, &root, "ares");
    let first_difference = expected
        .split(|&byte| byte == b'\n')
        .zip(actual.split(|&byte| byte == b'\n'))
        .position(|(native, ares)| native != ares)
        .map(|index| index + 1);
    assert!(
        actual == expected,
        "whole G-code differs at line {first_difference:?} in {root:?}"
    );
}

fn export_cube(root: &Path, runner: &OrcaRunner) -> Vec<u8> {
    let export = root.join("export");
    fs::create_dir(&export).unwrap();
    fs::write(export.join("cube.stl"), CUBE).unwrap();
    for (name, preset) in PRESETS {
        fs::write(export.join(name), preset).unwrap();
    }
    stages::run(
        runner.bin(),
        [
            "--debug",
            "0",
            "--export-3mf",
            "cube.3mf",
            "--load-settings",
            "machine.json;process.json",
            "--load-filaments",
            "pla.json;petg.json",
            "cube.stl",
        ]
        .map(str::to_owned),
        Stage::Export,
        &export,
    )
    .unwrap_or_else(|error| panic!("Native export: {error}; {export:?}"));
    fs::read(export.join("cube.3mf")).unwrap()
}

fn slice(bin: &Path, root: &Path, engine: &str) -> Vec<u8> {
    let dir = root.join(engine);
    fs::create_dir_all(dir.join("out")).unwrap();
    let output = Command::new(bin)
        .current_dir(&dir)
        .args([
            "--debug",
            "0",
            "--slice",
            "1",
            "--arrange",
            "0",
            "--outputdir",
            "out",
        ])
        .arg(root.join("project.3mf"))
        .output()
        .unwrap();
    fs::write(dir.join("stdout"), &output.stdout).unwrap();
    fs::write(dir.join("stderr"), &output.stderr).unwrap();
    assert!(output.status.success(), "slice failed in {dir:?}");
    banner_normalized(&fs::read(dir.join("out/plate_1.gcode")).unwrap())
}
