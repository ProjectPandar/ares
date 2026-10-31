//! Same-input control: the R1 base without flush_into_* — no wiping
//! pass, so the change-layer retract lands correctly and native/Ares
//! stay byte-identical; must PASS (proves the divergence needs the
//! wipe override).
use std::fs;

use serde_json::json;
use sha2::{Digest, Sha256};

use crate::runner::{OrcaRunner, stages};

const INPUT: &[u8] = include_bytes!(
    "../../../../tests/parity/validation/skirt_wipe_tower_flush/toolchange-wipe-control.3mf"
);
const INPUT_SHA256: &str = "85aabacdd30857f74fc65c6d9580d6b6fcad1423c18141a05b0b7ebdce91d3ed";

#[test]
fn toolchange_wipe_control_matches_native() {
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
        .prefix("toolchange-wipe-control-")
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
        "Native tool-change wipe activation"
    );
    let outcome = crate::artifacts::compare(
        &root,
        "toolchange-wipe-control",
        Ok(INPUT),
        Ok(&reference),
        json!({
            "kind":"skirt_per_object_tower_native",
            "input_sha256":hash,
            "native_command_directory":directory,
            "source":"GCode.cpp:4638-4688,5692-5697,7652-7670",
        }),
    )
    .unwrap();
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}
