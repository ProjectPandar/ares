//! Same-input control for the btPainted inner call site: no
//! brim_ear_points.txt, so make_brim_ears returns empty (Brim.cpp:359-361)
//! and neither engine emits a brim.
use std::fs;

use serde_json::json;
use sha2::{Digest, Sha256};

use crate::runner::{OrcaRunner, stages};

const INPUT: &[u8] = include_bytes!(
    "../../../../tests/parity/validation/brim_support_and_painted_ears/painted-inner-control.3mf"
);
const INPUT_SHA256: &str = "bdb02a48d4da6c24f8a622836356499347d1166dee0611a265925ce787801bdd";

#[test]
fn brim_painted_inner_control_no_points_matches_native() {
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
        .prefix("brim-painted-inner-control-")
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
    // Same-input control: no brim_ear_points.txt, so both engines emit
    // no brim (make_brim_ears early return, Brim.cpp:359-361).
    assert!(
        !std::str::from_utf8(&reference)
            .unwrap()
            .contains(";TYPE:Brim\n")
    );
    let outcome = crate::artifacts::compare(
        &root,
        "brim-painted-inner-control",
        Ok(INPUT),
        Ok(&reference),
        json!({
            "kind":"brim_painted_inner_control_native",
            "input_sha256":hash,
            "native_command_directory":directory,
            "source":"Brim.cpp:354-412,457-458,551-554,590-634",
        }),
    )
    .unwrap();
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}
