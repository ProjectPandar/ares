//! Fill.cpp:1619-1640,1705-1731; FillBase.cpp:73-75;
//! FillRectilinear.hpp:193-224; FillRectilinear.cpp:2853-2875,3387-3399.

use crate::{OrcaRunner, golden, runner::stages};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, process::Command};

#[test]
fn orca_parity_object_zag_ironing_zigzag_target() {
    check("zigzag");
}

#[test]
fn orca_parity_object_zag_ironing_crosszag_target() {
    check("crosszag");
}

#[test]
fn orca_parity_object_zag_ironing_lockedzag_target() {
    check("lockedzag");
}

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn assert_native_active(native: &str) {
    let mut model_layers = Vec::new();
    let mut ironing_layers = Vec::new();
    let mut feature = "";
    for line in native.lines() {
        if line == ";LAYER_CHANGE" {
            model_layers.push(false);
            ironing_layers.push(false);
        }
        if let Some(value) = line.strip_prefix(";TYPE:") {
            feature = value;
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
            if feature == "Ironing" {
                *ironing_layers.last_mut().unwrap() = true;
            } else {
                *model_layers.last_mut().unwrap() = true;
            }
        }
    }
    assert_eq!(
        model_layers,
        vec![true; 5],
        "all five model layers must print"
    );
    assert_eq!(ironing_layers, [false, false, false, false, true]);
    assert_eq!(native.matches(";TYPE:Ironing\n").count(), 1);
    assert!(!native.contains(";TYPE:Brim\n"));
    assert!(!native.contains(";TYPE:Skirt\n"));
}

fn check(pattern: &str) {
    let label = format!("{pattern}-active");
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let fixture = crate::runner::repo_root().join("tests/parity/object-zag-ironing");
    let manifest: Value =
        serde_json::from_slice(&fs::read(fixture.join("manifest.json")).unwrap()).unwrap();
    let bytes = fs::read(fixture.join(format!("{label}.3mf"))).unwrap();
    assert_eq!(sha(&bytes), manifest["cases"][label.as_str()]["sha256"]);
    let ironing_type = "topmost";
    let settings = crate::runner::application::settings(&bytes).unwrap();
    for (key, value) in [
        ("ironing_pattern", pattern),
        ("ironing_type", ironing_type),
        ("top_shell_layers", "1"),
        ("print_sequence", "by layer"),
        ("skirt_loops", "0"),
        ("brim_width", "0"),
        ("enable_support", "0"),
        ("raft_layers", "0"),
        ("enable_prime_tower", "0"),
    ] {
        assert_eq!(settings[key], value, "{key}");
    }
    let root = tempfile::Builder::new()
        .prefix(&format!("object-zag-ironing-{label}-"))
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
            "wrapper":runner.bin(), "native_binary":native_binary,
            "native_binary_sha256":sha(&fs::read(&native_binary).unwrap()),
            "input_sha256":sha(&bytes), "args":args, "source":manifest["source"]
        }))
        .unwrap(),
    )
    .unwrap();
    stages::run(runner.bin(), args.clone(), stages::Stage::Slice, &root).unwrap();
    let reference = fs::read(root.join("native/plate_1.gcode")).unwrap();
    let native = std::str::from_utf8(&reference).unwrap();
    assert_native_active(native);
    for key in [
        format!("ironing_pattern = {pattern}"),
        format!("ironing_type = {ironing_type}"),
    ] {
        assert!(native.contains(&format!("; {key}\n")));
    }
    fs::create_dir(root.join("native-repeat")).unwrap();
    args[3] = root.join("native-repeat").to_str().unwrap().into();
    fs::write(
        root.join("native-repeat-launch.json"),
        serde_json::to_vec_pretty(&json!({"args":args})).unwrap(),
    )
    .unwrap();
    stages::run(runner.bin(), args, stages::Stage::Slice, &root).unwrap();
    let repeat = fs::read(root.join("native-repeat/plate_1.gcode")).unwrap();
    assert_native_active(std::str::from_utf8(&repeat).unwrap());
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
    assert!(
        difference.is_none(),
        "Native repeat must be deterministic: {difference:?}"
    );
    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project_plate_validated(
            &bytes,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ));
    let output = root.join("ares.cli.gcode");
    let cli = Command::new(env!("CARGO_BIN_EXE_ares"))
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
            "api_error":api.as_ref().err().map(|error| format!("{error:?}")),
            "cli_code":cli.status.code(), "cli_output_exists":output.exists()
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
