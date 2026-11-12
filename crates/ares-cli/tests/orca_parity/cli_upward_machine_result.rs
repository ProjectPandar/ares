//! OrcaSlicer.cpp:1769,2517-2540,2656,428-432; existing CustomGCode -100.
//! Risks: wrong error family, wrong machine source, omitted/changed arrays,
//! incomplete channel comparison and a frozen Native result masking admission.
#![cfg(target_os = "linux")]

#[path = "cli_upward_machine_result/invalid_values.rs"]
mod invalid_values;

use std::fs;

use crate::{
    native_wall_clock::result_comparison_bytes,
    presets::VendorProfiles,
    runner::{self, CaseInputs, OrcaRunner},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

#[test]
fn cli_custom_gcode_failure_preserves_nonempty_upward_machine_header() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("cli-upward-machines-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let profiles = VendorProfiles::load(
        &runner::repo_root().join("OrcaSlicer/resources/profiles"),
        "BBL",
    )
    .unwrap();
    let selection = crate::select_printer(&profiles, "BBL", "Bambu Lab H2D 0.4 nozzle").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    let upward = machine["upward_compatible_machine"].clone();
    assert!(!upward.as_array().unwrap().is_empty());
    fs::write(
        root.join("stock-machine-full.json"),
        serde_json::to_vec_pretty(&machine).unwrap(),
    )
    .unwrap();
    // Fixed independent -100 witness; do not import the grouping layer.
    machine.insert(
        "machine_start_gcode".into(),
        json!("{ares_upward_result_missing_variable}\n"),
    );
    let mut process = profiles.process(&selection.process).unwrap();
    crate::normalize_process_defaults(&machine, &mut process);
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    crate::normalize_filament_defaults(&mut filaments);
    let input = runner
        .export_case(
            &CaseInputs {
                label: "cli/upward-machine-header/custom-gcode-source",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &crate::smoke_overrides(),
            &runner::repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap()
        .project;
    let input_path = root.join("input.3mf");
    fs::write(&input_path, &input).unwrap();
    let input_sha = Sha256::digest(&input)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    fs::write(root.join("input.sha256"), &input_sha).unwrap();
    let native = runner.custom_gcode_reference(&input_path).unwrap();
    fs::write(root.join("native.stdout"), &native.stdout).unwrap();
    fs::write(root.join("native.stderr"), &native.stderr).unwrap();
    fs::write(root.join("native.result.json"), &native.result_json).unwrap();
    fs::copy(
        native.directory.join("receipt.json"),
        root.join("native.receipt.json"),
    )
    .unwrap();
    let native_json: Value = serde_json::from_slice(&native.result_json).unwrap();
    assert_eq!(native_json["return_code"], -100);
    assert_eq!(native_json["upward_compatible_machine"], upward);
    assert!(
        !native_json
            .as_object()
            .unwrap()
            .contains_key("downward_compatible_machine")
    );
    let output_dir = root.join("ares");
    fs::create_dir(&output_dir).unwrap();
    let gcode = output_dir.join("plate_1.gcode");
    let cli = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(&gcode)
        .arg(&input_path)
        .output()
        .unwrap();
    fs::write(root.join("ares.stdout"), &cli.stdout).unwrap();
    fs::write(root.join("ares.stderr"), &cli.stderr).unwrap();
    fs::write(root.join("ares.exit"), format!("{:?}\n", cli.status.code())).unwrap();
    eprintln!("upward custom-gcode witness: artifacts {root:?}");
    assert_eq!(cli.status.code(), Some(156), "{root:?}");
    assert_eq!(cli.stdout, native.stdout, "{root:?}");
    assert_eq!(cli.stderr, native.stderr, "{root:?}");
    assert!(!gcode.exists(), "no final G-code on -100: {root:?}");
    let ares = fs::read(output_dir.join("result.json")).unwrap();
    assert_eq!(
        result_comparison_bytes(&ares),
        result_comparison_bytes(&native.result_json),
        "complete result JSON, including upward header: {root:?}"
    );
}
