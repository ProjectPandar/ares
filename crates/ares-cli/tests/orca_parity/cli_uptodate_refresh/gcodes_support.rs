//! Shared helpers for retained preset G-code refresh checks.
use std::{
    fs,
    io::{Cursor, Write},
    path::{Path, PathBuf},
    process::Command,
};

use ares_core::decode_config_value_by_key;
use serde_json::{Map, Value};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use super::{OrcaRunner, banner_normalized, settings};
use crate::{load_settings_cli::stages_run, runner::repo_root};

pub(super) const COLUMNS: &str = "different_settings_to_system";
pub(super) const SKIP: &str = "--skip-modified-gcodes";
pub(super) const REFRESH: &[&str] = &[
    "--uptodate",
    "--uptodate-settings",
    "machine.json;process.json",
    "--uptodate-filaments",
    "filament.json",
];

pub(super) fn stock_resources() -> PathBuf {
    repo_root().join("OrcaSlicer/resources")
}

pub(super) fn read_json(root: &Path, name: &str) -> Value {
    serde_json::from_slice(&fs::read(root.join(name)).unwrap()).unwrap()
}

pub(super) fn edit_project(root: &Path, edit: impl FnOnce(&mut Map<String, Value>)) {
    let path = root.join("retained.3mf");
    let input = fs::read(&path).unwrap();
    repack_file(&input, &path, edit);
}

pub(super) fn repack_file(source: &[u8], dest: &Path, edit: impl FnOnce(&mut Map<String, Value>)) {
    let mut saved = settings(source).unwrap();
    edit(&mut saved);
    let mut archive = ZipArchive::new(Cursor::new(source)).unwrap();
    let mut output = ZipWriter::new(fs::File::create(dest).unwrap());
    for index in 0..archive.len() {
        let entry = archive.by_index(index).unwrap();
        if entry.name() == "Metadata/project_settings.config" {
            output
                .start_file(entry.name(), SimpleFileOptions::default())
                .unwrap();
            output
                .write_all(&serde_json::to_vec(&saved).unwrap())
                .unwrap();
        } else {
            output.raw_copy_file(entry).unwrap();
        }
    }
    output.finish().unwrap();
}

pub(super) fn compare_toggle(
    runner: &OrcaRunner,
    root: &Path,
    project_name: &str,
    argv: &[&str],
    ares_only: &[&str],
) -> Vec<u8> {
    let off = pair_on((runner, root), project_name, "omitted", argv, ares_only);
    let on = pair_on(
        (runner, root),
        project_name,
        "enabled",
        &[argv, &[SKIP]].concat(),
        ares_only,
    );
    same(&off, &on, root);
    on
}

pub(super) fn pair(
    runner: &OrcaRunner,
    root: &Path,
    row: &str,
    argv: &[&str],
    ares_only: &[&str],
) -> Vec<u8> {
    pair_on((runner, root), "retained.3mf", row, argv, ares_only)
}

pub(super) fn pair_on(
    (runner, root): (&OrcaRunner, &Path),
    project_name: &str,
    row: &str,
    argv: &[&str],
    ares_only: &[&str],
) -> Vec<u8> {
    let native_dir = format!("native-{row}");
    let native_args = [
        ["--slice", "0", "--arrange", "0", "--outputdir", &native_dir].as_slice(),
        argv,
        &[project_name],
    ]
    .concat();
    stages_run(runner, root, &native_args);
    let native = fs::read(root.join(&native_dir).join("plate_1.gcode")).unwrap();

    let ares_dir = format!("ares-{row}");
    let out = root.join(&ares_dir);
    fs::create_dir(&out).unwrap();
    let ares_argv = [argv, ares_only].concat();
    let gcode_path = out.join("plate_1.gcode");
    let exact = [
        &[env!("CARGO_BIN_EXE_ares"), "slice"][..],
        &ares_argv,
        &["-o", gcode_path.to_str().unwrap(), project_name],
    ]
    .concat();
    fs::write(out.join("argv.json"), serde_json::to_vec(&exact).unwrap()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ares"))
        .current_dir(root)
        .arg("slice")
        .args(&ares_argv)
        .arg("-o")
        .arg(&gcode_path)
        .arg(project_name)
        .output()
        .unwrap();
    fs::write(out.join("stdout"), &output.stdout).unwrap();
    fs::write(out.join("stderr"), &output.stderr).unwrap();
    fs::write(out.join("status"), output.status.to_string()).unwrap();
    assert!(
        output.status.success(),
        "Ares {row}: {}; {root:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    same(&native, &fs::read(gcode_path).unwrap(), &out);
    native
}

pub(super) fn same(left: &[u8], right: &[u8], dir: &Path) {
    assert!(
        banner_normalized(left) == banner_normalized(right),
        "whole G-code differs; {dir:?}"
    );
}

pub(super) fn columns(gcode: &[u8]) -> Vec<String> {
    decode_strings(gcode, COLUMNS)
}

pub(super) fn decode_string(gcode: &[u8], key: &str) -> String {
    serde_json::from_str(&decode_config_value_by_key(key, config(gcode, key)).unwrap()).unwrap()
}

pub(super) fn decode_strings(gcode: &[u8], key: &str) -> Vec<String> {
    serde_json::from_str(&decode_config_value_by_key(key, config(gcode, key)).unwrap()).unwrap()
}

pub(super) fn config<'a>(gcode: &'a [u8], key: &str) -> &'a str {
    let prefix = format!("; {key} = ");
    std::str::from_utf8(gcode)
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap()
}
