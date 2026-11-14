//! OrcaSlicer.cpp:1728–1760,2280–2374,2922–2945,3005,3237–3259.
//! Risks: omitted kinds replaced by defaults, wrong compatibility or variants,
//! lost upward metadata, missing output mistaken for whole-output equality.

use std::{fs, path::Path, process::Command};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{native_wall_clock::result_comparison_bytes, runner::OrcaRunner};

#[test]
fn stock_machine_preserves_embedded_kinds() {
    exercise("stock", false);
}

#[test]
fn empty_inherits_machine_preserves_embedded_kinds() {
    exercise("empty-inherits", true);
}

#[test]
fn catalog_keeps_filament_conflict() {
    catalog_conflict("--load-filaments");
}

#[test]
fn catalog_keeps_resource_conflict() {
    catalog_conflict("--resources-dir");
}

fn catalog_conflict(conflicting: &str) {
    let root = tempfile::Builder::new()
        .prefix("machine-catalog-conflict-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let argv = [
        "slice",
        "--project-preset-catalog",
        "catalog.json",
        conflicting,
        "conflicting-input",
        "-o",
        "output.gcode",
        "input.3mf",
    ];
    fs::write(
        root.join("argv.json"),
        serde_json::to_vec_pretty(&argv).unwrap(),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(argv)
        .current_dir(&root)
        .output()
        .unwrap();
    channels(&root, "ares", &output);
    assert_eq!(output.status.code(), Some(2), "{root:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--project-preset-catalog") && stderr.contains(conflicting),
        "{stderr}; {root:?}"
    );
    assert!(output.stdout.is_empty());
    assert!(!root.join("output.gcode").exists());
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

fn exercise(label: &str, empty_inherits: bool) {
    if OrcaRunner::from_env().unwrap().is_none() {
        return;
    }
    let root = tempfile::Builder::new()
        .prefix(&format!("machine-only-cli-{label}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let repo = crate::runner::repo_root();
    let original =
        fs::read(repo.join("tests/parity/machine-full-resource/stock-machine-full.json")).unwrap();
    assert_eq!(
        digest(&original),
        "2a2e89e53063119290a5b8c185ea01f4e207044dc11f4e78f5a34cd22d162e4b"
    );
    let mut machine = String::from_utf8(original.clone()).unwrap();
    if empty_inherits {
        machine = machine.replacen("{\n", "{\n    \"inherits\": \"\",\n", 1);
        let mut expected: Value = serde_json::from_slice(&original).unwrap();
        expected["inherits"] = json!("");
        assert_eq!(serde_json::from_str::<Value>(&machine).unwrap(), expected);
    } else {
        assert_eq!(machine.as_bytes(), original);
    }
    let input =
        fs::read(repo.join("tests/parity/master-extruder-id/stock-right-only.3mf")).unwrap();
    assert_eq!(
        digest(&input),
        "69d425b79297129c1b3894ee106269d3b33b7f1badcbeed15598dfe774cff09a"
    );
    let input_path = root.join("stock-right-only.3mf");
    let machine_path = root.join("machine.json");
    fs::write(&input_path, &input).unwrap();
    fs::write(&machine_path, &machine).unwrap();
    let native_dir = root.join("native");
    fs::create_dir(&native_dir).unwrap();
    let native_bin = std::env::var_os("ARES_ORCA_BIN").unwrap();
    let supply = crate::runner::application::project_supply().unwrap();
    fs::write(
        root.join("catalog-supply.json"),
        serde_json::to_vec_pretty(&supply.map(|supply| {
            json!({
                "catalog_path": supply.catalog_path, "catalog_sha256": supply.digest,
                "native_dir": supply.native_dir
            })
        }))
        .unwrap(),
    )
    .unwrap();
    if let Some(supply) = supply {
        fs::write(root.join("catalog.json"), &supply.catalog).unwrap();
    }
    let native_argv = crate::runner::application::native_arguments(vec![
        "--load-settings".to_owned(),
        machine_path.display().to_string(),
        "--slice".to_owned(),
        "0".to_owned(),
        "--outputdir".to_owned(),
        native_dir.display().to_string(),
        input_path.display().to_string(),
    ])
    .unwrap();
    fs::write(
        root.join("native.argv.json"),
        serde_json::to_vec_pretty(&native_argv).unwrap(),
    )
    .unwrap();
    let native = Command::new(&native_bin)
        .args(&native_argv)
        .current_dir(&root)
        .output()
        .unwrap();
    channels(&root, "native", &native);
    let native_report = fs::read(native_dir.join("result.json")).unwrap();
    let native_json: Value = serde_json::from_slice(&native_report).unwrap();
    let native_gcode = fs::read(native_dir.join("plate_1.gcode")).unwrap();
    let expected_upward = json!(["Bambu Lab H2D Pro 0.4 nozzle"]);
    fs::write(
        root.join("native-premise.json"),
        serde_json::to_vec_pretty(&json!({
            "exit":native.status.code(), "report":native_json,
            "gcode_bytes":native_gcode.len(), "gcode_sha256":digest(&native_gcode)
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(native.status.code(), Some(0), "{root:?}");
    assert_eq!(native_json["return_code"], 0, "{root:?}");
    assert_eq!(native_json["upward_compatible_machine"], expected_upward);
    assert!(!native_gcode.is_empty(), "{root:?}");

    let ares_dir = root.join("ares");
    fs::create_dir(&ares_dir).unwrap();
    let mut ares_argv = vec![
        "slice".to_owned(),
        "--load-settings".to_owned(),
        machine_path.display().to_string(),
        "-o".to_owned(),
        ares_dir.join("plate_1.gcode").display().to_string(),
        input_path.display().to_string(),
    ];
    if let Some(supply) = supply {
        ares_argv.splice(
            1..1,
            [
                "--project-preset-catalog".to_owned(),
                supply.catalog_path.display().to_string(),
            ],
        );
    }
    fs::write(
        root.join("ares.argv.json"),
        serde_json::to_vec_pretty(&ares_argv).unwrap(),
    )
    .unwrap();
    let ares = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(&ares_argv)
        .current_dir(&root)
        .output()
        .unwrap();
    channels(&root, "ares", &ares);
    let ares_report = fs::read(ares_dir.join("result.json")).ok();
    let ares_json = ares_report
        .as_deref()
        .map(serde_json::from_slice::<Value>)
        .transpose()
        .unwrap();
    let ares_gcode = fs::read(ares_dir.join("plate_1.gcode")).ok();
    let report_equal = ares_report.as_deref().is_some_and(|bytes| {
        result_comparison_bytes(bytes) == result_comparison_bytes(&native_report)
    });
    let gcode_equal = ares_gcode.as_deref().is_some_and(|bytes| {
        super::banner_normalized(bytes) == super::banner_normalized(&native_gcode)
    });
    let upward_equal = ares_json
        .as_ref()
        .is_some_and(|report| report["upward_compatible_machine"] == expected_upward);
    fs::write(root.join("comparison.json"), serde_json::to_vec_pretty(&json!({
        "case":label, "empty_inherits":empty_inherits,
        "input_sha256":digest(&input), "machine_sha256":digest(machine.as_bytes()),
        "raw_cli":env!("CARGO_BIN_EXE_ares"), "native_bin":native_bin.to_string_lossy(),
        "native_exit":native.status.code(), "ares_exit":ares.status.code(),
        "native_report":native_json, "ares_report":ares_json,
        "native_gcode_sha256":digest(&native_gcode),
        "ares_gcode_sha256":ares_gcode.as_deref().map(digest),
        "whole_report_equal":report_equal, "upward_equal":upward_equal,
        "whole_gcode_equal":gcode_equal,
        "comparators":["native_wall_clock::result_comparison_bytes","load_settings_cli::banner_normalized"]
    })).unwrap()).unwrap();
    eprintln!("machine_only {label}: artifacts {root:?}");
    assert_eq!(fs::read(&input_path).unwrap(), input);
    assert_eq!(fs::read(&machine_path).unwrap(), machine.as_bytes());
    assert_eq!(ares.status.code(), native.status.code(), "exit: {root:?}");
    assert!(upward_equal, "complete upward metadata: {root:?}");
    assert!(report_equal, "whole result.json: {root:?}");
    assert!(gcode_equal, "whole G-code: {root:?}");
}
