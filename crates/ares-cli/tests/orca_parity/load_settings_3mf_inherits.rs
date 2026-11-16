//! Runtime Native source-admission witnesses through the public core API.
#![cfg(target_os = "linux")]

use std::{fs, path::Path};

use ares_core::{
    ConfigOverrides, GenerationMetadata, LoadSettingsFile, LoadSettingsSliceRequest, Placement,
    SlicerContext,
};
use serde_json::Value;

use crate::load_settings_3mf::load_settings_3mf_fixture;

async fn public_api(
    project: &[u8],
    settings: &[(&Path, &str)],
    filaments: &[(&Path, &str)],
) -> Result<Vec<u8>, ares_core::SliceError> {
    let settings = settings
        .iter()
        .map(|(path, json)| LoadSettingsFile {
            label: path.to_str().unwrap(),
            json,
        })
        .collect::<Vec<_>>();
    let filaments = filaments
        .iter()
        .map(|(path, json)| LoadSettingsFile {
            label: path.to_str().unwrap(),
            json,
        })
        .collect::<Vec<_>>();
    let mut sliced_plate = ares_core::SlicedPlate::default();
    ares_core::slice_with_load_settings(
        project,
        LoadSettingsSliceRequest {
            filename: "compatible-input.3mf",
            catalog_json: None,
            uptodate: false,
            settings: &settings,
            filaments: &filaments,
            plate: None,
            overrides: &ConfigOverrides::default(),
            placement: Placement::None,
            sliced_plate: &mut sliced_plate,
        },
        GenerationMetadata::deterministic(2026, 11, 16, 0, 0, 0),
        SlicerContext::new(0),
    )
    .await
}

fn native_rejection(
    runner: &crate::runner::OrcaRunner,
    directory: &Path,
    machine_process: &str,
    filaments: &str,
    project: &Path,
) -> (Value, Vec<u8>, Vec<u8>) {
    let failure = crate::runner::stages::run(
        runner.bin(),
        [
            "--slice",
            "0",
            "--load-settings",
            machine_process,
            "--load-filaments",
            filaments,
            "--arrange",
            "0",
            "--outputdir",
            "native-output",
            project.to_str().unwrap(),
        ]
        .into_iter()
        .map(str::to_owned),
        crate::runner::stages::Stage::Slice,
        directory,
    )
    .expect_err("Native must reject the unadmitted source");
    assert_eq!(failure.kind, crate::runner::stages::FailureKind::Process);
    let evidence = failure.evidence.unwrap();
    let status: Value =
        serde_json::from_slice(&fs::read(evidence.join("status.json")).unwrap()).unwrap();
    let stdout = fs::read(evidence.join("stdout.bin")).unwrap();
    let stderr = fs::read(evidence.join("stderr.bin")).unwrap();
    assert_eq!(status["code"], 251);
    assert_eq!(stdout, b"run found error, return -5, exit...\n");
    assert!(String::from_utf8_lossy(&stderr).contains("Preset was not found"));
    assert!(!directory.join("native-output/plate_1.gcode").exists());
    (status, stdout, stderr)
}

#[tokio::test]
async fn load_settings_3mf_user_inherits_whole_file() {
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };
    let directory = tempfile::Builder::new()
        .prefix("load-settings-3mf-user-inherits-")
        .tempdir()
        .unwrap()
        .keep();
    let project = load_settings_3mf_fixture("compatible-input.3mf");
    let machine = load_settings_3mf_fixture("user-machine.json");
    let process = load_settings_3mf_fixture("process.json");
    let filament = load_settings_3mf_fixture("compatible-filament-0.json");
    let machine_json = fs::read_to_string(&machine).unwrap();
    let process_json = fs::read_to_string(&process).unwrap();
    let filament_json = fs::read_to_string(&filament).unwrap();
    let machine_process = format!("{};{}", machine.display(), process.display());
    let filament_arg = filament.display().to_string();
    let (_, native_stdout, native_stderr) = native_rejection(
        &runner,
        &directory,
        &machine_process,
        &filament_arg,
        &project,
    );
    let api = public_api(
        &fs::read(&project).unwrap(),
        &[(&machine, &machine_json), (&process, &process_json)],
        &[(&filament, &filament_json)],
    )
    .await;
    let error = api.expect_err("Ares must reject the unadmitted user source");
    assert!(error.to_string().contains("Preset was not found"));
    assert_eq!(native_stdout, b"run found error, return -5, exit...\n");
    assert!(String::from_utf8_lossy(&native_stderr).contains("Preset was not found"));
}

#[tokio::test]
async fn load_settings_3mf_filament_inherits_no_chain() {
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };
    let directory = tempfile::Builder::new()
        .prefix("load-settings-3mf-filament-inherits-")
        .tempdir()
        .unwrap()
        .keep();
    let project = load_settings_3mf_fixture("compatible-input.3mf");
    let machine = load_settings_3mf_fixture("machine.json");
    let process = load_settings_3mf_fixture("process.json");
    let filament = load_settings_3mf_fixture("child-filament.json");
    let machine_json = fs::read_to_string(&machine).unwrap();
    let process_json = fs::read_to_string(&process).unwrap();
    let filament_json = fs::read_to_string(&filament).unwrap();
    let machine_process = format!("{};{}", machine.display(), process.display());
    let filament_arg = filament.display().to_string();
    let (_, native_stdout, native_stderr) = native_rejection(
        &runner,
        &directory,
        &machine_process,
        &filament_arg,
        &project,
    );
    let api = public_api(
        &fs::read(&project).unwrap(),
        &[(&machine, &machine_json), (&process, &process_json)],
        &[(&filament, &filament_json)],
    )
    .await;
    let error = api.expect_err("Ares must reject the unadmitted filament source");
    assert!(error.to_string().contains("Preset was not found"));
    assert_eq!(native_stdout, b"run found error, return -5, exit...\n");
    assert!(String::from_utf8_lossy(&native_stderr).contains("Preset was not found"));
}
