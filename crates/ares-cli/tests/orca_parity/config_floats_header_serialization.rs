//! Config.hpp `ConfigOptionFloats::serialize` (ConfigOptionVector
//! serialization: per-element `%g`, comma-joined) reached through the
//! GCode.cpp full-config block for the two registry Floats keys without
//! a typed export representation: `material_correction` and
//! `relative_correction`. At load time Ares decodes their preset string
//! into the raw vector list
//! (`options/json_deserialize.rs:76 normalize_scalar_vector_strings`)
//! and the raw echo re-renders that list with the generic `;` join
//! (`project/raw_settings.rs:211 RawConfigValue::token` List arm →
//! `render_list_token`, join(";")) — Native re-serializes both keys
//! comma-joined and value-normalized (JSON array `["1e-1","0.50"]` →
//! `0.1,0.5`, `["2.0","2.0"]` → `2,2`), so the RED must FAIL on the
//! strict whole-output compare (first differing line: `; material_correction = 0.1,0.5`
//! vs `; material_correction = 1e-1;0.50`). The control fixture holds
//! the typed `retraction_speed=["2.0","2.0"]` (→ `2,2` both sides)
//! and must PASS. Vector preset values are frozen in the 3MF's native
//! JSON-array form; scalar-string vectors and nullable `nil`-element
//! vectors serialize at parity through the slicer but the parity
//! harness's auxiliary typed re-parse cannot load their string form,
//! so those stay out of this chain's frozen legs.
//! The GREEN fixes the single raw-token rendering source (the
//! `option_token`/`token` list join for Floats kinds) to the upstream
//! comma separator; no writer conversion and no per-key branches.
use std::fs;

use serde_json::json;
use sha2::{Digest, Sha256};

use crate::runner::{OrcaRunner, stages};

const INPUT: &[u8] = include_bytes!("../../../../tests/parity/config-floats-header/class.3mf");
const INPUT_SHA256: &str = "8ce99e92b1991787ed60f0ddcbb0c77977336c3d773c3d42d74bdc35a7f6e666";
const CONTROL: &[u8] = include_bytes!("../../../../tests/parity/config-floats-header/control.3mf");
const CONTROL_SHA256: &str = "b219dadfad135233899ef00518a65a5f43c0b91bdce6b7053d168c3b81a015ba";

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
fn config_floats_header_serialization_control_matches_native() {
    assert_eq!(sha256(CONTROL), CONTROL_SHA256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let reference = slice_native(&runner, &root, CONTROL, "config-floats-header-ctl-");
    let outcome = crate::artifacts::compare(
        &root,
        "config-floats-header-control",
        Ok(CONTROL),
        Ok(reference.as_bytes()),
        json!({
            "kind":"config_floats_header_control",
            "input_sha256":CONTROL_SHA256,
            "source":"Config.hpp ConfigOptionFloats::serialize;GCode.cpp config block",
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
fn config_floats_header_serialization_class_matches_native() {
    let hash = sha256(INPUT);
    assert_eq!(hash, INPUT_SHA256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let reference = slice_native(&runner, &root, INPUT, "config-floats-header-class-");
    // The class must be ACTIVE and normalized by Native: elements
    // serialize via %g and the vector joins with commas.
    for expected in [
        "; material_correction = 0.1,0.5",
        "; relative_correction = 2,2",
    ] {
        assert!(
            reference.lines().any(|line| line == expected),
            "Native serializes {expected} via ConfigOptionFloats::serialize"
        );
    }
    let outcome = crate::artifacts::compare(
        &root,
        "config-floats-header-class",
        Ok(INPUT),
        Ok(reference.as_bytes()),
        json!({
            "kind":"config_floats_header_class",
            "input_sha256":hash,
            "source":"Config.hpp ConfigOptionFloats::serialize;GCode.cpp config block",
        }),
    )
    .unwrap();
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}
