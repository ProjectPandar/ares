//! 3MF `--load-settings` witnesses (spec:
//! docs/superpowers/specs/2026-10-10-cli-3mf-load-settings.md) — the
//! `-19` embedded-post_process rejection, the compatible whole-file
//! control, and the `-5`/`-19` read-ordering RED.
#![cfg(target_os = "linux")]

#[path = "load_settings_3mf_filament_count.rs"]
mod filament_count;

use std::fs;

use crate::load_settings_cli::{banner_normalized, stages_run};

pub(crate) fn load_settings_3mf_fixture(name: &str) -> std::path::PathBuf {
    crate::runner::repo_root()
        .join("tests/parity/validation/load-settings-3mf")
        .join(name)
}

pub(crate) fn parse_return_code(stdout: &[u8]) -> i32 {
    // "run found error, return -19, exit...\n"
    let text = std::str::from_utf8(stdout).unwrap();
    text.split("return ")
        .nth(1)
        .unwrap()
        .split(',')
        .next()
        .unwrap()
        .parse()
        .unwrap()
}

/// The `-19` rejection witness (`OrcaSlicer.cpp:1682-1690`): the embedded
/// `post_process: [""]` is a non-empty array, and the check lives in the
/// 3MF load block before `--load-settings` processing, so Native rejects
/// while the Ares guard (`main.rs`) exits 1.
#[test]
fn load_settings_3mf_post_process_reject() {
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };
    let artifacts = tempfile::Builder::new()
        .prefix("load-settings-3mf-reject-")
        .tempdir()
        .unwrap()
        .keep();
    let project = load_settings_3mf_fixture("input.3mf");
    let settings = ["machine.json", "process.json"]
        .into_iter()
        .map(|name| {
            load_settings_3mf_fixture(name)
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    let filaments = [load_settings_3mf_fixture("filament-0.json")
        .to_string_lossy()
        .into_owned()];
    let rejection = crate::runner::EarlyExit::Postprocess;
    let native = runner
        .load_settings_early_reference(&project, &settings, &filaments, rejection)
        .unwrap();
    let hit = runner
        .load_settings_early_reference(&project, &settings, &filaments, rejection)
        .unwrap();
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
        .current_dir(&artifacts)
        .args([
            "slice",
            "--load-settings",
            &settings.join(";"),
            "--load-filaments",
            &filaments.join(";"),
            "-o",
        ])
        .arg(&gcode)
        .arg(&project)
        .output()
        .unwrap();
    fs::write(artifacts.join("stdout"), &output.stdout).unwrap();
    fs::write(artifacts.join("stderr"), &output.stderr).unwrap();
    fs::write(
        artifacts.join("exit"),
        format!("{:?}\n", output.status.code()),
    )
    .unwrap();
    let code = parse_return_code(&native.stdout);
    assert_eq!(code, -19, "native return code: {artifacts:?}");
    // flush_and_exit returns the negative code; POSIX exposes 256+n.
    assert_eq!(
        output.status.code(),
        Some(256 - 19),
        "ares channels: {artifacts:?}"
    );
    assert_eq!(output.stdout, native.stdout, "stdout channel");
    assert_eq!(output.stderr, native.stderr, "stderr channel");
    assert_eq!(
        crate::native_wall_clock::result_comparison_bytes(
            &fs::read(artifacts.join("result.json")).unwrap(),
        ),
        crate::native_wall_clock::result_comparison_bytes(&native.result_json),
        "complete result.json bytes: {artifacts:?}"
    );
    assert!(!gcode.exists(), "early rejection must not export G-code");
}

/// The compatible forward case: the witness 3MF with the embedded
/// `post_process` emptied; the loaded process's `compatible_printers`
/// names the loaded machine (`OrcaSlicer.cpp:2576-2585` branch 1), so
/// Native slices and the whole-file output must match (guard-normalized).
#[test]
fn load_settings_3mf_compatible_whole_file() {
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };
    let directory = tempfile::Builder::new()
        .prefix("load-settings-3mf-compatible-")
        .tempdir()
        .unwrap()
        .keep();
    let project = load_settings_3mf_fixture("compatible-input.3mf");
    let machine_process = [
        load_settings_3mf_fixture("machine.json"),
        load_settings_3mf_fixture("process.json"),
    ]
    .iter()
    .map(|path| path.to_string_lossy().into_owned())
    .collect::<Vec<_>>()
    .join(";");
    let filaments = load_settings_3mf_fixture("compatible-filament-0.json")
        .to_string_lossy()
        .into_owned();
    stages_run(
        &runner,
        &directory,
        &[
            "--slice",
            "0",
            "--load-settings",
            &machine_process,
            "--load-filaments",
            &filaments,
            // Native's default `need_arrange` re-places the model at the
            // loaded bed center; `--arrange 0` pins the embedded transform
            // (the arrange port is a deferred upstream slice — same
            // convention as the STL case and `runner/profile_entry.rs`).
            "--arrange",
            "0",
            "--outputdir",
            "native-output",
            project.to_str().unwrap(),
        ],
    );
    let native = fs::read(directory.join("native-output/plate_1.gcode")).unwrap();
    let output = crate::runner::application::cli_command()
        .current_dir(&directory)
        .args([
            "slice",
            "--load-settings",
            &machine_process,
            "--load-filaments",
            &filaments,
            "-o",
            "ares.gcode",
        ])
        .arg(&project)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "ares rejected the 3MF --load-settings argv:\n{}",
        String::from_utf8_lossy(&output.stderr),
    );
    let ares = fs::read(directory.join("ares.gcode")).unwrap();
    assert_eq!(
        banner_normalized(&native),
        banner_normalized(&ares),
        "3MF+load-settings whole-file divergence; artifacts {}",
        directory.display(),
    );
}

