//! GCode.cpp:426-490; GCodeWriter.cpp:933-952: zero-E wipe description.

use crate::{OrcaRunner, golden, runner::stages};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, process::Command};

#[test]
fn orca_parity_wipe_zero_retraction_comment_before_50() {
    check("before-50");
}

#[test]
fn orca_parity_wipe_zero_retraction_comment_before_100() {
    check("before-100");
}

#[test]
fn orca_parity_wipe_zero_retraction_comment_zero_retraction() {
    check("zero-retraction");
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
    let fixture = crate::runner::repo_root().join("tests/parity/wipe-zero-retraction-comment");
    let manifest: Value =
        serde_json::from_slice(&fs::read(fixture.join("manifest.json")).unwrap()).unwrap();
    let bytes = fs::read(fixture.join(format!("{label}.3mf"))).unwrap();
    assert_eq!(sha(&bytes), manifest["cases"][label]["sha256"]);
    let settings = crate::runner::application::settings(&bytes).unwrap();
    let case = &manifest["cases"][label];
    for key in ["retract_before_wipe", "retraction_length", "wipe"] {
        assert_eq!(settings[key], json!([case[key].as_str().unwrap()]));
    }
    assert_eq!(settings["gcode_comments"], "1");
    let root = tempfile::Builder::new()
        .prefix(&format!("wipe-zero-retraction-{label}-"))
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
            "args":args,"input_sha256":sha(&bytes),
            "source":"GCode.cpp426-490; GCodeWriter.cpp933-952; Extruder.cpp168-172"
        }))
        .unwrap(),
    )
    .unwrap();
    stages::run(runner.bin(), args, stages::Stage::Slice, &root).unwrap();
    let result: Value =
        serde_json::from_slice(&fs::read(root.join("native/result.json")).unwrap()).unwrap();
    assert_eq!(result["return_code"], 0, "Native admission {root:?}");
    let reference = fs::read(root.join("native/plate_1.gcode")).unwrap();
    let text = std::str::from_utf8(&reference).unwrap();
    for key in ["retract_before_wipe", "retraction_length", "wipe"] {
        assert!(text.contains(&format!("; {key} = {}\n", case[key].as_str().unwrap())));
    }
    let moves: Vec<_> = text
        .split(";WIPE_START\n")
        .skip(1)
        .flat_map(|block| {
            block
                .split(";WIPE_END")
                .next()
                .unwrap()
                .lines()
                .filter(|line| line.starts_with("G1 X") && line.contains(" Y"))
        })
        .collect();
    assert!(!moves.is_empty(), "Native real XY wipe absent {root:?}");
    assert!(
        moves
            .iter()
            .all(|line| line.ends_with("; wipe and retract")),
        "Native description {root:?}"
    );
    let zero_e = label != "before-50";
    assert!(
        moves.iter().all(|line| line.contains(" E") != zero_e),
        "Native E omission/presence {root:?}"
    );
    fs::write(root.join("witness.json"), serde_json::to_vec_pretty(&json!({
        "case":label,"zero_E":zero_e,"wipe_blocks":text.matches(";WIPE_START\n").count(),"wipe_moves":moves
    })).unwrap()).unwrap();
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
    if let Ok(gcode) = &api {
        fs::write(root.join("ares.api.gcode"), gcode).unwrap();
    }
    let api_comparison = api
        .as_ref()
        .map_err(|e| format!("API: {e:?}"))
        .and_then(|g| golden::compare_ordered_bytes_generator_only(&reference, g));
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
    let cli_comparison = cli_bytes
        .as_ref()
        .map_err(|e| format!("CLI: {e}"))
        .and_then(|g| golden::compare_ordered_bytes_generator_only(&reference, g));
    fs::write(
        root.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "case":label,"api_error":api.as_ref().err().map(|e|format!("{e:?}")),
            "api_comparison":api_comparison,"cli_code":cli.status.code(),
            "cli_comparison":cli_comparison,"cli_binary_sha256":sha(&fs::read(binary).unwrap()),
            "strict_parity_required":true
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(api.is_ok(), "public API admission RED {root:?}: {api:?}");
    assert!(cli.status.success(), "CLI admission RED {root:?}: {cli:?}");
    api_comparison.unwrap_or_else(|e| panic!("{root:?}: {e}"));
    cli_comparison.unwrap_or_else(|e| panic!("{root:?}: {e}"));
    assert_eq!(
        api.unwrap(),
        cli_bytes.unwrap(),
        "API/CLI raw equality {root:?}"
    );
}
