//! Upstream CLI early rejections: Utils.hpp:27,43-44,52-53;
//! OrcaSlicer.cpp:398-460 channels/result,1608,1688,3648,5723,6062-6104.
#![cfg(target_os = "linux")]

use std::{fs, path::Path, process::Command};

use crate::{native_wall_clock::result_comparison_bytes, runner::EarlyExit};

#[path = "cli_missing_project_plate.rs"]
mod missing_project_plate;
#[path = "cli_success_result.rs"]
mod success_result;

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/parity/validation/cli-exit")
        .join(name)
}

fn assert_family(case: &str, rejection: EarlyExit) {
    let artifacts = tempfile::Builder::new()
        .prefix(&format!("cli-early-{case}-"))
        .tempdir()
        .unwrap()
        .keep();
    let project = if matches!(rejection, EarlyExit::NotFound) {
        artifacts.join("definitely-missing.3mf")
    } else {
        fixture(&format!("{case}.3mf"))
    };
    assert_channels(case, &project, &artifacts, rejection);
}

/// Native's stdout, stderr and complete result.json for `project`, and Ares's against them.
fn assert_channels(case: &str, project: &Path, artifacts: &Path, rejection: EarlyExit) {
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };
    if matches!(rejection, EarlyExit::NotFound) {
        assert!(
            !project.exists(),
            "missing input must be absent before Native"
        );
    }
    let native = runner.early_cli_reference(project, rejection).unwrap();
    let hit = runner.early_cli_reference(project, rejection).unwrap();
    assert_eq!(native.directory, hit.directory);
    assert_eq!(native.stdout, hit.stdout);
    assert_eq!(native.stderr, hit.stderr);
    assert_eq!(native.result_json, hit.result_json);
    assert_eq!(native.gcode, hit.gcode);
    assert!(native.gcode.is_none());
    fs::write(artifacts.join("native.stdout"), &native.stdout).unwrap();
    fs::write(artifacts.join("native.stderr"), &native.stderr).unwrap();
    fs::write(artifacts.join("native.result.json"), &native.result_json).unwrap();
    fs::copy(
        native.directory.join("receipt.json"),
        artifacts.join("native.receipt.json"),
    )
    .unwrap();

    let gcode = artifacts.join("gcode");
    let output = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(&gcode)
        .arg(project)
        .output()
        .unwrap();
    fs::write(artifacts.join("stdout"), &output.stdout).unwrap();
    fs::write(artifacts.join("stderr"), &output.stderr).unwrap();
    fs::write(
        artifacts.join("exit"),
        format!("{:?}\n", output.status.code()),
    )
    .unwrap();
    let code = String::from_utf8(native.stdout.clone())
        .unwrap()
        .parse_line_return_code();
    assert_eq!(code, rejection.code(), "{case}: {artifacts:?}");
    // flush_and_exit returns the negative code; POSIX exposes 256+n.
    assert_eq!(
        output.status.code(),
        Some(256 + code),
        "{case}: {artifacts:?}"
    );
    assert_eq!(output.stdout, native.stdout, "{case}: stdout channel");
    assert_eq!(output.stderr, native.stderr, "{case}: stderr channel");
    assert_eq!(
        result_comparison_bytes(&fs::read(artifacts.join("result.json")).unwrap()),
        result_comparison_bytes(&native.result_json),
        "{case}: complete result.json bytes: {artifacts:?}"
    );
    assert!(!gcode.exists(), "early rejection must not export G-code");

    if matches!(rejection, EarlyExit::NotFound) {
        fs::write(project, b"input now exists").unwrap();
        let failure = runner
            .early_cli_reference(project, rejection)
            .err()
            .unwrap();
        fs::remove_file(project).unwrap();
        assert!(failure.to_string().contains("must remain absent"));
    }
}

trait ReturnCode {
    fn parse_line_return_code(&self) -> i32;
}

impl ReturnCode for String {
    fn parse_line_return_code(&self) -> i32 {
        // "run found error, return -18, exit...\n"
        let mid = self.split("return ").nth(1).unwrap();
        mid.split(',').next().unwrap().parse().unwrap()
    }
}

#[test]
fn cli_exit_reference_requires_native_not_frozen_channels() {
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "cli_exit_channels::cli_exit_values_family_matches_native",
            "--nocapture",
        ])
        .env("ARES_REQUIRE_ORCA", "1")
        .env_remove("ARES_ORCA_NATIVE_BIN")
        .output()
        .unwrap();
    let diagnostic = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        diagnostic.contains("running 1 test"),
        "selector drift: {diagnostic}"
    );
    assert!(
        !output.status.success() && diagnostic.contains("ARES_ORCA_NATIVE_BIN"),
        "frozen early CLI reference hid required Native absence: {diagnostic}"
    );
}

#[test]
fn cli_exit_values_family_matches_native() {
    assert_family("values", EarlyExit::InvalidValues);
}

#[test]
fn cli_exit_numeric_range_family_matches_native() {
    assert_family("range", EarlyExit::InvalidValues);
}

#[test]
fn cli_exit_empty_plate_family_matches_native() {
    assert_family("empty-plate", EarlyExit::EmptyPlate);
}

/// The empty plate with an unknown `support_ironing_pattern` key: the 3MF load substitutes the
/// default and reports it on stderr and as a `config_value_substituted` warning
/// (`cli_report_config_substitutions`, OrcaSlicer.cpp:520-534, :1992) before the -50 rejection
/// (:6562-6573) writes result.json.
#[test]
fn cli_exit_empty_plate_reports_a_substituted_value_like_native() {
    let artifacts = tempfile::Builder::new()
        .prefix("cli-early-substituted-")
        .tempdir()
        .unwrap()
        .keep();
    // The report names the input path, and the Native reference is cached by content and file
    // name: the project needs a path that is the same on every run.
    let directory = std::env::temp_dir().join("ares-cli-exit-substituted");
    fs::create_dir_all(&directory).unwrap();
    let project = directory.join("empty-plate-cross-zag.3mf");
    fs::write(
        &project,
        substituted(&fs::read(fixture("empty-plate.3mf")).unwrap()),
    )
    .unwrap();
    assert_channels(
        "empty-plate-substituted",
        &project,
        &artifacts,
        EarlyExit::EmptyPlate,
    );
}

/// `input` with `support_ironing_pattern` set to the unknown key `cross-zag`.
fn substituted(input: &[u8]) -> Vec<u8> {
    use std::io::{Cursor, Read, Write};
    let mut reader = zip::ZipArchive::new(Cursor::new(input)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let needle = r#""support_ironing_pattern": "rectilinear""#;
            let text = String::from_utf8(bytes).unwrap();
            assert!(
                text.contains(needle),
                "fixture must carry the rectilinear pattern"
            );
            bytes = text
                .replacen(needle, r#""support_ironing_pattern": "cross-zag""#, 1)
                .into_bytes();
        }
        writer
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::default().compression_method(file.compression()),
            )
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

#[test]
fn cli_exit_validate_family_matches_native() {
    assert_family("validate", EarlyExit::Validation);
}

#[test]
fn cli_exit_postprocess_family_matches_native() {
    assert_family("postprocess", EarlyExit::Postprocess);
}

#[test]
fn cli_exit_notfound_family_matches_native() {
    assert_family("notfound", EarlyExit::NotFound);
}

#[test]
fn cli_exit_process_compatible_family_matches_native() {
    assert_family("process-not-compatible", EarlyExit::ProcessNotCompatible);
}
