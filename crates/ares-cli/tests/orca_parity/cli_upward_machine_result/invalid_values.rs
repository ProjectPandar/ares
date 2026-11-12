//! OrcaSlicer.cpp:3005 publishes upward machines before validate(true):4272–4278.
//! Covers missing metadata, wrong error channels, extra timing masks and move-file leakage.

use std::fs;

use crate::{
    native_wall_clock::result_comparison_bytes,
    presets::VendorProfiles,
    runner::{self, EarlyExit, OrcaRunner},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

#[test]
fn cli_invalid_values_preserves_upward_machines() {
    invalid_values(false);
}

#[test]
fn cli_invalid_values_dump_moves_preserves_upward_machines() {
    invalid_values(true);
}

fn invalid_values(dump_moves: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("cli-upward-invalid-values-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input = fs::read(
        runner::repo_root().join("tests/parity/cli-upward-invalid-values/firmware-wipe.3mf"),
    )
    .unwrap();
    let input_sha = Sha256::digest(&input)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(
        input_sha,
        "5f7adc3f35fdb9a06ee98964178990cd41906ee1610ef7e4e0114f716daced29"
    );
    let input_path = root.join("firmware-wipe.3mf");
    fs::write(&input_path, &input).unwrap();
    fs::write(root.join("input.sha256"), &input_sha).unwrap();
    let native = runner
        .early_cli_reference(&input_path, EarlyExit::InvalidValues)
        .unwrap();
    fs::write(root.join("native.stdout"), &native.stdout).unwrap();
    fs::write(root.join("native.stderr"), &native.stderr).unwrap();
    fs::write(root.join("native.result.json"), &native.result_json).unwrap();
    fs::copy(
        native.directory.join("receipt.json"),
        root.join("native.receipt.json"),
    )
    .unwrap();
    let native_json: Value = serde_json::from_slice(&native.result_json).unwrap();
    assert_eq!(native_json["return_code"], -18);
    assert_eq!(
        native_json["upward_compatible_machine"],
        json!(["Bambu Lab H2D Pro 0.4 nozzle"])
    );
    // The coordinator permits only these two upstream uninitialized timing members.
    let timing_exception = ["prepare_time", "export_time"];
    let mut expected = native_json.clone();
    for field in timing_exception {
        expected[field] = json!(0);
    }
    let comparable = result_comparison_bytes(&native.result_json);
    assert_eq!(
        serde_json::from_slice::<Value>(&comparable).unwrap(),
        expected
    );
    fs::write(root.join("native.comparison.json"), &comparable).unwrap();
    let output = root.join("ares");
    fs::create_dir(&output).unwrap();
    let gcode = output.join("plate_1.gcode");
    let moves = output.join("moves.csv");
    let profiles = VendorProfiles::load(
        &runner::repo_root().join("OrcaSlicer/resources/profiles"),
        "BBL",
    )
    .unwrap();
    let stock = profiles.machine("Bambu Lab H2D 0.4 nozzle").unwrap();
    assert_eq!(
        stock["upward_compatible_machine"],
        native_json["upward_compatible_machine"]
    );
    // Invoke the default plain-project entry without the adapter's implicit catalog refresh.
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_ares"));
    command.args(["slice", "-o"]).arg(&gcode);
    if dump_moves {
        command.arg("--dump-moves").arg(&moves);
    }
    let cli = command.arg(&input_path).output().unwrap();
    fs::write(root.join("ares.stdout"), &cli.stdout).unwrap();
    fs::write(root.join("ares.stderr"), &cli.stderr).unwrap();
    fs::write(root.join("ares.exit"), format!("{:?}\n", cli.status.code())).unwrap();
    let ares = fs::read(output.join("result.json")).unwrap();
    let ares_json: Value = serde_json::from_slice(&ares).unwrap();
    fs::write(
        root.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "input_sha256": input_sha,
            "dump_moves": dump_moves,
            "timing_exception": timing_exception,
            "native_raw": native_json,
            "ares_raw": ares_json,
            "whole_result_equal_except_authorized_timings": ares == comparable,
            "stdout_equal": cli.stdout == native.stdout,
            "stderr_equal": cli.stderr == native.stderr,
            "gcode_absent": !gcode.exists(),
            "moves_absent": !moves.exists(),
        }))
        .unwrap(),
    )
    .unwrap();
    eprintln!("upward invalid-values witness: artifacts {root:?}");
    assert_eq!(cli.status.code(), Some(238), "{root:?}");
    assert_eq!(cli.stdout, native.stdout, "{root:?}");
    assert_eq!(cli.stderr, native.stderr, "{root:?}");
    assert!(!gcode.exists() && !moves.exists(), "{root:?}");
    for field in timing_exception {
        assert_eq!(ares_json[field], 0, "deterministic Ares {field}: {root:?}");
    }
    assert_eq!(
        ares, comparable,
        "all result fields/bytes, except only prepare_time/export_time: {root:?}"
    );
}
