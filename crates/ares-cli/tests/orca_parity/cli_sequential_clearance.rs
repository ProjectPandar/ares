//! Print.cpp:613-930,1298-1301 -> CLI_OBJECT_COLLISION_IN_SEQ_PRINT(-63).
#![cfg(target_os = "linux")]

use std::{fs, path::Path, process::Command};

use crate::native_wall_clock::result_comparison_bytes;

fn run(case: &str, expected_code: i32) {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/parity/validation/sequential-clearance");
    let artifacts = tempfile::Builder::new()
        .prefix(&format!("cli-s4-{case}-"))
        .tempdir()
        .unwrap()
        .keep();
    let Some(runner) = crate::runner::OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project = fixtures.join(format!("{case}.3mf"));
    let native = runner
        .sequential_clearance_reference(&project, expected_code == 0)
        .unwrap();
    let hit = runner
        .sequential_clearance_reference(&project, expected_code == 0)
        .unwrap();
    assert_eq!(native.directory, hit.directory);
    assert_eq!(native.stdout, hit.stdout);
    assert_eq!(native.stderr, hit.stderr);
    assert_eq!(native.result_json, hit.result_json);
    assert_eq!(native.gcode, hit.gcode);
    fs::write(artifacts.join("native.stdout"), &native.stdout).unwrap();
    fs::write(artifacts.join("native.stderr"), &native.stderr).unwrap();
    fs::write(artifacts.join("native.result.json"), &native.result_json).unwrap();
    fs::copy(
        native.directory.join("receipt.json"),
        artifacts.join("native.receipt.json"),
    )
    .unwrap();
    if let Some(gcode) = &native.gcode {
        fs::write(artifacts.join("native.gcode"), gcode).unwrap();
    }
    let gcode = artifacts.join("plate_1.gcode");
    let output = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(&gcode)
        .arg(&project)
        .output()
        .unwrap();
    fs::write(artifacts.join("stdout"), &output.stdout).unwrap();
    fs::write(artifacts.join("stderr"), &output.stderr).unwrap();
    fs::write(
        artifacts.join("exit"),
        format!("{:?}\n", output.status.code()),
    )
    .unwrap();
    assert_eq!(
        output.status.code(),
        Some(expected_code),
        "{case}: {artifacts:?}"
    );
    assert_eq!(output.stderr, native.stderr);
    if expected_code == 193 {
        assert_eq!(output.stdout, native.stdout);
        assert_eq!(
            result_comparison_bytes(&fs::read(artifacts.join("result.json")).unwrap()),
            result_comparison_bytes(&native.result_json)
        );
        assert!(!gcode.exists(), "rejected before emission");
        assert!(native.gcode.is_none());
    } else {
        assert!(
            !fs::read(gcode).unwrap().is_empty(),
            "last object within printable_height must slice"
        );
        assert!(
            !native.gcode.unwrap().is_empty(),
            "Native last-object control must slice"
        );
    }
}

#[test]
fn sequential_clearance_reference_requires_native_not_frozen_channels() {
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "cli_sequential_clearance::cli_s4_rejects_horizontal_clearance_collision",
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
        "frozen sequential reference hid required Native absence: {diagnostic}"
    );
}

#[test]
fn cli_s4_rejects_horizontal_clearance_collision() {
    run("close", 193);
}
#[test]
fn cli_s4_rejects_tall_object_before_later_y_overlap() {
    run("rod-tall", 193);
}
#[test]
fn cli_s4_rejects_tall_object_before_later_disjoint_y() {
    run("lid-tall", 193);
}
#[test]
fn cli_s4_last_tall_object_uses_printable_height() {
    run("last-tall", 0);
}
