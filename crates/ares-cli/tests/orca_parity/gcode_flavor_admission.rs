//! PrintConfig.cpp:3904-3920: loaded gcode_flavor allowed-enum admission.

use crate::{OrcaRunner, golden, runner::stages};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, process::Command};

#[test]
fn gcode_flavor_admission_makerware_wipe0() {
    check("makerware-wipe0");
}

#[test]
fn gcode_flavor_admission_makerware_wipe1() {
    check("makerware-wipe1");
}

#[test]
fn gcode_flavor_admission_marlin2_control() {
    check("marlin2-wipe0");
}

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn check(label: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let fixture = crate::runner::repo_root().join("tests/parity/gcode-flavor-admission");
    let manifest: Value =
        serde_json::from_slice(&fs::read(fixture.join("manifest.json")).unwrap()).unwrap();
    let bytes = fs::read(fixture.join(format!("{label}.3mf"))).unwrap();
    assert_eq!(sha(&bytes), manifest["cases"][label]["sha256"]);
    let case = &manifest["cases"][label];
    let settings = crate::runner::application::settings(&bytes).unwrap();
    assert_eq!(settings["gcode_flavor"], case["gcode_flavor"]);
    assert_eq!(settings["wipe"], json!([case["wipe"].as_str().unwrap()]));
    assert_eq!(settings["enable_pressure_advance"], json!(["0"]));
    assert_eq!(settings["adaptive_pressure_advance"], json!(["0"]));
    let rejected = case["gcode_flavor"] == "makerware";
    let diagnostic = "gcode_flavor: invalid value makerware";
    let root = tempfile::Builder::new()
        .prefix(&format!("gcode-flavor-admission-{label}-"))
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
            "wrapper":runner.bin(),"native_binary":native_binary,
            "native_binary_sha256":sha(&fs::read(&native_binary).unwrap()),
            "input_sha256":sha(&bytes),"args":args,"source":"PrintConfig.cpp3904-3920"
        }))
        .unwrap(),
    )
    .unwrap();
    let native = stages::run(runner.bin(), args, stages::Stage::Slice, &root);
    if rejected {
        let error = native.unwrap_err();
        assert!(
            error.input_rejection("gcode_flavor"),
            "Native rejection {root:?}: {error}"
        );
        let captured = error.captured.unwrap();
        let expected_code = if cfg!(unix) { 238 } else { -18 };
        assert_eq!(captured.code, Some(expected_code));
        let text = format!(
            "{}\n{}",
            String::from_utf8_lossy(&captured.stdout),
            String::from_utf8_lossy(&captured.stderr)
        );
        assert!(text.lines().any(|line| line == diagnostic));
        assert!(!root.join("native/plate_1.gcode").exists());
        fs::write(
            root.join("native-rejection.json"),
            serde_json::to_vec_pretty(&json!({
                "code":captured.code,"stdout":String::from_utf8_lossy(&captured.stdout),
                "stderr":String::from_utf8_lossy(&captured.stderr),"diagnostic":diagnostic
            }))
            .unwrap(),
        )
        .unwrap();
    } else {
        native.unwrap();
        let result: Value =
            serde_json::from_slice(&fs::read(root.join("native/result.json")).unwrap()).unwrap();
        assert_eq!(result["return_code"], 0);
        let text = fs::read_to_string(root.join("native/plate_1.gcode")).unwrap();
        assert!(text.contains("; gcode_flavor = marlin2\n"));
        assert!(
            text.lines()
                .any(|line| line.starts_with("G1 X") && line.contains(" E"))
        );
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
    if let Ok(gcode) = &api {
        fs::write(root.join("ares.api.gcode"), gcode).unwrap();
    }
    let output = root.join("ares.cli.gcode");
    let binary = env!("CARGO_BIN_EXE_ares");
    let cli = Command::new(binary)
        .args(["slice", "-o"])
        .arg(&output)
        .arg(root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(root.join("ares.cli.stdout"), &cli.stdout).unwrap();
    fs::write(root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    let cli_bytes = fs::read(&output);
    let api_error_matches = matches!(&api, Err(ares_core::SliceError::InvalidValuesInProject(text)) if text.lines().any(|line| line == diagnostic));
    let cli_error_matches = cli.status.code() == Some(if cfg!(unix) { 238 } else { -18 })
        && String::from_utf8_lossy(&cli.stderr).contains(diagnostic)
        && !output.exists();
    let comparison = if rejected {
        None
    } else {
        let reference = fs::read(root.join("native/plate_1.gcode")).unwrap();
        Some((
            api.as_ref()
                .map_err(|e| format!("{e:?}"))
                .and_then(|g| golden::compare_ordered_bytes_generator_only(&reference, g)),
            cli_bytes
                .as_ref()
                .map_err(|e| e.to_string())
                .and_then(|g| golden::compare_ordered_bytes_generator_only(&reference, g)),
        ))
    };
    fs::write(root.join("comparison.json"), serde_json::to_vec_pretty(&json!({
        "case":label,"rejection_required":rejected,"api_error":api.as_ref().err().map(|e|format!("{e:?}")),
        "api_error_matches":api_error_matches,"cli_code":cli.status.code(),"cli_error_matches":cli_error_matches,
        "cli_output_exists":output.exists(),"api_gcode_exists":root.join("ares.api.gcode").exists(),
        "control_comparison":comparison,"raw_api_cli_equal":api.as_ref().ok().zip(cli_bytes.as_ref().ok()).map(|(a,b)|a==b),
        "cli_binary_sha256":sha(&fs::read(binary).unwrap())
    })).unwrap()).unwrap();
    if rejected {
        assert!(
            api_error_matches,
            "public API must reject gcode_flavor {root:?}; got {:?}",
            api.as_ref().err()
        );
        assert!(
            cli_error_matches,
            "CLI must reject/no G-code {root:?}: {cli:?}"
        );
    } else {
        assert!(api.is_ok(), "public API control admission {root:?}");
        assert!(cli.status.success(), "CLI control admission {root:?}");
        let (api_comparison, cli_comparison) = comparison.unwrap();
        api_comparison.unwrap_or_else(|e| panic!("{root:?}: {e}"));
        cli_comparison.unwrap_or_else(|e| panic!("{root:?}: {e}"));
        assert_eq!(
            api.unwrap(),
            cli_bytes.unwrap(),
            "raw control API/CLI {root:?}"
        );
    }
}
