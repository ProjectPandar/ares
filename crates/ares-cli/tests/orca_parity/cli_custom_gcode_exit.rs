//! GCode.cpp:3640-3645 PlaceholderParserError -> OrcaSlicer.cpp:6286-6291.
//! Fresh Native seed0/cpu1 captured all five stock inputs with Unix156.
#![cfg(target_os = "linux")]

use std::{fs, path::Path, process::Command};

fn assert_native_failure(case: &str) {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/parity/validation/cli-exit/custom-gcode");
    let artifacts = tempfile::Builder::new()
        .prefix(&format!("cli-custom-gcode-{case}-"))
        .tempdir()
        .unwrap()
        .keep();
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };
    let native = runner
        .custom_gcode_reference(&fixtures.join(format!("{case}.3mf")))
        .unwrap();
    let hit = runner
        .custom_gcode_reference(&fixtures.join(format!("{case}.3mf")))
        .unwrap();
    assert_eq!(native.directory, hit.directory);
    assert_eq!(native.stdout, hit.stdout);
    assert_eq!(native.stderr, hit.stderr);
    assert_eq!(native.result_json, hit.result_json);
    fs::write(artifacts.join("native.stdout"), &native.stdout).unwrap();
    fs::write(artifacts.join("native.stderr"), &native.stderr).unwrap();
    fs::write(artifacts.join("native.result.json"), &native.result_json).unwrap();
    fs::copy(
        native.directory.join("receipt.json"),
        artifacts.join("native.receipt.json"),
    )
    .unwrap();
    let retained = artifacts.join("plate_1.gcode");
    let output = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(&retained)
        .arg(fixtures.join(format!("{case}.3mf")))
        .output()
        .unwrap();
    fs::write(artifacts.join("stdout"), &output.stdout).unwrap();
    fs::write(artifacts.join("stderr"), &output.stderr).unwrap();
    fs::write(
        artifacts.join("exit"),
        format!("{:?}\n", output.status.code()),
    )
    .unwrap();
    assert_eq!(output.status.code(), Some(156), "{case}: {artifacts:?}");
    assert_eq!(output.stdout, native.stdout);
    assert_eq!(output.stderr, native.stderr);
    assert_eq!(
        result_comparison_bytes(&fs::read(artifacts.join("result.json")).unwrap()),
        result_comparison_bytes(&native.result_json),
        "complete native JSON: generic error, applied metadata"
    );
    assert!(
        !retained.exists(),
        "no final exported G-code on custom-template failure"
    );
}

fn result_comparison_bytes(bytes: &[u8]) -> Vec<u8> {
    // OrcaSlicer.cpp:5540/436 and :7104/437 serialize UTC elapsed seconds.
    let mut json = std::str::from_utf8(bytes).unwrap().to_owned();
    for field in ["prepare_time", "export_time"] {
        let prefix = format!("\"{field}\": ");
        let start = json.find(&prefix).unwrap() + prefix.len();
        let digits = json[start..].bytes().take_while(u8::is_ascii_digit).count();
        json.replace_range(start..start + digits, "0");
    }
    json.into_bytes()
}

#[test]
fn custom_gcode_json_comparison_ignores_only_wall_clock_values() {
    let actual = br#"{"prepare_time": 0, "export_time": 0, "return_code": -100, "error_string": "numbers 27 12", "untouched": 27}"#;
    let native = br#"{"prepare_time": 27, "export_time": 12, "return_code": -100, "error_string": "numbers 27 12", "untouched": 27}"#;
    assert_eq!(
        result_comparison_bytes(actual),
        result_comparison_bytes(native)
    );
    assert_eq!(result_comparison_bytes(native).as_slice(), actual);
}

#[test]
fn custom_gcode_json_comparison_preserves_other_bytes() {
    let actual = br#"{"prepare_time": 0, "export_time": 0, "return_code": -100}"#;
    for different in [
        br#"{"prepare_time": 0, "export_time": 0, "return_code": -101}"#.as_slice(),
        br#"{"prepare_time": 0, "export_time": 0, "return_code":  -100}"#.as_slice(),
        br#"{"export_time": 0, "prepare_time": 0, "return_code": -100}"#.as_slice(),
    ] {
        assert_ne!(
            result_comparison_bytes(actual),
            result_comparison_bytes(different)
        );
    }
}

#[test]
fn custom_gcode_reference_requires_native_even_when_frozen_outputs_exist() {
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "cli_custom_gcode_exit::cli_custom_gcode_flsun_qqs_matches_native",
            "--nocapture",
        ])
        .env("ARES_REQUIRE_ORCA", "1")
        .env_remove("ARES_ORCA_NATIVE_BIN")
        .output()
        .unwrap();
    let diagnostic = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        diagnostic.contains("running 1 test"),
        "selector drift: {diagnostic}"
    );
    assert!(
        !output.status.success() && diagnostic.contains("ARES_ORCA_NATIVE_BIN"),
        "frozen reference concealed missing required Native: {diagnostic}"
    );
}

#[test]
fn cli_custom_gcode_flsun_qqs_matches_native() {
    assert_native_failure("flsun-qqs");
}

#[test]
fn cli_custom_gcode_ginger_1_2_matches_native() {
    assert_native_failure("ginger-1_2");
}

#[test]
fn cli_custom_gcode_ginger_3_0_matches_native() {
    assert_native_failure("ginger-3_0");
}

#[test]
fn cli_custom_gcode_ginger_5_0_matches_native() {
    assert_native_failure("ginger-5_0");
}

#[test]
fn cli_custom_gcode_ginger_8_0_matches_native() {
    assert_native_failure("ginger-8_0");
}
