//! GCode.cpp:5685-5718,7678-7704; GCodeWriter.cpp:1026-1046,626-648.

use crate::{OrcaRunner, golden, runner::stages};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, process::Command};

#[test]
fn orca_parity_firmware_zero_layer_lift_target() {
    check("fw-zero-hop");
}

#[test]
fn orca_parity_firmware_zero_layer_lift_positive_control() {
    check("fw-positive-hop");
}

#[test]
fn orca_parity_firmware_zero_layer_lift_no_hop_control() {
    check("fw-zero-no-hop");
}

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn assert_native_active(native: &str, normal_lifts: usize) {
    let mut model_layers = Vec::new();
    for line in native.lines() {
        if line == ";LAYER_CHANGE" {
            model_layers.push(false);
        }
        if !line.starts_with("G1 ") || model_layers.is_empty() {
            continue;
        }
        let e = line
            .split_whitespace()
            .find_map(|word| word.strip_prefix('E'));
        if let Some(e) = e
            && e.parse::<f64>().unwrap() > 0.0
            && line
                .split_whitespace()
                .any(|word| word.starts_with('X') || word.starts_with('Y'))
        {
            *model_layers.last_mut().unwrap() = true;
        }
    }
    assert_eq!(
        model_layers,
        vec![true; 75],
        "Native model must print all75 layers"
    );
    assert_eq!(native.matches("normal lift Z").count(), normal_lifts);
    assert_eq!(native.matches("G10 ; retract\n").count(), 76);
    assert!(native.contains("G11 ; unretract\n"));
    assert!(!native.contains(";TYPE:Brim\n"));
    assert!(!native.contains(";TYPE:Skirt\n"));
}

fn check(label: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let fixture = crate::runner::repo_root().join("tests/parity/firmware-zero-layer-lift");
    let manifest: Value =
        serde_json::from_slice(&fs::read(fixture.join("manifest.json")).unwrap()).unwrap();
    let bytes = fs::read(fixture.join(format!("{label}.3mf"))).unwrap();
    assert_eq!(sha(&bytes), manifest["cases"][label]["sha256"]);
    let length = if label == "fw-positive-hop" { "4" } else { "0" };
    let hop = if label == "fw-zero-no-hop" {
        "0"
    } else {
        "0.4"
    };
    let settings = crate::runner::application::settings(&bytes).unwrap();
    for (key, value) in [
        ("use_firmware_retraction", json!("1")),
        ("retraction_length", json!([length])),
        ("z_hop", json!([hop])),
        ("retract_when_changing_layer", json!(["1"])),
        ("retraction_minimum_travel", json!(["100"])),
        ("z_hop_types", json!(["Normal Lift"])),
        ("wipe", json!(["0"])),
        ("gcode_flavor", json!("marlin2")),
        ("print_sequence", json!("by layer")),
        ("skirt_loops", json!("0")),
        ("brim_width", json!("0")),
        ("enable_support", json!("0")),
        ("raft_layers", json!("0")),
        ("enable_prime_tower", json!("0")),
    ] {
        assert_eq!(settings[key], value, "{key}");
    }
    let root = tempfile::Builder::new()
        .prefix(&format!("firmware-zero-layer-lift-{label}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    fs::write(root.join("input.3mf"), &bytes).unwrap();
    fs::create_dir(root.join("native")).unwrap();
    let args = vec![
        "--slice".into(),
        "0".into(),
        "--outputdir".into(),
        root.join("native").to_str().unwrap().into(),
        root.join("input.3mf").to_str().unwrap().into(),
    ];
    let native_binary = std::env::var_os("ARES_ORCA_NATIVE_BIN").unwrap();
    fs::write(
        root.join("launch.json"),
        serde_json::to_vec_pretty(&json!({
            "wrapper":runner.bin(), "native_binary":native_binary,
            "native_binary_sha256":sha(&fs::read(&native_binary).unwrap()),
            "input_sha256":sha(&bytes), "args":args, "source":manifest["source"]
        }))
        .unwrap(),
    )
    .unwrap();
    stages::run(runner.bin(), args, stages::Stage::Slice, &root).unwrap();
    let reference = fs::read(root.join("native/plate_1.gcode")).unwrap();
    let native = std::str::from_utf8(&reference).unwrap();
    assert_native_active(native, if hop == "0" { 0 } else { 75 });
    for key in [
        "use_firmware_retraction = 1".to_owned(),
        "retract_when_changing_layer = 1".to_owned(),
        "retraction_minimum_travel = 100".to_owned(),
        format!("retraction_length = {length}"),
        format!("z_hop = {hop}"),
    ] {
        assert!(native.contains(&format!("; {key}\n")));
    }
    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project_plate_validated(
            &bytes,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(root.join("ares.api.gcode"), &api).unwrap();
    let output = root.join("ares.cli.gcode");
    let cli = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(&output)
        .arg(root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(root.join("ares.cli.stdout"), &cli.stdout).unwrap();
    fs::write(root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    assert!(cli.status.success(), "{root:?}: {cli:?}");
    let cli_bytes = fs::read(&output).unwrap();
    let api_comparison = golden::compare_ordered_bytes_generator_only(&reference, &api);
    let cli_comparison = golden::compare_ordered_bytes_generator_only(&reference, &cli_bytes);
    fs::write(
        root.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "case":label, "api_comparison":api_comparison, "cli_comparison":cli_comparison,
            "raw_api_cli_equal":api==cli_bytes, "cli_code":cli.status.code()
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(api, cli_bytes, "raw API/CLI {root:?}");
    api_comparison.unwrap_or_else(|error| panic!("{root:?}: {error}"));
    cli_comparison.unwrap_or_else(|error| panic!("{root:?}: {error}"));
}
