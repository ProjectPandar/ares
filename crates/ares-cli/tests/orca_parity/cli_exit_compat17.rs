//! Same-argv `-17` adjudication branches 1–2 under `--load-settings`
//! (spec: docs/superpowers/specs/2026-10-10-cli-load-settings-compat17-branches.md;
//! upstream `OrcaSlicer.cpp:2575-2620` + `:2650-2654`, `Utils.hpp:42`).
//! Branch 1 (this slice): loaded machine + loaded process — `new_print_compatible_printers`
//! must contain `new_printer_system_name` (`:2575-2584`). Branch 2: loaded
//! machine only — `current_print_compatible_printers` must contain it, or a
//! `machine_switch` (`new_printer_name != current_printer_name`) accepts. Branch-2 cases ship once the composite admits partial lists.
//! (`:2585-2594,:2624-2647` semantics are in the adjudication; its REDs/! wait for the partial-list composite — worker5 boundary; `-16` upward! gate stays deferred).
#![cfg(target_os = "linux")]

use std::fs;

use crate::runner::EarlyExit;

fn fixture(name: &str) -> std::path::PathBuf {
    crate::runner::repo_root()
        .join("tests/parity/validation/cli-exit/compat17")
        .join(name)
}

fn names(paths: &[&str]) -> Vec<String> {
    paths
        .iter()
        .map(|name| fixture(name).to_string_lossy().into_owned())
        .collect()
}

fn parse_return_code(stdout: &[u8]) -> i32 {
    // "run found error, return -17, exit...\n"
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

/// `-17` family comparison under `--load-settings` (same argv on both
/// sides): exit 239, stdout `run found error, return -17`, empty
/// stderr, complete result.json bytes — `OrcaSlicer.cpp:2650-2654`.
fn assert_load_settings_reject(case: &str, settings: &[&str], filaments: &[&str]) {
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };
    let artifacts = tempfile::Builder::new()
        .prefix(&format!("compat17-{case}-"))
        .tempdir()
        .unwrap()
        .keep();
    let project = fixture("process-not-compatible.3mf");
    let settings = names(settings);
    let filaments = names(filaments);
    let rejection = EarlyExit::ProcessNotCompatible;
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
    assert_eq!(code, -17, "native return code: {artifacts:?}");
    assert_eq!(
        output.status.code(),
        Some(256 - 17),
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

/// Branch-1 reject (`:2575-2584`): machine + process loaded, the
/// process's `compatible_printers` excludes the machine's system name.
#[test]
fn compat17_branch1_reject() {
    assert_load_settings_reject(
        "branch1-reject",
        &[
            "machine-same-name.json",
            "process-compatible-excluding.json",
        ],
        &["filament-generic-pla.json"],
    );
}

/// Positive accept witnesses: both sides must produce output.
fn assert_load_settings_accept(case: &str, settings: &[&str], filaments: &[&str]) {
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };
    let directory = tempfile::Builder::new()
        .prefix(&format!("compat17-{case}-"))
        .tempdir()
        .unwrap()
        .keep();
    let project = fixture("process-not-compatible.3mf");
    let machine_process = names(settings).join(";");
    let filaments = names(filaments).join(";");
    crate::runner::stages::run(
        runner.bin(),
        [
            "--slice",
            "0",
            "--load-settings",
            &machine_process,
            "--load-filaments",
            &filaments,
            "--arrange",
            "0",
            "--outputdir",
            "native-output",
            project.to_str().unwrap(),
        ]
        .iter()
        .map(|arg| arg.to_string()),
        crate::runner::stages::Stage::Slice,
        &directory,
    )
    .unwrap();
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
    fs::write(directory.join("stdout"), &output.stdout).unwrap();
    fs::write(directory.join("stderr"), &output.stderr).unwrap();
    assert!(
        output.status.success(),
        "ares rejected the compatible argv: {}",
        directory.display(),
    );
    let ares = fs::read(directory.join("ares.gcode")).unwrap();
    let normalized = |bytes: &[u8]| -> Vec<u8> {
        bytes
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.starts_with(b"; generated by "))
            .fold(Vec::new(), |mut acc, line| {
                acc.extend_from_slice(line);
                acc.push(b'\n');
                acc
            })
    };
    assert_eq!(
        normalized(&native),
        normalized(&ares),
        "whole-file divergence; artifacts {}",
        directory.display(),
    );
}

/// Machine+process reject through the switch path (`:2624-2647`):
/// the machine file names a different printer and the process's
/// `compatible_printers` excludes its system name, so upstream still
/// exits `-17` — the `:2625-2627` auto-accept needs a machine-only
/// list, and the upward `-16` sub-path stays deferred (spec
/// `2026-10-11-cli-load-settings-compat17-branch3-upward16.md`).
/// Regression control: on the merged-raw implementation the `-17`
/// verdict holds via a coincidental equal-name path; the deferred
/// embedded-raw correction keeps the same verdict for the right
/// reason — this test pins both.
#[test]
fn compat17_branch1_switch_reject() {
    assert_load_settings_reject(
        "branch1-switch-reject",
        &[
            "machine-different-name.json",
            "process-compatible-excluding.json",
        ],
        &["filament-generic-pla.json"],
    );
}

/// Branch-1 accept: the process's `compatible_printers` contains the
/// machine's system name — both sides slice (`:2576-2583`).
#[test]
fn compat17_branch1_accept() {
    assert_load_settings_accept(
        "branch1-accept",
        &[
            "machine-same-name.json",
            "process-compatible-including.json",
        ],
        &["filament-generic-pla.json"],
    );
}

/// Branch-2 machine-switch accept (`:2624-2647`): machine file names a
/// different printer than embedded → upstream sets compatible without
/// consulting `current_print_compatible_printers`; the fixture's empty
/// `upward_compatible_machine` keeps the deferred `-16` path out.
#[test]
fn compat17_branch2_machine_switch_accept() {
    assert_load_settings_accept(
        "branch2-switch",
        &["machine-different-name.json"],
        &["filament-generic-pla.json"],
    );
}

/// Branch-3 accept (`:2595-2605`): process only and its
/// `compatible_printers` contains the embedded system name — both
/// sides slice. The `-rel` clone keeps the embedded relative-E
/// addressing (`use_relative_e_distances:1`): the shared process
/// clones pin `0`, which against the embedded `before_layer_change_
/// gcode`'s `G92 E0` makes native exit `-51` after adjudication —
/// harmless for reject cases but masking the accept path.
#[test]
fn compat17_branch3_accept() {
    assert_load_settings_accept(
        "branch3-accept",
        &["process-compatible-including-rel.json"],
        &["filament-generic-pla.json"],
    );
}
