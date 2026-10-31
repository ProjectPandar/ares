//! `--load-settings`/`--load-filaments` direct-load witnesses (spec:
//! docs/superpowers/specs/2026-10-11-cli-load-settings-inherits.md) —
//! the CLI path never resolves `inherits`
//! (`OrcaSlicer.cpp:1953-2010` loads each file's fields directly);
//! `inherits` is a value (the system name for `from: user` files),
//! never a link. RED: Ares resolves import chains today.
#![cfg(target_os = "linux")]

use std::{fs, process::Command};

use crate::load_settings_3mf::load_settings_3mf_fixture;
use crate::load_settings_cli::{banner_normalized, stages_run};

/// RED A: a `from: user` machine whose `inherits` names a preset that is
/// not imported loads directly (Native slices); the fixture points
/// `inherits` at the machine's own system identity so the system-name
/// semantics match the from:system witness and the change isolates the
/// load boundary. Ares today fails parent resolution.
#[test]
fn load_settings_3mf_user_inherits_whole_file() {
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };
    let directory = tempfile::Builder::new()
        .prefix("load-settings-3mf-user-inherits-")
        .tempdir()
        .unwrap()
        .keep();
    let project = load_settings_3mf_fixture("compatible-input.3mf");
    let machine_process = [
        load_settings_3mf_fixture("user-machine.json"),
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
            "--arrange",
            "0",
            "--outputdir",
            "native-output",
            project.to_str().unwrap(),
        ],
    );
    let native = fs::read(directory.join("native-output/plate_1.gcode")).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ares"))
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
        "ares rejected the user-inherits --load-settings argv:\n{}",
        String::from_utf8_lossy(&output.stderr),
    );
    let ares = fs::read(directory.join("ares.gcode")).unwrap();
    assert_eq!(
        banner_normalized(&native),
        banner_normalized(&ares),
        "user-inherits load-settings whole-file divergence; artifacts {}",
        directory.display(),
    );
}

/// RED B (no-chain control): the child filament declares `inherits`
/// naming a parent that is NOT listed and omits `filament_flow_ratio`.
/// Native direct-loads the single slot (chain reads commented out,
/// `OrcaSlicer.cpp:1987,1991`) and slices; a chain-resolving
/// implementation fails with a missing-parent error. The spec's
/// same-list pair control is unreachable as a whole-file witness: the
/// reference Native 2.4.2 deterministically SIGSEGVs on ANY two-entry
/// `--load-filaments` list (verified with unrelated pairs, both
/// orders); `parent-filament.json` stays as the documented pair
/// counterpart of that investigation.
#[test]
fn load_settings_3mf_filament_inherits_no_chain() {
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };
    let directory = tempfile::Builder::new()
        .prefix("load-settings-3mf-filament-inherits-")
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
    let filaments = load_settings_3mf_fixture("child-filament.json")
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
            "--arrange",
            "0",
            "--outputdir",
            "native-output",
            project.to_str().unwrap(),
        ],
    );
    let native = fs::read(directory.join("native-output/plate_1.gcode")).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ares"))
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
        "ares rejected the filament-inherits --load-filaments argv:\n{}",
        String::from_utf8_lossy(&output.stderr),
    );
    let ares = fs::read(directory.join("ares.gcode")).unwrap();
    assert_eq!(
        banner_normalized(&native),
        banner_normalized(&ares),
        "filament-inherits no-chain whole-file divergence; artifacts {}",
        directory.display(),
    );
}
