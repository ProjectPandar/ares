//! The wasm slice adapter's legend Custom G-code overview of the sliced plate
//! (`GCodeViewer.cpp:4477-4546` over `get_curr_plate_custom_gcodes()`, the plate
//! selected as `OrcaSlicer.cpp:5690`) against the live Native oracle of
//! `tools/chain-probe/custom-gcode-overview.nix` for the same Ares G-code.
//! Failure inventory: the first plate's list for another plate; a plate key
//! off by one; the list of the default plate when none is given; the f32 time
//! changing in the JSON.
//! Public seams: `slice_project_plate_processed_bytes`, `customGcodeOverview`.

use std::{path::PathBuf, process::Command};

use ares_wasm::slice_project_plate_processed_bytes;

#[path = "../../../tests/support/native_environment.rs"]
mod native_environment;

const TEST: &str = "second_plate_rows_like_native";
const OVERVIEW: &str = "ARES_CUSTOM_GCODE_OVERVIEW_BIN";
const PROBE: &str = "ARES_PLATE_CUSTOM_GCODES_BIN";

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// `(label, printed layer, time f32 bits)`.
type Row = (String, String, u32);

/// The Ares rows and the Native rows for `plate` of `two-plates.3mf`; `key` is
/// the Native zero-based plate key.
fn rows(plate: Option<u32>, key: &str) -> Option<(Vec<Row>, Vec<Vec<String>>)> {
    let project = repo().join("tests/parity/custom-gcode-overview/two-plates.3mf");
    let mut output = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(slice_project_plate_processed_bytes(
            std::fs::read(&project).unwrap(),
            plate,
            None,
        ))
        .unwrap();
    let json: serde_json::Value =
        serde_json::from_str(&output.custom_gcode_overview().unwrap()).unwrap();
    let ares = json
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let time = row["time"].as_f64().unwrap();
            assert_eq!(f64::from(time as f32), time, "time is a widened f32");
            (
                row["label"].as_str().unwrap().to_owned(),
                row["layer"].as_i64().unwrap().to_string(),
                (time as f32).to_bits(),
            )
        })
        .collect();

    native_environment::binary(OVERVIEW)?;
    native_environment::binary(PROBE)?;
    let directory = tempfile::Builder::new()
        .prefix("custom-gcode-overview-wasm-")
        .tempdir()
        .unwrap()
        .keep();
    eprintln!("live custom-gcode-overview artifacts: {directory:?}");
    let gcode = directory.join("ares.gcode");
    std::fs::write(&gcode, output.take_gcode()).unwrap();
    let status = Command::new("node")
        .arg(repo().join("tools/chain-probe/custom-gcode-overview-fixtures.mjs"))
        .arg(directory.join("oracle"))
        .arg(&project)
        .arg(&gcode)
        .arg(key)
        .output()
        .unwrap();
    assert!(
        status.status.success(),
        "live custom-gcode-overview oracle failed: {} (artifacts {directory:?})",
        String::from_utf8_lossy(&status.stderr)
    );
    let native = std::fs::read_to_string(directory.join("oracle/rows.tsv"))
        .unwrap()
        .lines()
        .map(|line| line.split('\t').map(str::to_owned).collect())
        .collect();
    Some((ares, native))
}

fn expected(native: &[Vec<String>]) -> Vec<Row> {
    native
        .iter()
        .map(|r| (r[0].clone(), r[1].clone(), r[2].parse().unwrap()))
        .collect()
}

#[test]
fn required_missing_native_cannot_pass() {
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", TEST, "--nocapture"])
        .env("ARES_REQUIRE_ORCA", "1")
        .env_remove(OVERVIEW)
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "running 1 test"),
        "child selector did not run exactly one test: {output:?}"
    );
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("ARES_REQUIRE_ORCA=1: missing ARES_CUSTOM_GCODE_OVERVIEW_BIN"),
        "required failure did not identify the missing Native producer",
    );
}

#[test]
fn optional_missing_native_reports_skipped() {
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", TEST, "--nocapture"])
        .env_remove("ARES_REQUIRE_ORCA")
        .env_remove(OVERVIEW)
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "running 1 test"),
        "child selector did not run exactly one test: {output:?}"
    );
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("SKIPPED:"));
}

/// Plate 2 is the second plate (key 1); its list differs from plate 1's.
#[test]
fn second_plate_rows_like_native() {
    let Some((ares, native)) = rows(Some(2), "1") else {
        return;
    };
    assert_eq!(native.len(), 3, "two-plates: Native plate 2 list");
    assert_eq!(ares, expected(&native), "two-plates: plate 2 overview rows");
}

/// No plate slices the first plate (key 0), as the slice itself does.
#[test]
fn default_plate_rows_like_native() {
    let Some((ares, native)) = rows(None, "0") else {
        return;
    };
    assert_eq!(native.len(), 9, "two-plates: Native plate 1 list");
    assert_eq!(ares, expected(&native), "two-plates: plate 1 overview rows");
}
