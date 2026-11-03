#![cfg(target_os = "linux")]

use std::fs;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{
    native_wall_clock::result_comparison_bytes,
    runner::{EarlyExit, OrcaRunner},
};

#[path = "stock_early_cli/cases.rs"]
mod cases;

fn compare(case: &cases::Case) {
    let Some(_) = crate::native_environment::binary("ARES_ORCA_BIN") else {
        return;
    };
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let root = crate::runner::repo_root();
    let profiles = crate::presets::VendorProfiles::load(
        &root.join("OrcaSlicer/resources/profiles"),
        case.vendor,
    )
    .unwrap();
    let selected = crate::select_printer(&profiles, case.vendor, case.printer).unwrap();
    let machine = profiles.machine(&selected.printer).unwrap();
    let process = profiles.process(&selected.process).unwrap();
    let filaments = selected
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    let label = format!("stock/{}/{}", case.vendor, case.printer);
    let exported = runner
        .export_case(
            &crate::runner::CaseInputs {
                label: &label,
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &serde_json::Map::new(),
            &root.join("tests/parity/cube10.stl"),
        )
        .unwrap();
    let mut builder = tempfile::Builder::new();
    builder.prefix("stock-early-cli-");
    let artifacts = if std::env::var_os("ARES_PARITY_ARTIFACT_ROOT").is_some() {
        builder.tempdir_in(crate::artifacts::root_from_env().unwrap())
    } else {
        builder.tempdir()
    }
    .unwrap()
    .keep();
    let input = artifacts.join("input.3mf");
    fs::write(&input, &exported.project).unwrap();
    assert_eq!(fs::read(&input).unwrap(), exported.project);
    let sha = Sha256::digest(&exported.project)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let rejection = match case.upstream_code {
        -50 => EarlyExit::EmptyPlate,
        -19 => EarlyExit::Postprocess,
        _ => unreachable!(),
    };
    let native = runner.early_cli_reference(&input, rejection).unwrap();
    let hit = runner.early_cli_reference(&input, rejection).unwrap();
    assert_eq!(native.directory, hit.directory);
    assert_eq!(native.stdout, hit.stdout);
    assert_eq!(native.stderr, hit.stderr);
    assert_eq!(native.result_json, hit.result_json);
    assert_eq!(native.gcode, hit.gcode);
    assert!(native.gcode.is_none());
    fs::write(artifacts.join("native.stdout"), &native.stdout).unwrap();
    fs::write(artifacts.join("native.stderr"), &native.stderr).unwrap();
    fs::write(artifacts.join("native.result.json"), &native.result_json).unwrap();
    let receipt_bytes = fs::read(native.directory.join("receipt.json")).unwrap();
    fs::write(artifacts.join("native.receipt.json"), &receipt_bytes).unwrap();
    let receipt: Value = serde_json::from_slice(&receipt_bytes).unwrap();
    let native_exit = i32::try_from(receipt["exit"].as_i64().unwrap()).unwrap();
    let gcode = artifacts.join("plate_1.gcode");
    let output = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(&gcode)
        .arg(&input)
        .output()
        .unwrap();
    fs::write(artifacts.join("ares.stdout"), &output.stdout).unwrap();
    fs::write(artifacts.join("ares.stderr"), &output.stderr).unwrap();
    fs::write(
        artifacts.join("ares.exit"),
        format!("{:?}\n", output.status.code()),
    )
    .unwrap();
    let result = fs::read(artifacts.join("result.json")).unwrap();
    fs::write(artifacts.join("comparison.json"), serde_json::to_vec_pretty(&json!({
        "vendor": case.vendor, "printer": case.printer, "process": selected.process, "filaments": selected.filaments,
        "provenance": "RUNTIME_UNCHANGED_STOCK_SELECTOR_NATIVE_EXPORT",
        "input_sha256": sha, "input": input, "upstream_code": rejection.code(),
        "native_exit": native_exit, "ares_exit": output.status.code(),
        "stdout_equal": output.stdout == native.stdout, "stderr_equal": output.stderr == native.stderr,
        "result_equal_except_two_wall_clock_fields": result_comparison_bytes(&result) == result_comparison_bytes(&native.result_json),
        "native_cache": native.directory,
    })).unwrap()).unwrap();
    assert_eq!(
        native_exit,
        rejection.code().rem_euclid(256),
        "Native admission: {artifacts:?}"
    );
    assert_eq!(
        output.status.code(),
        Some(native_exit),
        "CLI exit: {artifacts:?}"
    );
    assert_eq!(output.stdout, native.stdout, "CLI stdout: {artifacts:?}");
    assert_eq!(output.stderr, native.stderr, "CLI stderr: {artifacts:?}");
    assert_eq!(
        result_comparison_bytes(&result),
        result_comparison_bytes(&native.result_json),
        "CLI result bytes: {artifacts:?}"
    );
    assert!(!gcode.exists(), "no final G-code: {artifacts:?}");
}

macro_rules! case {
    ($test:ident, $index:literal) => {
        #[test]
        fn $test() {
            compare(&cases::CASES[$index]);
        }
    };
}

case!(stock_kobra3_02_eligibility_channels_match_native, 0);
case!(stock_kobra3_04_eligibility_channels_match_native, 1);
case!(stock_kobra3_06_eligibility_channels_match_native, 2);
case!(stock_kobra3_08_eligibility_channels_match_native, 3);
case!(stock_mk3s_025_postprocess_channels_match_native, 4);
case!(stock_mk3s_04_postprocess_channels_match_native, 5);
case!(stock_mk3s_06_postprocess_channels_match_native, 6);
case!(stock_mk3s_08_postprocess_channels_match_native, 7);
