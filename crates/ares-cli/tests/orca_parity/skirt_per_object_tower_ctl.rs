//! Same-input control: identical SEMM fixture with skirt_type=combined;
//! native and Ares stay byte-identical (raw normalized diff=0 verified
//! before the RED) and the control must PASS.
use std::fs;

use serde_json::json;
use sha2::{Digest, Sha256};

use crate::runner::{OrcaRunner, stages};

const INPUT: &[u8] =
    include_bytes!("../../../../tests/parity/validation/w5-skirt/per-object-tower-ctl-w5.3mf");
const INPUT_SHA256: &str = "85aabacdd30857f74fc65c6d9580d6b6fcad1423c18141a05b0b7ebdce91d3ed";

#[test]
fn skirt_per_object_tower_control_matches_native() {
    let hash = Sha256::digest(INPUT)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(hash, INPUT_SHA256);

    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix("skirt-per-object-tower-ctl-w5-")
        .tempdir_in(&root)
        .unwrap()
        .keep();
    let path = directory.join("input.3mf");
    let output = directory.join("orca");
    fs::write(&path, INPUT).unwrap();
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
    let reference = fs::read(output.join("plate_1.gcode")).unwrap();
    assert!(
        std::str::from_utf8(&reference)
            .unwrap()
            .contains(";TYPE:Skirt\n"),
        "Native skirt activation"
    );
    let outcome = crate::artifacts::compare(
        &root,
        "skirt-per-object-tower-ctl-w5",
        Ok(INPUT),
        Ok(&reference),
        json!({
            "kind":"skirt_per_object_tower_ctl_native",
            "input_sha256":hash,
            "native_command_directory":directory,
            "source":"Print.cpp:2784-2996,2821-2823,2865-2899;GCode.cpp:4465-4485",
        }),
    )
    .unwrap();
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}
