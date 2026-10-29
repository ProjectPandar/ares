//! Brim.cpp btPainted inner call site: painted-inner-w5.3mf carries a
//! hole point (bbs_3mf.cpp:2962-3010/7583-7594) consumed by
//! make_brim_ears is_outer_brim=false (Brim.cpp:551-554), clipped into
//! the hole and against the support no-brim mask (:590-634).
use std::fs;

use serde_json::json;
use sha2::{Digest, Sha256};

use crate::runner::{OrcaRunner, stages};

const INPUT: &[u8] =
    include_bytes!("../../../../tests/parity/validation/w5-brim/painted-inner-w5.3mf");
const INPUT_SHA256: &str = "6adadabe73cd91d8c87ced07d952dd6bc5c8458e404cd1de43c80c8ad75b332e";

#[test]
fn brim_painted_inner_w5_matches_native() {
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
        .prefix("brim-painted-inner-w5-")
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
        "brim-painted-inner-w5",
        Ok(INPUT),
        Ok(&reference),
        json!({
            "kind":"brim_painted_inner_native",
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
