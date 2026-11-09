//! Runtime Native inheritance admission and whole-output witnesses.
#![cfg(target_os = "linux")]

use std::fs;

use serde_json::Value;

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

/// Compare the original unlisted-parent filament against live Native admission.
/// OrcaSlicer.cpp:2113–2161 routes user files to PresetBundle resolution;
/// PresetBundle.cpp:556–650 binds a source file to its loaded preset identity.
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
    let native = crate::runner::stages::run(
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
        .into_iter()
        .map(str::to_owned),
        crate::runner::stages::Stage::Slice,
        &directory,
    );
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
    fs::write(directory.join("ares.stdout"), &output.stdout).unwrap();
    fs::write(directory.join("ares.stderr"), &output.stderr).unwrap();
    fs::write(
        directory.join("ares.exit.json"),
        serde_json::to_vec(&output.status.code()).unwrap(),
    )
    .unwrap();
    if let Err(native) = native {
        assert_eq!(native.kind, crate::runner::stages::FailureKind::Process);
        let evidence = native.evidence.unwrap();
        let status: Value =
            serde_json::from_slice(&fs::read(evidence.join("status.json")).unwrap()).unwrap();
        assert!(
            status["code"].is_i64(),
            "Native process did not exit normally"
        );
        assert_eq!(
            serde_json::json!(output.status.code()),
            status["code"],
            "Native/Ares exit code mismatch; artifacts {}",
            directory.display(),
        );
        assert_eq!(
            output.stdout,
            fs::read(evidence.join("stdout.bin")).unwrap(),
            "Native/Ares stdout mismatch; artifacts {}",
            directory.display(),
        );
        assert_eq!(
            output.stderr,
            fs::read(evidence.join("stderr.bin")).unwrap(),
            "Native/Ares stderr mismatch; artifacts {}",
            directory.display(),
        );
        assert_eq!(
            directory.join("ares.gcode").exists(),
            directory.join("native-output/plate_1.gcode").exists(),
            "Native/Ares G-code presence mismatch",
        );
        return;
    }
    let native = fs::read(directory.join("native-output/plate_1.gcode")).unwrap();
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
