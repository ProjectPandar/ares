//! GCode.cpp:4638-4688/5692-5697/7652-7670 tool-change layer-boundary
//! retract+wipe timing: the SEMM fixture (R1 base, combined skirt,
//! flush_into_infill=1) leaves the change-layer retract to the first
//! post-purge travel instead of the layer head — the RED must FAIL
//! until the boundary flush arms from config state instead of the
//! positioned-gated pending flag the tool transition clears.
use std::fs;

use serde_json::json;
use sha2::{Digest, Sha256};

use crate::runner::{OrcaRunner, stages};

const INPUT: &[u8] =
    include_bytes!("../../../../tests/parity/validation/w5-skirt/toolchange-wipe-flush-w5.3mf");
const INPUT_SHA256: &str = "aedd1cd9b356e6693299aba0cbff4ab4a55434dc7429947b74f56c2ade258e9d";

#[test]
fn toolchange_wipe_flush_w5_matches_native() {
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
        .prefix("toolchange-wipe-flush-w5-")
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
        "toolchange-wipe-flush-w5",
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
