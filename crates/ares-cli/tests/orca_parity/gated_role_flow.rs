//! `GCode.cpp:6485-6507`: the complete set_other_flow_ratios block.

use crate::{OrcaRunner, golden, runner::stages};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs};

macro_rules! cases {
    ($($test:ident => $label:literal),* $(,)?) => {
        $(#[test] fn $test() { check($label); })*
    };
}

cases! {
    orca_parity_gated_flow_box_unit => "box-unit",
    orca_parity_gated_flow_outer075 => "outer075",
    orca_parity_gated_flow_inner065 => "inner065",
    orca_parity_gated_flow_sparse070_guard => "sparse070",
    orca_parity_gated_flow_solid060 => "solid060",
    orca_parity_gated_flow_gate_off_all => "gate-off-all",
    orca_parity_gated_flow_first_unit => "first-unit",
    orca_parity_gated_flow_first055_on => "first055-on",
    orca_parity_gated_flow_first055_off => "first055-off",
    orca_parity_gated_flow_gap_unit => "gap-unit",
    orca_parity_gated_flow_gap070 => "gap070",
    orca_parity_gated_flow_support_unit => "support-unit",
    orca_parity_gated_flow_support070 => "support070",
    orca_parity_gated_flow_interface065 => "interface065",
    orca_parity_gated_flow_overhang070 => "overhang070",
}

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn assert_native_active(native: &str, case: &Value) -> Value {
    let mut layers = Vec::new();
    let mut roles = BTreeMap::<String, usize>::new();
    let mut feature = "";
    for line in native.lines() {
        if line == ";LAYER_CHANGE" {
            layers.push(0usize);
        }
        if let Some(value) = line.strip_prefix(";TYPE:") {
            feature = value;
        }
        if !line.starts_with("G1 ") || layers.is_empty() {
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
            *layers.last_mut().unwrap() += 1;
            *roles.entry(feature.to_owned()).or_default() += 1;
        }
    }
    assert_eq!(
        layers.len() as u64,
        case["positive_layers"].as_u64().unwrap()
    );
    assert!(layers.iter().all(|count| *count > 0));
    assert_eq!(
        serde_json::to_value(&roles).unwrap(),
        case["native_activity"]
    );
    for (key, value) in case["changes"].as_object().unwrap() {
        assert!(native.contains(&format!("; {key} = {}\n", value.as_str().unwrap())));
    }
    json!({"positive_moves_per_layer":layers,"positive_role_moves":roles})
}

fn check(label: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let fixture = crate::runner::repo_root().join("tests/parity/gated-role-flow");
    let manifest: Value =
        serde_json::from_slice(&fs::read(fixture.join("manifest.json")).unwrap()).unwrap();
    let case = &manifest["cases"][label];
    let bytes = fs::read(fixture.join(format!("{label}.3mf"))).unwrap();
    assert_eq!(sha(&bytes), case["archive_sha256"]);
    let settings = crate::runner::application::settings(&bytes).unwrap();
    for (key, value) in case["changes"].as_object().unwrap() {
        assert_eq!(settings[key], *value, "input setting {key}");
    }
    let root = tempfile::Builder::new()
        .prefix(&format!("gated-flow-{label}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    fs::write(root.join("input.3mf"), &bytes).unwrap();
    fs::create_dir(root.join("native")).unwrap();
    let mut args = vec![
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
            "case":label,"classification":case["classification"],
            "wrapper":runner.bin(),"native_binary":native_binary,
            "native_binary_sha256":sha(&fs::read(&native_binary).unwrap()),
            "input_sha256":sha(&bytes),"args":args,"source":manifest["source"]
        }))
        .unwrap(),
    )
    .unwrap();
    stages::run(runner.bin(), args.clone(), stages::Stage::Slice, &root).unwrap();
    let reference = fs::read(root.join("native/plate_1.gcode")).unwrap();
    let activity = assert_native_active(std::str::from_utf8(&reference).unwrap(), case);
    fs::write(
        root.join("native-activity.json"),
        serde_json::to_vec_pretty(&activity).unwrap(),
    )
    .unwrap();
    fs::create_dir(root.join("native-repeat")).unwrap();
    args[3] = root.join("native-repeat").to_str().unwrap().into();
    stages::run(runner.bin(), args, stages::Stage::Slice, &root).unwrap();
    let repeat = fs::read(root.join("native-repeat/plate_1.gcode")).unwrap();
    assert_native_active(std::str::from_utf8(&repeat).unwrap(), case);
    let first =
        golden::normalize_one_generator_line(&reference, golden::GeneratorKind::Orca).unwrap();
    let second =
        golden::normalize_one_generator_line(&repeat, golden::GeneratorKind::Orca).unwrap();
    let difference = golden::first_difference(&first, &second).map(|value| value.to_string());
    fs::write(
        root.join("native-determinism.json"),
        serde_json::to_vec_pretty(&json!({
            "raw_equal":reference==repeat,
            "complete_native_repeat_equal_generator_identity_only":first==second,
            "difference":difference
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(difference.is_none(), "Native repeat: {difference:?}");
    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            &bytes,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ));
    let output = root.join("ares.cli.gcode");
    let cli = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(&output)
        .arg(root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(root.join("ares.cli.stdout"), &cli.stdout).unwrap();
    fs::write(root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    fs::write(
        root.join("admission.json"),
        serde_json::to_vec_pretty(&json!({
            "api_error":api.as_ref().err().map(|error|format!("{error:?}")),
            "cli_code":cli.status.code(),"cli_output_exists":output.exists()
        }))
        .unwrap(),
    )
    .unwrap();
    let api =
        api.unwrap_or_else(|error| panic!("Native-active input must slice: {error:?}; {root:?}"));
    fs::write(root.join("ares.api.gcode"), &api).unwrap();
    assert!(cli.status.success(), "{root:?}: {cli:?}");
    let cli_bytes = fs::read(&output).unwrap();
    let api_comparison = golden::compare_ordered_bytes_generator_only(&reference, &api);
    let cli_comparison = golden::compare_ordered_bytes_generator_only(&reference, &cli_bytes);
    fs::write(
        root.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "case":label,"api_comparison":api_comparison,"cli_comparison":cli_comparison,
            "raw_api_cli_equal":api==cli_bytes,"cli_code":cli.status.code()
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(api, cli_bytes, "raw API/CLI {root:?}");
    api_comparison.unwrap_or_else(|error| panic!("{root:?}: {error}"));
    cli_comparison.unwrap_or_else(|error| panic!("{root:?}: {error}"));
}
