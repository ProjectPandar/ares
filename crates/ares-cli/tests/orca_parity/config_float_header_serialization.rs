//! Config.cpp `ConfigOptionFloat::serialize` (decimal-point `%g`,
//! integral values lose the fraction) reached through the GCode.cpp
//! full-config block: every key the project presets define is loaded
//! into the upstream config object and re-serialized, never echoed
//! verbatim. The Ares raw-settings echo loop
//! (`options/config_export/writer.rs`) instead copies preset tokens
//! verbatim for keys without a typed export representation, so the
//! 49 registry Float keys that upstream normalizes (SLA material/
//! pad/hollowing, SLA pillar supports, absolute/relative correction,
//! area_fill, bottle_*, display_*, elefant_foot_min_width,
//! gamma_correction, exposure/tilt times) keep their input spelling
//! `2.0` while Native prints `2`. The class fixture sets the whole
//! class to the integer-valued float `2.0` and must FAIL on the
//! strict whole-output compare (first differing lines: the class
//! keys in the trailing config dump, e.g. `; absolute_correction =
//! 2` vs `; absolute_correction = 2.0`); the control fixture holds
//! only typed keys (`top_shell_thickness`, `bridge_speed`) at the
//! same integer-valued spelling and must PASS — the GREEN retires
//! the verbatim echo for registry Float kinds with the single typed
//! `format_number` serialization source, no per-key special cases.
use std::fs;

use serde_json::json;
use sha2::{Digest, Sha256};

use crate::runner::{OrcaRunner, stages};

const INPUT: &[u8] = include_bytes!("../../../../tests/parity/config-float-header/class.3mf");
const INPUT_SHA256: &str = "39130052643315104ed0927c40548420b69a0e010dbc109273b59ceac931be66";
const CONTROL: &[u8] = include_bytes!("../../../../tests/parity/config-float-header/control.3mf");
const CONTROL_SHA256: &str = "bbf8b207f759444a85c220903de1c49e61a54e71034b605f5b723a0fc3accf37";

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn slice_native(runner: &OrcaRunner, root: &std::path::Path, input: &[u8], prefix: &str) -> String {
    let directory = tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in(root)
        .unwrap()
        .keep();
    let path = directory.join("input.3mf");
    let output = directory.join("orca");
    fs::write(&path, input).unwrap();
    fs::create_dir(&output).unwrap();
    let args = [
        "--slice".to_owned(),
        "0".to_owned(),
        "--outputdir".to_owned(),
        output.to_str().unwrap().to_owned(),
        path.to_str().unwrap().to_owned(),
    ];
    stages::run(
        runner.bin(),
        args.iter().cloned(),
        stages::Stage::Slice,
        &directory,
    )
    .unwrap();
    fs::read_to_string(output.join("plate_1.gcode")).unwrap()
}

#[test]
fn config_float_header_serialization_control_matches_native() {
    assert_eq!(sha256(CONTROL), CONTROL_SHA256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let reference = slice_native(&runner, &root, CONTROL, "config-float-header-control-");
    let outcome = crate::artifacts::compare(
        &root,
        "config-float-header-control",
        Ok(CONTROL),
        Ok(reference.as_bytes()),
        json!({
            "kind":"config_float_header_control",
            "input_sha256":CONTROL_SHA256,
            "source":"Config.cpp ConfigOptionFloat::serialize;GCode.cpp config block",
        }),
    )
    .unwrap();
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}

#[test]
fn config_float_header_serialization_class_matches_native() {
    let hash = sha256(INPUT);
    assert_eq!(hash, INPUT_SHA256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let reference = slice_native(&runner, &root, INPUT, "config-float-header-class-");
    // The class must be ACTIVE and normalized by Native: integral
    // floats serialize without the fraction.
    for key in ["absolute_correction", "elefant_foot_min_width"] {
        let expected = format!("; {key} = 2");
        assert!(
            reference.lines().any(|line| line == expected),
            "Native serializes {key} via ConfigOptionFloat::serialize"
        );
    }
    let outcome = crate::artifacts::compare(
        &root,
        "config-float-header-class",
        Ok(INPUT),
        Ok(reference.as_bytes()),
        json!({
            "kind":"config_float_header_class",
            "input_sha256":hash,
            "source":"Config.cpp ConfigOptionFloat::serialize;GCode.cpp config block",
        }),
    )
    .unwrap();
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}
