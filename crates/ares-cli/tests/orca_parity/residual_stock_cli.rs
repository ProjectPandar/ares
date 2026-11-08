#![cfg(target_os = "linux")]

use std::fs;

use sha2::{Digest, Sha256};

use crate::{golden, native_wall_clock::result_comparison_bytes, runner::OrcaRunner};

fn compare(case: &str, historical_sha: &str) {
    let Some(_) = crate::native_environment::binary("ARES_ORCA_BIN") else {
        return;
    };
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let (vendor, printer) = match case {
        "flsun-qqs" => ("FLSun", "FLSun QQ-S Pro 0.4 nozzle"),
        "ginger-1_2" => ("Ginger Additive", "Ginger G1 1.2 nozzle"),
        "ginger-3_0" => ("Ginger Additive", "Ginger G1 3.0 nozzle"),
        "ginger-5_0" => ("Ginger Additive", "Ginger G1 5.0 nozzle"),
        "ginger-8_0" => ("Ginger Additive", "Ginger G1 8.0 nozzle"),
        _ => unreachable!(),
    };
    let root = crate::runner::repo_root();
    let profiles =
        crate::presets::VendorProfiles::load(&root.join("OrcaSlicer/resources/profiles"), vendor)
            .unwrap();
    let selected = crate::select_printer(&profiles, vendor, printer).unwrap();
    let machine = profiles.machine(&selected.printer).unwrap();
    let process = profiles.process(&selected.process).unwrap();
    let filaments = selected
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    let label = format!("stock/{vendor}/{printer}");
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
    let bytes = &exported.project;
    let sha = Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let mut builder = tempfile::Builder::new();
    builder.prefix("residual-stock-cli-");
    let artifacts = if std::env::var_os("ARES_PARITY_ARTIFACT_ROOT").is_some() {
        builder.tempdir_in(crate::artifacts::root_from_env().unwrap())
    } else {
        builder.tempdir()
    }
    .unwrap()
    .keep();
    let input = artifacts.join(format!("{case}.3mf"));
    fs::write(&input, bytes).unwrap();
    assert_eq!(fs::read(&input).unwrap(), *bytes);
    let native = runner.custom_gcode_success_reference(&input).unwrap();
    let hit = runner.custom_gcode_success_reference(&input).unwrap();
    assert_eq!(native.directory, hit.directory);
    assert_eq!(native.stdout, hit.stdout);
    assert_eq!(native.stderr, hit.stderr);
    assert_eq!(native.result_json, hit.result_json);
    fs::write(artifacts.join("native.stdout"), &native.stdout).unwrap();
    fs::write(artifacts.join("native.stderr"), &native.stderr).unwrap();
    fs::write(artifacts.join("native.result.json"), &native.result_json).unwrap();
    fs::copy(
        native.directory.join("receipt.json"),
        artifacts.join("native.receipt.json"),
    )
    .unwrap();
    let retained = artifacts.join("plate_1.gcode");
    let output = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(&retained)
        .arg(&input)
        .output()
        .unwrap();
    fs::write(artifacts.join("ares.stdout"), &output.stdout).unwrap();
    fs::write(artifacts.join("ares.stderr"), &output.stderr).unwrap();
    let actual_result = fs::read(artifacts.join("result.json")).unwrap();
    fs::write(artifacts.join("comparison.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "case": case, "input_sha256": sha, "input": input,
        "provenance": "RUNTIME_STOCK_SELECTOR_NATIVE_EXPORT",
        "historical_stock_input_sha256": historical_sha,
        "ares_exit": output.status.code(), "native_code": 0,
        "stdout_equal": output.stdout == native.stdout,
        "stderr_equal": output.stderr == native.stderr,
        "result_equal_except_two_wall_clock_fields": result_comparison_bytes(&actual_result) == result_comparison_bytes(&native.result_json),
        "native_cache": native.directory,
    })).unwrap()).unwrap();
    assert_eq!(output.status.code(), Some(0), "artifacts {artifacts:?}");
    assert_eq!(output.stdout, native.stdout, "artifacts {artifacts:?}");
    assert_eq!(output.stderr, native.stderr, "artifacts {artifacts:?}");
    assert_eq!(
        result_comparison_bytes(&actual_result),
        result_comparison_bytes(&native.result_json),
        "artifacts {artifacts:?}"
    );
    assert!(retained.exists(), "artifacts {artifacts:?}");
    let expected_gcode = golden::normalize_one_generator_line(
        native.gcode.as_ref().unwrap(),
        golden::GeneratorKind::Orca,
    )
    .unwrap();
    let actual_gcode = golden::normalize_one_generator_line(
        &fs::read(&retained).unwrap(),
        golden::GeneratorKind::Ares,
    )
    .unwrap();
    assert_eq!(actual_gcode, expected_gcode, "artifacts {artifacts:?}");
}

#[test]
fn current_stock_flsun_custom_template_channels_match_native() {
    compare(
        "flsun-qqs",
        "61badf4744af6770e333a181d41da1c05027f5cdcf9db9f501c6c3d993bc1784",
    );
}

#[test]
fn current_stock_ginger_1_2_custom_template_channels_match_native() {
    compare(
        "ginger-1_2",
        "872dcd0186b10f88dfbd1dd7ba967a1654b3b0ff5a58100fc8a16730c6f5e068",
    );
}

#[test]
fn current_stock_ginger_3_0_custom_template_channels_match_native() {
    compare(
        "ginger-3_0",
        "41e7eb9342fc250ecf0bb070603a042be322c5229e646395a29a3c0236ffaa68",
    );
}

#[test]
fn current_stock_ginger_5_0_custom_template_channels_match_native() {
    compare(
        "ginger-5_0",
        "8c6b5a1d4c3143b369ecd9f2952834f8941f093353a1eec3aa720d05bf262761",
    );
}

#[test]
fn current_stock_ginger_8_0_custom_template_channels_match_native() {
    compare(
        "ginger-8_0",
        "c8bc4b49d47416ca324776fc9e27f8165838b1d776416be1adea30d592d25bf5",
    );
}
