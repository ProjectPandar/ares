//! Same-input control for the btPainted EFC snap: identical fixture with
//! brim_use_efc_outline=0 (Brim.cpp:58 gate closed), so neither engine
//! snaps the painted ear centre and the outputs stay byte-identical.
use std::fs;

use serde_json::json;
use sha2::{Digest, Sha256};

use crate::runner::{OrcaRunner, stages};

const INPUT: &[u8] =
    include_bytes!("../../../../tests/parity/validation/w5-brim/painted-efc-ctl-w5.3mf");
const INPUT_SHA256: &str = "03660965077394763c5841a1e3a78c9e3e11b2833fb902a5e55b20c0c62c837a";

#[test]
fn brim_painted_efc_control_no_snap_matches_native() {
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
        .prefix("brim-painted-efc-ctl-w5-")
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
            .contains(";TYPE:Brim\n"),
        "Native brim activation"
    );
    let outcome = crate::artifacts::compare(
        &root,
        "brim-painted-efc-ctl-w5",
        Ok(INPUT),
        Ok(&reference),
        json!({
            "kind":"brim_painted_efc_ctl_native",
            "input_sha256":hash,
            "native_command_directory":directory,
            "source":"Brim.cpp:56-62,354-412,537,551-554",
        }),
    )
    .unwrap();
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}