/// The `-5`/`-19` read-ordering witness (`OrcaSlicer.cpp:1682-1690`
/// before `:1953-2024`): the 3MF load block admits the project before
/// the producer reads the preset files, so an absent settings entry
/// plus an embedded `post_process` still rejects `-19`. The Ares CLI
/// adapter reads the files first (`main.rs`), so today the missing-file
/// legacy failure (exit 1) shadows the `-19` — the ordering RED.
#[test]
fn load_settings_3mf_missing_file_post_process_order() {
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };
    let artifacts = tempfile::Builder::new()
        .prefix("load-settings-3mf-order-")
        .tempdir()
        .unwrap()
        .keep();
    let project = load_settings_3mf_fixture("input.3mf");
    let absent_machine = artifacts.join("machine.json");
    let settings = [
        absent_machine.to_string_lossy().into_owned(),
        load_settings_3mf_fixture("process.json")
            .to_string_lossy()
            .into_owned(),
    ];
    let filaments = [load_settings_3mf_fixture("filament-0.json")
        .to_string_lossy()
        .into_owned()];
    let rejection = crate::runner::EarlyExit::MissingSettingsFile;
    let native = runner
        .load_settings_early_reference(&project, &settings, &filaments, rejection)
        .unwrap();
    assert_eq!(native.gcode, None);
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
        .current_dir(&artifacts)
        .args([
            "slice",
            "--load-settings",
            &settings.join(";"),
            "--load-filaments",
            &filaments.join(";"),
            "-o",
        ])
        .arg(&gcode)
        .arg(&project)
        .output()
        .unwrap();
    fs::write(artifacts.join("stdout"), &output.stdout).unwrap();
    fs::write(artifacts.join("stderr"), &output.stderr).unwrap();
    fs::write(
        artifacts.join("exit"),
        format!("{:?}\n", output.status.code()),
    )
    .unwrap();
    let code = parse_return_code(&native.stdout);
    assert_eq!(code, -19, "native return code: {artifacts:?}");
    // flush_and_exit returns the negative code; POSIX exposes 256+n.
    assert_eq!(
        output.status.code(),
        Some(256 - 19),
        "ares channels: {artifacts:?}"
    );
    assert_eq!(output.stdout, native.stdout, "stdout channel");
    assert_eq!(output.stderr, native.stderr, "stderr channel");
    assert_eq!(
        crate::native_wall_clock::result_comparison_bytes(
            &fs::read(artifacts.join("result.json")).unwrap(),
        ),
        crate::native_wall_clock::result_comparison_bytes(&native.result_json),
        "complete result.json bytes: {artifacts:?}"
    );
    assert!(!gcode.exists(), "early rejection must not export G-code");
}
