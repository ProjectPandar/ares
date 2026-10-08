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
    assert_eq!(
        output.stdout,
        fs::read(fixtures.join(format!("{case}.stdout"))).unwrap()
    );
    assert_eq!(
        output.stderr,
        fs::read(fixtures.join(format!("{case}.stderr"))).unwrap()
    );
    assert_eq!(
        fs::read(artifacts.join("result.json")).unwrap(),
        fs::read(fixtures.join(format!("{case}.result.json"))).unwrap(),
        "complete native JSON: generic error, applied metadata"
    );
    assert!(
        !retained.exists(),
        "no final exported G-code on custom-template failure"
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
