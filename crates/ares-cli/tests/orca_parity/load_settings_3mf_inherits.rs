//! Runtime Native source-admission witnesses through the public core API.
#![cfg(target_os = "linux")]

use std::{fs, path::Path};

use ares_core::{
    ConfigOverrides, GenerationMetadata, LoadSettingsFile, LoadSettingsSliceRequest, Placement,
    SlicerContext,
};
use serde_json::Value;

use crate::load_settings_3mf::load_settings_3mf_fixture;

#[test]
fn load_settings_source_manifest_admits_only_manifest_file_identity() {
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("source-manifest-")
        .tempdir()
        .unwrap()
        .keep();
    let vendor = root.join("IdentityVendor");
    fs::create_dir_all(vendor.join("machine")).unwrap();
    let base: Value =
        serde_json::from_slice(&fs::read(load_settings_3mf_fixture("machine.json")).unwrap())
            .unwrap();
    let canonical_name = base["name"].clone();
    let model = serde_json::json!({"type":"machine_model", "name":base["printer_model"], "model_id":"IdentityModel", "nozzle_diameter":"0.4", "machine_tech":"FFF", "family":"IdentityVendor"});
    fs::write(vendor.join("machine/model.json"), model.to_string()).unwrap();
    let mut parent = base;
    parent["name"] = "Identity base".into();
    parent["instantiation"] = "false".into();
    parent.as_object_mut().unwrap().remove("inherits");
    fs::write(vendor.join("machine/base.json"), parent.to_string()).unwrap();
    let child = serde_json::json!({"type":"machine", "from":"system", "name":canonical_name, "inherits":"Identity base", "instantiation":"true", "printer_settings_id":canonical_name});
    let source = vendor.join("machine/child.json");
    fs::write(&source, child.to_string()).unwrap();
    let index = serde_json::json!({"name":"IdentityVendor", "version":"02.05.00.00", "force_update":"0", "machine_model_list":[{"name":model["name"],"sub_path":"machine/model.json"}], "machine_list":[{"name":"Identity base", "sub_path":"machine/base.json"},{"name":canonical_name, "sub_path":"machine/child.json"}], "process_list":[], "filament_list":[]});
    fs::write(root.join("IdentityVendor.json"), index.to_string()).unwrap();
    let process = load_settings_3mf_fixture("process.json");
    let filament = load_settings_3mf_fixture("compatible-filament-0.json");
    let project = load_settings_3mf_fixture("compatible-input.3mf");
    let native = runner
        .slice_stl_with_settings([&source, &process, &filament], &project)
        .unwrap();
    fs::write(root.join("native.gcode"), &native).unwrap();
    let output = root.join("ares.gcode");
    let cli = std::process::Command::new(env!("CARGO_BIN_EXE_ares"))
        .args([
            "slice",
            "--load-settings",
            &format!("{};{}", source.display(), process.display()),
            "--load-filaments",
            filament.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
            project.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    fs::write(root.join("ares.stderr"), &cli.stderr).unwrap();
    assert!(
        cli.status.success(),
        "{}; {}",
        root.display(),
        String::from_utf8_lossy(&cli.stderr)
    );
    assert!(output.is_file());
    let alias = vendor.join("machine/hard-link.json");
    fs::hard_link(&source, &alias).unwrap();
    let alias_native = runner
        .slice_stl_with_settings([&alias, &process, &filament], &project)
        .unwrap();
    fs::write(root.join("alias-native.gcode"), alias_native).unwrap();
    let alias_output = root.join("alias-ares.gcode");
    let alias_cli = std::process::Command::new(env!("CARGO_BIN_EXE_ares"))
        .args([
            "slice",
            "--load-settings",
            &format!("{};{}", alias.display(), process.display()),
            "--load-filaments",
            filament.to_str().unwrap(),
            "-o",
            alias_output.to_str().unwrap(),
            project.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    fs::write(root.join("alias-ares.stderr"), &alias_cli.stderr).unwrap();
    assert!(
        alias_cli.status.success(),
        "{}",
        String::from_utf8_lossy(&alias_cli.stderr)
    );
    assert!(alias_output.is_file());
    // Same bytes and name outside the manifest must not acquire membership.
    let external = root.join("copy.json");
    fs::copy(&source, &external).unwrap();
    let rejected = root.join("rejected");
    fs::create_dir(&rejected).unwrap();
    native_rejection(
        &runner,
        &rejected,
        &format!("{};{}", external.display(), process.display()),
        filament.to_str().unwrap(),
        &project,
    );
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_ares"))
        .args([
            "slice",
            "--load-settings",
            &format!("{};{}", external.display(), process.display()),
            "--load-filaments",
            filament.to_str().unwrap(),
            "-o",
            rejected.join("ares.gcode").to_str().unwrap(),
            project.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    fs::write(rejected.join("ares.stdout"), &result.stdout).unwrap();
    fs::write(rejected.join("ares.stderr"), &result.stderr).unwrap();
    assert_eq!(result.status.code(), Some(251));
    assert_eq!(result.stdout, b"run found error, return -5, exit...\n");
    assert!(!rejected.join("ares.gcode").exists());
}

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
            source_identity: None,
        })
        .collect::<Vec<_>>();
    let filaments = filaments
        .iter()
        .map(|(path, json)| LoadSettingsFile {
            label: path.to_str().unwrap(),
            json,
            source_identity: None,
        })
        .collect::<Vec<_>>();
    let mut sliced_plate = ares_core::SlicedPlate::default();
    ares_core::slice_with_load_settings(
        project,
        LoadSettingsSliceRequest {
            filename: "compatible-input.3mf",
            catalog_json: None,
            loaded_sources: None,
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
    fs::create_dir_all(directory.join("native-output")).unwrap();
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
    let result: Value =
        serde_json::from_slice(&fs::read(directory.join("native-output/result.json")).unwrap())
            .unwrap();
    assert_eq!(result["return_code"], -5);
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
    assert!(matches!(error, ares_core::SliceError::PresetAdmission(_)));
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
    assert!(matches!(error, ares_core::SliceError::PresetAdmission(_)));
    assert_eq!(native_stdout, b"run found error, return -5, exit...\n");
    assert!(String::from_utf8_lossy(&native_stderr).contains("Preset was not found"));
}
