//! Fill.cpp:1611-1640,1705-1731; FillBase.cpp:61;
//! FillPlanePath.cpp:69-200 (object, not support, ironing).

use crate::{OrcaRunner, golden, runner::stages};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;

#[test]
fn object_archimedean_ironing_target() {
    check("archimedean-active");
}

#[test]
fn object_archimedean_ironing_rectilinear_control() {
    check("rectilinear-control");
}

#[test]
fn object_archimedean_ironing_disabled_control() {
    check("archimedean-disabled-control");
}

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn assert_native_active(native: &str, ironing: bool) {
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
    assert_eq!(ironing_layers, [false, false, false, false, ironing]);
    assert_eq!(
        native.matches(";TYPE:Ironing\n").count(),
        usize::from(ironing)
    );
    assert!(!native.contains(";TYPE:Brim\n"));
    assert!(!native.contains(";TYPE:Skirt\n"));
}

fn check(label: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let fixture = crate::runner::repo_root().join("tests/parity/object-archimedean-ironing");
    let manifest: Value =
        serde_json::from_slice(&fs::read(fixture.join("manifest.json")).unwrap()).unwrap();
    let bytes = fs::read(fixture.join(format!("{label}.3mf"))).unwrap();
    assert_eq!(sha(&bytes), manifest["cases"][label]["sha256"]);
    let pattern = if label == "rectilinear-control" {
        "rectilinear"
    } else {
        "archimedeanchords"
    };
    let ironing = label != "archimedean-disabled-control";
    let ironing_type = if ironing { "topmost" } else { "no ironing" };
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
        .prefix(&format!("object-archimedean-ironing-{label}-"))
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
    assert_native_active(native, ironing);
    for key in [
        format!("ironing_pattern = {pattern}"),
        format!("ironing_type = {ironing_type}"),
    ] {
        assert!(native.contains(&format!("; {key}\n")));
    }
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
