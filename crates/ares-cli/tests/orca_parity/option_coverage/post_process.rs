//! OrcaSlicer.cpp:1682-1690: embedded vector size, not trimmed script text.
#![cfg(target_os = "linux")]

use std::{
    fs,
    io::{Cursor, Read, Write},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use crate::{
    cli_retained_input::normalize_generator,
    native_wall_clock::result_comparison_bytes,
    runner::{EarlyExit, OrcaRunner, stages},
};
use serde_json::{Value, json};

#[test]
fn post_process_empty_string_vector_is_cli_rejection() {
    compare(Some(json!([""])));
}

#[test]
fn post_process_nonempty_script_is_cli_rejection_without_execution() {
    compare(None);
}

#[test]
fn post_process_empty_vector_allows_complete_cli_export() {
    compare(Some(json!([])));
}

fn input(directory: &Path, scripts: &Value) -> Vec<u8> {
    let original = fs::read(
        crate::runner::repo_root().join("tests/parity/validation/cli-exit/postprocess.3mf"),
    )
    .unwrap();
    let mut before = zip::ZipArchive::new(Cursor::new(&original)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..before.len() {
        let mut file = before.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(settings["post_process"], json!([""]));
            settings["post_process"] = scripts.clone();
            bytes = serde_json::to_vec_pretty(&settings).unwrap();
        }
        output
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    let rewritten = output.finish().unwrap().into_inner();
    let mut expected = crate::runner::application::settings(&original).unwrap();
    assert_eq!(expected.remove("post_process"), Some(json!([""])));
    let mut actual = crate::runner::application::settings(&rewritten).unwrap();
    assert_eq!(actual.remove("post_process"), Some(scripts.clone()));
    assert_eq!(actual, expected);
    let mut after = zip::ZipArchive::new(Cursor::new(&rewritten)).unwrap();
    assert_eq!(before.len(), after.len());
    for index in 0..before.len() {
        let mut a = before.by_index(index).unwrap();
        if a.name() == "Metadata/project_settings.config" {
            continue;
        }
        let mut b = after.by_name(a.name()).unwrap();
        let mut aa = Vec::new();
        let mut bb = Vec::new();
        a.read_to_end(&mut aa).unwrap();
        b.read_to_end(&mut bb).unwrap();
        assert_eq!(aa, bb, "{}", a.name());
    }
    fs::write(directory.join("original.3mf"), original).unwrap();
    fs::write(directory.join("input.3mf"), &rewritten).unwrap();
    rewritten
}

fn compare(scripts: Option<Value>) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    compare_with(&runner, scripts);
}

pub(super) fn compare_with(runner: &OrcaRunner, scripts: Option<Value>) -> (PathBuf, Value) {
    let directory = tempfile::Builder::new()
        .prefix("post-process-cli-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let script = directory.join("must-not-run.sh");
    let marker = directory.join("must-not-run.sh.marker");
    fs::write(&script, b"#!/bin/sh\nprintf executed > \"$0.marker\"\n").unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o700)).unwrap();
    let scripts = scripts.unwrap_or_else(|| json!([script.to_str().unwrap()]));
    let bytes = input(&directory, &scripts);
    let input = directory.join("input.3mf");
    let rejected = !scripts.as_array().unwrap().is_empty();
    let native = directory.join("native");
    fs::create_dir(&native).unwrap();
    let (code, stdout, stderr, result, gcode) = if rejected {
        let capture = runner
            .early_cli_reference(&input, EarlyExit::Postprocess)
            .unwrap();
        fs::copy(
            capture.directory.join("receipt.json"),
            directory.join("native.receipt.json"),
        )
        .unwrap();
        assert!(capture.gcode.is_none());
        (
            237,
            capture.stdout,
            capture.stderr,
            capture.result_json,
            None,
        )
    } else {
        let args = vec![
            "--slice".into(),
            "0".into(),
            "--outputdir".into(),
            native.to_str().unwrap().into(),
            input.to_str().unwrap().into(),
        ];
        fs::write(
            directory.join("native.args.json"),
            serde_json::to_vec(&args).unwrap(),
        )
        .unwrap();
        stages::run(runner.bin(), args, stages::Stage::Slice, &directory).unwrap();
        let captures = fs::read_dir(&directory)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| {
                path.file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with("command-")
            })
            .collect::<Vec<_>>();
        assert_eq!(captures.len(), 1);
        let status: Value =
            serde_json::from_slice(&fs::read(captures[0].join("status.json")).unwrap()).unwrap();
        assert_eq!(status["code"], 0);
        (
            0,
            fs::read(captures[0].join("stdout.bin")).unwrap(),
            fs::read(captures[0].join("stderr.bin")).unwrap(),
            fs::read(native.join("result.json")).unwrap(),
            Some(fs::read_to_string(native.join("plate_1.gcode")).unwrap()),
        )
    };
    fs::write(directory.join("native.stdout"), &stdout).unwrap();
    fs::write(directory.join("native.stderr"), &stderr).unwrap();
    fs::write(directory.join("native.result.json"), &result).unwrap();
    assert!(
        !marker.exists(),
        "Native CLI must not execute external scripts"
    );
    let actual = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(directory.join("ares.gcode"))
        .arg(&input)
        .output()
        .unwrap();
    fs::write(directory.join("ares.stdout"), &actual.stdout).unwrap();
    fs::write(directory.join("ares.stderr"), &actual.stderr).unwrap();
    fs::write(
        directory.join("ares.exit"),
        format!("{:?}\n", actual.status.code()),
    )
    .unwrap();
    assert_eq!(fs::read(&input).unwrap(), bytes);
    assert!(!marker.exists(), "Ares must not execute external scripts");
    assert_eq!(actual.status.code(), Some(code), "{directory:?}");
    if rejected {
        let metadata: Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(metadata["return_code"], -19);
        assert_eq!(metadata["plate_index"], 0);
        assert_eq!(actual.stdout, stdout);
        assert_eq!(actual.stderr, stderr);
        assert_eq!(
            result_comparison_bytes(&fs::read(directory.join("result.json")).unwrap()),
            result_comparison_bytes(&result)
        );
        assert!(!directory.join("ares.gcode").exists());
    } else {
        let native = gcode.unwrap();
        assert!(!native.is_empty());
        fs::write(directory.join("native.gcode"), &native).unwrap();
        let ares = fs::read_to_string(directory.join("ares.gcode")).unwrap();
        assert_eq!(
            normalize_generator(&ares, "Ares"),
            normalize_generator(&native, "OrcaSlicer")
        );
    }
    (directory, scripts)
}
