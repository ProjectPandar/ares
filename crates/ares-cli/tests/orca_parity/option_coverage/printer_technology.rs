//! FFF-only CLI admission (`OrcaSlicer.cpp:3619,3627-3633`).

use crate::{CaseInputs, OrcaRunner, VendorProfiles, runner};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Cursor, Read, Write},
};

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn export(runner: &OrcaRunner, technology: &str) -> runner::ExportedCase {
    let profiles = VendorProfiles::load(
        &runner::repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        crate::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    machine.insert("printer_technology".into(), json!(technology));
    let mut process = profiles.process(&selection.process).unwrap();
    crate::normalize_process_defaults(&machine, &mut process);
    let mut filaments = selection
        .filaments
        .iter()
        .map(|n| profiles.filament(n).unwrap())
        .collect::<Vec<_>>();
    crate::normalize_filament_defaults(&mut filaments);
    runner
        .export_case(
            &CaseInputs {
                label: &format!("printer-technology/machine-{technology}"),
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &crate::smoke_overrides(),
            &runner::repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap()
}

#[test]
fn printer_technology_fff_control() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let exported = export(&runner, "FFF");
    runner::application::verify_value(&exported.project, "printer_technology", &json!("FFF"))
        .unwrap();
    let sliced = runner.slice_case(&exported).unwrap();
    let outcome = crate::compare_case(&sliced);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
    let root = outcome.artifacts.unwrap();
    let cli = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(root.join("cli.gcode"))
        .arg(root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(root.join("cli.stderr"), &cli.stderr).unwrap();
    assert!(cli.status.success(), "{cli:?}");
    let cli_bytes = fs::read(root.join("cli.gcode")).unwrap();
    crate::golden::compare_ordered_bytes_generator_only(&sliced.reference, &cli_bytes).unwrap();
    assert_eq!(fs::read(root.join("ares.gcode")).unwrap(), cli_bytes);
}

#[test]
fn printer_technology_original_sla_machine_mismatch() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let exported = export(&runner, "SLA");
    let root = tempfile::Builder::new()
        .prefix("technology-sla-machine-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    fs::write(root.join("exported.3mf"), &exported.project).unwrap();
    let error =
        runner::application::verify_value(&exported.project, "printer_technology", &json!("SLA"))
            .unwrap_err();
    assert!(matches!(error.stage, runner::stages::Stage::Application));
    assert!(matches!(
        error.kind,
        runner::stages::FailureKind::ApplicationMismatch
    ));
    assert_eq!(
        runner::application::settings(&exported.project).unwrap()["printer_technology"],
        "FFF"
    );
    fs::write(root.join("application.json"), serde_json::to_vec_pretty(&json!({
        "requested":"SLA","effective":"FFF","error":error.to_string(),
        "original_audit_status":"FAIL","scope":"expected mismatch regression, NOT SLA admission",
        "project_sha256":sha(&exported.project),
        "historical_attribution_root":"/tmp/worker2-option-full-103wd7e0"
    })).unwrap()).unwrap();
}

#[test]
fn printer_technology_parsed_sla_rejection() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let exported = export(&runner, "FFF");
    let mut input = zip::ZipArchive::new(Cursor::new(&exported.project)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..input.len() {
        let mut member = input.by_index(i).unwrap();
        let mut bytes = Vec::new();
        member.read_to_end(&mut bytes).unwrap();
        if member.name() == "Metadata/project_settings.config" {
            let mut config: Value = serde_json::from_slice(&bytes).unwrap();
            config["printer_technology"] = json!("SLA");
            bytes = serde_json::to_vec(&config).unwrap();
        }
        output
            .start_file(
                member.name(),
                zip::write::SimpleFileOptions::DEFAULT.compression_method(member.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    let bytes = output.finish().unwrap().into_inner();
    runner::application::verify_value(&bytes, "printer_technology", &json!("SLA")).unwrap();
    let root = tempfile::Builder::new()
        .prefix("technology-parsed-sla-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let path = root.join("input.3mf");
    fs::write(&path, &bytes).unwrap();
    fs::create_dir(root.join("native")).unwrap();
    let args = vec![
        "--slice".into(),
        "0".into(),
        "--outputdir".into(),
        root.join("native").to_str().unwrap().into(),
        path.to_str().unwrap().into(),
    ];
    fs::write(
        root.join("launch.json"),
        serde_json::to_vec_pretty(&json!({
            "wrapper":runner.bin(),"args":args,"input_sha256":sha(&bytes),
            "requested":"SLA","channel":"imported 3MF project_settings",
            "source":"OrcaSlicer.cpp1854-1859,3619,3627-3633"
        }))
        .unwrap(),
    )
    .unwrap();
    let native =
        runner::stages::run(runner.bin(), args, runner::stages::Stage::Slice, &root).unwrap_err();
    let captured = native.captured.as_ref().unwrap();
    let native_stderr = String::from_utf8(captured.stderr.clone()).unwrap();
    // Utils.hpp:31 defines -7; compare the OS-visible status without remapping Ares.
    let native_code = if cfg!(unix) { 249 } else { -7 };
    let cli = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(root.join("ares.gcode"))
        .arg(&path)
        .output()
        .unwrap();
    fs::write(root.join("ares.stdout"), &cli.stdout).unwrap();
    fs::write(root.join("ares.stderr"), &cli.stderr).unwrap();
    let cli_stderr = String::from_utf8(cli.stderr.clone()).unwrap();
    fs::write(
        root.join("rejection.json"),
        serde_json::to_vec_pretty(&json!({
            "input_sha256":sha(&bytes),"native_code":captured.code,"native_stderr":native_stderr,
            "native_status":captured.status,"native_error":native.to_string(),
            "ares_code":cli.status.code(),"ares_stderr":cli_stderr,
            "required_diagnostic":"invalid printer_technology",
            "sla_slicing_deferred":true,"historical_machine_sla_fail_unchanged":true
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        captured.code,
        Some(native_code),
        "Native technology branch {root:?}"
    );
    assert!(
        native_stderr
            .lines()
            .any(|line| line.trim() == "invalid printer_technology"),
        "{native_stderr}"
    );
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let api = runtime.block_on(crate::runner::application::slice_project_plate_validated(
        &bytes,
        ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    ));
    let wasm = runtime.block_on(ares_wasm::slice_project_plate_processed_bytes(
        bytes.clone(),
        None,
        None,
    ));
    fs::write(
        root.join("api-wasm.json"),
        serde_json::to_vec_pretty(&json!({
            "api_error":api.as_ref().err().map(|e| format!("{e:?}")),
            "wasm_error":wasm.as_ref().err(), "api_admitted":api.is_ok(),
            "wasm_admitted":wasm.is_ok()
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        api.as_ref().err().map(|e| format!("{e:?}")),
        Some("InvalidPrinterTechnology".into()),
        "typed API rejection RED {root:?}"
    );
    assert_eq!(
        wasm.as_ref().err().map(String::as_str),
        Some("invalid printer_technology"),
        "WASM adapter rejection RED {root:?}"
    );
    assert_eq!(
        cli.status.code(),
        captured.code,
        "Ares/Native rejection code RED {root:?}"
    );
    assert!(
        cli_stderr
            .lines()
            .any(|line| line.trim() == "invalid printer_technology"),
        "Ares/Native rejection diagnostic RED {root:?}: {cli_stderr}"
    );
}
