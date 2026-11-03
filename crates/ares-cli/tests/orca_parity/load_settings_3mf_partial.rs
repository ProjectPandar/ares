//! Partial `--load-settings` list witnesses (spec:
//! docs/superpowers/specs/2026-10-11-cli-load-settings-partial-lists.md)
//! — machine-only and filament-only lists keep the embedded
//! selection for the missing kinds (`OrcaSlicer.cpp:2009-2106` has
//! no completeness requirement; a missing kind contributes no
//! overlay). RED: `ProfileSelection::new` requires all three names.
#![cfg(target_os = "linux")]

use std::fs;

use crate::load_settings_3mf::load_settings_3mf_fixture;
use crate::load_settings_cli::{banner_normalized, stages_run};

/// Machine-only `--load-settings` + the compatible witness 3MF: the
/// machine overlay applies and the embedded process is retained; the
/// filament list still overlays its slot. Native slices (arrange
/// pinned), so GREEN is whole-file parity.
#[test]
fn load_settings_3mf_machine_only_whole_file() {
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };
    let directory = tempfile::Builder::new()
        .prefix("load-settings-3mf-machine-only-")
        .tempdir()
        .unwrap()
        .keep();
    let project = load_settings_3mf_fixture("compatible-input.3mf");
    let machine = load_settings_3mf_fixture("machine.json")
        .to_string_lossy()
        .into_owned();
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
            &machine,
            "--load-filaments",
            &filaments,
            // Native's default `need_arrange` re-places the model at the
            // loaded bed center; `--arrange 0` pins the embedded transform
            // (the arrange port is a deferred upstream slice).
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
            &machine,
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
        "ares rejected the machine-only --load-settings argv:\n{}",
        String::from_utf8_lossy(&output.stderr),
    );
    let ares = fs::read(directory.join("ares.gcode")).unwrap();
    assert_eq!(
        banner_normalized(&native),
        banner_normalized(&ares),
        "machine-only load-settings whole-file divergence; artifacts {}",
        directory.display(),
    );
}

/// Process-only `--load-settings` + the compatible witness 3MF: the
/// loaded process's `compatible_printers` names the EMBEDDED machine
/// (`OrcaSlicer.cpp:2597-2606` branch 3), so Native admits it and the
/// embedded machine is retained. The composition accepted this shape
/// from the start of the partial-list GREEN, so this is an always-green
/// control (no RED phase exists for it).
#[test]
fn load_settings_3mf_process_only_whole_file() {
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };
    let directory = tempfile::Builder::new()
        .prefix("load-settings-3mf-process-only-")
        .tempdir()
        .unwrap()
        .keep();
    let project = load_settings_3mf_fixture("compatible-input.3mf");
    let process = load_settings_3mf_fixture("prusa-process.json")
        .to_string_lossy()
        .into_owned();
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
            &process,
            "--load-filaments",
            &filaments,
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
            &process,
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
        "ares rejected the process-only --load-settings argv:\n{}",
        String::from_utf8_lossy(&output.stderr),
    );
    let ares = fs::read(directory.join("ares.gcode")).unwrap();
    assert_eq!(
        banner_normalized(&native),
        banner_normalized(&ares),
        "process-only load-settings whole-file divergence; artifacts {}",
        directory.display(),
    );
}

/// Filament-only `--load-filaments` (no settings files) + the
/// compatible witness 3MF: the filament overlay applies and the
/// embedded machine and process are retained. Native slices
/// (arrange pinned), so GREEN is whole-file parity.
#[test]
fn load_settings_3mf_filament_only_whole_file() {
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };
    let directory = tempfile::Builder::new()
        .prefix("load-settings-3mf-filament-only-")
        .tempdir()
        .unwrap()
        .keep();
    let project = load_settings_3mf_fixture("compatible-input.3mf");
    let filaments = load_settings_3mf_fixture("compatible-filament-0.json")
        .to_string_lossy()
        .into_owned();
    stages_run(
        &runner,
        &directory,
        &[
            "--slice",
            "0",
            "--load-filaments",
            &filaments,
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
        .args(["slice", "--load-filaments", &filaments, "-o", "ares.gcode"])
        .arg(&project)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "ares rejected the filament-only --load-filaments argv:\n{}",
        String::from_utf8_lossy(&output.stderr),
    );
    let ares = fs::read(directory.join("ares.gcode")).unwrap();
    assert_eq!(
        banner_normalized(&native),
        banner_normalized(&ares),
        "filament-only load-filaments whole-file divergence; artifacts {}",
        directory.display(),
    );
}
