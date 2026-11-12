//! OrcaSlicer.cpp:2118–2123,2205–2263,2820–2842; flattened full-config resources.
//! Failure modes: partial-profile rejection, accidental printer-only filtering,
//! unknown-key retention, object-to-bool coercion, or resource values overlaying slicing.
use std::{fs, path::Path, process::Command};

use ares_core::{ProfileFragment, UpwardMachineInput, UpwardMachineSource};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::runner::{self, OrcaRunner};

#[test]
fn stock_original_resource_uses_full_config_reader() {
    exercise("stock", None, "0", "0.2");
}

#[test]
fn cooling_enabled_resource_uses_full_config_reader() {
    exercise(
        "cooling-enabled",
        Some(("cooling_filter_enabled", json!("1"))),
        "1",
        "0.2",
    );
}

#[test]
fn print_option_resource_uses_full_config_reader() {
    exercise(
        "print-option",
        Some(("layer_height", json!("0.21"))),
        "0",
        "0.21",
    );
}

#[test]
fn unknown_option_resource_is_deleted() {
    exercise(
        "unknown-option",
        Some(("ares_unknown_machine_option", json!("1"))),
        "0",
        "0.2",
    );
}

#[test]
fn non_bool_resource_value_preserves_default() {
    exercise(
        "non-bool",
        Some(("cooling_filter_enabled", json!({"invalid":"1"}))),
        "0",
        "0.2",
    );
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn channels(root: &Path, prefix: &str, output: &std::process::Output) {
    fs::write(root.join(format!("{prefix}.stdout")), &output.stdout).unwrap();
    fs::write(root.join(format!("{prefix}.stderr")), &output.stderr).unwrap();
    fs::write(
        root.join(format!("{prefix}.exit")),
        format!("{:?}\n", output.status.code()),
    )
    .unwrap();
}

fn exercise(label: &str, mutation: Option<(&str, Value)>, cooling: &str, height: &str) {
    if OrcaRunner::from_env().unwrap().is_none() {
        return;
    }
    let root = tempfile::Builder::new()
        .prefix(&format!("machine-full-{label}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let fixture = runner::repo_root().join("tests/parity/machine-full-resource");
    let original = fs::read(fixture.join("stock-machine-full.json")).unwrap();
    assert_eq!(
        digest(&original),
        "2a2e89e53063119290a5b8c185ea01f4e207044dc11f4e78f5a34cd22d162e4b"
    );
    let mut resource = String::from_utf8(original.clone()).unwrap();
    if let Some((key, value)) = mutation {
        let original_values: Value = serde_json::from_slice(&original).unwrap();
        resource = if let Some(previous) = original_values.get(key) {
            resource.replacen(
                &format!("\"{key}\": {previous}"),
                &format!("\"{key}\": {value}"),
                1,
            )
        } else {
            resource.replacen("{\n", &format!("{{\n    \"{key}\": {value},\n"), 1)
        };
        let mut expected = original_values;
        expected[key] = value;
        assert_eq!(serde_json::from_str::<Value>(&resource).unwrap(), expected);
    }
    let input =
        fs::read(runner::repo_root().join("tests/parity/master-extruder-id/stock-right-only.3mf"))
            .unwrap();
    assert_eq!(
        digest(&input),
        "69d425b79297129c1b3894ee106269d3b33b7f1badcbeed15598dfe774cff09a"
    );
    let input_path = root.join("stock-right-only.3mf");
    fs::write(&input_path, &input).unwrap();
    let source = UpwardMachineSource::from_project(&input).unwrap().unwrap();
    let resources = root.join("resources");
    let machine_dir = resources.join("profiles/BBL/machine_full");
    fs::create_dir_all(&machine_dir).unwrap();
    let machine_path = machine_dir.join(format!("{}.json", source.system_printer()));
    fs::write(&machine_path, &resource).unwrap();
    let api = source
        .select(UpwardMachineInput::FetchOriginal(Some(resource.as_bytes())))
        .map_err(|error| error.to_string());
    let refreshed_api = UpwardMachineSource::from_project(&input)
        .unwrap()
        .unwrap()
        .select(UpwardMachineInput::UpToDateOriginal(Some(
            resource.as_bytes(),
        )))
        .map_err(|error| error.to_string());
    let strict_fragment_rejected = ProfileFragment::from_json_bytes(resource.as_bytes()).is_err();
    let native_dir = root.join("native");
    fs::create_dir(&native_dir).unwrap();
    let native_bin = std::env::var_os("ARES_ORCA_BIN").unwrap();
    let native_argv = vec![
        "--load-settings".to_owned(),
        machine_path.display().to_string(),
        "--slice".to_owned(),
        "0".to_owned(),
        "--outputdir".to_owned(),
        native_dir.display().to_string(),
        input_path.display().to_string(),
    ];
    let native = Command::new(&native_bin)
        .args(&native_argv)
        .current_dir(&root)
        .output()
        .unwrap();
    channels(&root, "native", &native);
    let baseline_dir = root.join("baseline");
    let ares_dir = root.join("ares");
    fs::create_dir(&baseline_dir).unwrap();
    fs::create_dir(&ares_dir).unwrap();
    let baseline = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(baseline_dir.join("plate_1.gcode"))
        .arg(&input_path)
        .output()
        .unwrap();
    channels(&root, "baseline", &baseline);
    let ares = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "--resources-dir"])
        .arg(&resources)
        .arg("-o")
        .arg(ares_dir.join("plate_1.gcode"))
        .arg(&input_path)
        .output()
        .unwrap();
    channels(&root, "ares", &ares);
    fs::write(root.join("observation.json"), serde_json::to_vec_pretty(&json!({
        "case": label, "input_sha256": digest(&input), "resource_sha256": digest(resource.as_bytes()),
        "resource_path": machine_path, "native_bin": native_bin, "native_argv": native_argv,
        "raw_cli": env!("CARGO_BIN_EXE_ares"), "api": api, "refreshed_api": refreshed_api,
        "strict_fragment_rejected": strict_fragment_rejected,
        "baseline_exit": baseline.status.code(), "ares_exit": ares.status.code(),
        "native_exit": native.status.code(),
        "scope": "Full-config admission and upward selection, not complete Native/Ares slicing parity"
    })).unwrap()).unwrap();
    eprintln!("machine_full {label}: artifacts {root:?}");
    assert_eq!(native.status.code(), Some(0), "{root:?}");
    let native_report: Value =
        serde_json::from_slice(&fs::read(native_dir.join("result.json")).unwrap()).unwrap();
    assert_eq!(native_report["return_code"], 0, "{root:?}");
    let expected_upward = vec!["Bambu Lab H2D Pro 0.4 nozzle".to_owned()];
    assert_eq!(
        native_report["upward_compatible_machine"],
        json!(expected_upward),
        "{root:?}"
    );
    let native_gcode = fs::read_to_string(native_dir.join("plate_1.gcode")).unwrap();
    assert!(
        native_gcode
            .lines()
            .any(|line| line == format!("; cooling_filter_enabled = {cooling}")),
        "{root:?}"
    );
    assert!(
        native_gcode
            .lines()
            .any(|line| line == format!("; layer_height = {height}")),
        "{root:?}"
    );
    assert!(
        !native_gcode.contains("ares_unknown_machine_option"),
        "{root:?}"
    );
    assert!(
        strict_fragment_rejected,
        "strict public API is unchanged: {root:?}"
    );
    assert_eq!(baseline.status.code(), Some(0), "{root:?}");
    assert_eq!(api, Ok(expected_upward.clone()), "FetchOriginal: {root:?}");
    assert_eq!(
        refreshed_api,
        Ok(expected_upward.clone()),
        "UpToDateOriginal: {root:?}"
    );
    assert_eq!(ares.status.code(), Some(0), "raw CLI: {root:?}");
    assert_eq!(ares.stdout, baseline.stdout, "{root:?}");
    assert_eq!(ares.stderr, baseline.stderr, "{root:?}");
    assert_eq!(
        fs::read(ares_dir.join("plate_1.gcode")).unwrap(),
        fs::read(baseline_dir.join("plate_1.gcode")).unwrap(),
        "resource must not overlay slicing: {root:?}"
    );
    let report = fs::read(ares_dir.join("result.json")).unwrap();
    assert_eq!(
        report,
        fs::read(baseline_dir.join("result.json")).unwrap(),
        "complete resource/no-resource result bytes: {root:?}"
    );
    let report: Value = serde_json::from_slice(&report).unwrap();
    assert_eq!(
        report["upward_compatible_machine"],
        json!(expected_upward),
        "{root:?}"
    );
    assert_eq!(
        digest(&fs::read(&machine_path).unwrap()),
        digest(resource.as_bytes())
    );
    assert_eq!(digest(&fs::read(&input_path).unwrap()), digest(&input));
}
