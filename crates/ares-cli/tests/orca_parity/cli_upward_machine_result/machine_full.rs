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

#[test]
fn original_resource_source_and_shape_controls() {
    if OrcaRunner::from_env().unwrap().is_none() {
        return;
    }
    let root = tempfile::Builder::new()
        .prefix("upward-source-controls-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input =
        fs::read(runner::repo_root().join("tests/parity/master-extruder-id/stock-right-only.3mf"))
            .unwrap();
    let select = |resource: Option<&[u8]>| {
        UpwardMachineSource::from_project(&input)
            .unwrap()
            .unwrap()
            .select(UpwardMachineInput::FetchOriginal(resource))
    };
    let embedded = UpwardMachineSource::from_project(&input)
        .unwrap()
        .unwrap()
        .select(UpwardMachineInput::Embedded)
        .unwrap();
    assert_eq!(select(None).unwrap(), embedded);
    let mut machine = json!({"type":"machine", "name":"Bambu Lab H2D 0.4 nozzle",
        "from":"system", "upward_compatible_machine":["one", "one", "two"]});
    for source in ["system", "User", "user"] {
        machine["from"] = json!(source);
        assert_eq!(
            select(Some(&serde_json::to_vec(&machine).unwrap())).unwrap(),
            ["one", "one", "two"]
        );
    }
    machine["from"] = json!("unsupported");
    let source_error = select(Some(&serde_json::to_vec(&machine).unwrap()))
        .unwrap_err()
        .to_string();
    assert_eq!(source_error, "unsupported machine source from metadata");
    machine["from"] = json!("system");
    machine["upward_compatible_machine"] = json!([]);
    assert!(
        select(Some(&serde_json::to_vec(&machine).unwrap()))
            .unwrap()
            .is_empty()
    );
    machine["type"] = json!("unknown");
    let type_error = select(Some(&serde_json::to_vec(&machine).unwrap()))
        .unwrap_err()
        .to_string();
    assert_eq!(type_error, "profile type is unsupported");
    machine.as_object_mut().unwrap().remove("type");
    let missing_type = select(Some(&serde_json::to_vec(&machine).unwrap()))
        .unwrap_err()
        .to_string();
    assert_eq!(missing_type, "profile type is required");
    let malformed = select(Some(b"{")).unwrap_err().to_string();
    assert!(malformed.starts_with("profile JSON is invalid:"));
    let original = fs::read(
        runner::repo_root().join("tests/parity/machine-full-resource/stock-machine-full.json"),
    )
    .unwrap();
    let mut inherited: Value = serde_json::from_slice(&original).unwrap();
    inherited["inherits"] = json!("parent");
    let inherited_error = select(Some(&serde_json::to_vec(&inherited).unwrap()))
        .unwrap_err()
        .to_string();
    assert!(inherited_error.contains("unknown or misplaced profile option cooling_filter_enabled"));
    let explicit_json = String::from_utf8(original).unwrap();
    let explicit_file = ares_core::LoadSettingsFile {
        label: "stock-machine-full.json",
        json: &explicit_json,
    };
    let explicit_error = UpwardMachineSource::from_project(&input)
        .unwrap()
        .unwrap()
        .select(UpwardMachineInput::UpToDateFiles(&[explicit_file]))
        .unwrap_err()
        .to_string();
    assert!(explicit_error.contains("unknown or misplaced profile option cooling_filter_enabled"));
    fs::write(root.join("controls.json"), serde_json::to_vec_pretty(&json!({
        "input_sha256":digest(&input), "embedded":embedded, "allowed_sources":["system","User","user"],
        "ordered_with_duplicates":["one","one","two"], "present_empty_replaces":true,
        "source_error":source_error, "type_error":type_error, "missing_type":missing_type,
        "malformed":malformed, "inherited_error":inherited_error, "explicit_error":explicit_error
    })).unwrap()).unwrap();
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
    let ares_dir = root.join("ares");
    fs::create_dir(&ares_dir).unwrap();
    let ares = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "--resources-dir"])
        .arg(&resources)
        .arg("-o")
        .arg(ares_dir.join("plate_1.gcode"))
        .arg(&input_path)
        .output()
        .unwrap();
    channels(&root, "ares", &ares);
    let report = fs::read(ares_dir.join("result.json")).ok();
    let report_json = report
        .as_deref()
        .map(serde_json::from_slice::<Value>)
        .transpose()
        .unwrap();
    fs::write(root.join("observation.json"), serde_json::to_vec_pretty(&json!({
        "case": label, "input_sha256": digest(&input), "resource_sha256": digest(resource.as_bytes()),
        "resource_path": machine_path, "native_bin": native_bin.to_string_lossy(), "native_argv": native_argv,
        "raw_cli": env!("CARGO_BIN_EXE_ares"), "api": api, "refreshed_api": refreshed_api,
        "strict_fragment_rejected": strict_fragment_rejected,
        "ares_exit": ares.status.code(), "ares_report": report_json,
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
    assert_eq!(api, Ok(expected_upward.clone()), "FetchOriginal: {root:?}");
    assert_eq!(
        refreshed_api,
        Ok(expected_upward.clone()),
        "UpToDateOriginal: {root:?}"
    );
    let report = report_json.expect("resource admission must produce its report");
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
