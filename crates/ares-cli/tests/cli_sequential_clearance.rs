//! Print.cpp:613-930,1298-1301 -> CLI_OBJECT_COLLISION_IN_SEQ_PRINT(-63).
#![cfg(target_os = "linux")]

use std::{fs, path::Path, process::Command};

fn run(case: &str, expected_code: i32) {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/parity/validation/sequential-clearance");
    let artifacts = tempfile::Builder::new()
        .prefix(&format!("cli-s4-{case}-"))
        .tempdir()
        .unwrap()
        .keep();
    let gcode = artifacts.join("plate_1.gcode");
    let output = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(&gcode)
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
    assert_eq!(
        output.status.code(),
        Some(expected_code),
        "{case}: {artifacts:?}"
    );
    assert_eq!(
        output.stderr,
        fs::read(fixtures.join(format!("{case}.stderr"))).unwrap()
    );
    if expected_code == 193 {
        assert_eq!(
            output.stdout,
            fs::read(fixtures.join(format!("{case}.stdout"))).unwrap()
        );
        assert_eq!(
            fs::read(artifacts.join("result.json")).unwrap(),
            fs::read(fixtures.join(format!("{case}.result.json"))).unwrap()
        );
        assert!(!gcode.exists(), "rejected before emission");
    } else {
        assert!(
            !fs::read(gcode).unwrap().is_empty(),
            "last object within printable_height must slice"
        );
    }
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
