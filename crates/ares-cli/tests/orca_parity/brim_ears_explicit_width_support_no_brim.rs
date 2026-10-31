//! Brim.cpp explicit-width BrimEars ears+support single-object parity; approved
//! SPEC 7f47a247c (BrimEars arm support no-brim, Brim.cpp:590-634).
use std::fs;

use serde_json::json;
use sha2::{Digest, Sha256};

use crate::runner::{OrcaRunner, stages};

const INPUT: &[u8] =
    include_bytes!("../../../../tests/parity/validation/brim_support_and_painted_ears/ears.3mf");
const INPUT_SHA256: &str = "e940f42d061f2d4b920b1e34407b91e862b8bd6a4b332d89586652b7e508f5e5";

#[test]
fn brim_ears_explicit_width_support_no_brim_matches_native() {
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
        .prefix("brim-ears-")
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
        "brim-ears",
        Ok(INPUT),
        Ok(&reference),
        json!({
            "kind":"brim_ears_explicit_width_support_no_brim_native",
            "input_sha256":hash,
            "native_command_directory":directory,
            "source":"Brim.cpp:590-634,682-689",
        }),
    )
    .unwrap();
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}
