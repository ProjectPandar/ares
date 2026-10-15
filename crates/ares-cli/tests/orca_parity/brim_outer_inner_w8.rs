//! Brim.cpp explicit-width OuterAndInner single-object parity; approved
//! SPEC c3c28ab6 (support no-brim is brim_type-independent, Brim.cpp:590-634).
use std::fs;

use serde_json::json;
use sha2::{Digest, Sha256};

use crate::runner::{OrcaRunner, stages};

const INPUT: &[u8] =
    include_bytes!("../../../../tests/parity/validation/w5-brim/outer-inner-w8.3mf");
const INPUT_SHA256: &str = "70e2ec7f72543a24bdee55e86d196aa4799b1e68dee1fe2addda01c9589ddda1";

#[test]
fn brim_outer_inner_width8_matches_native() {
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
        .prefix("brim-outerinner-w8-")
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
        "brim-outerinner-w8",
        Ok(INPUT),
        Ok(&reference),
        json!({
            "kind":"brim_outer_inner_w8_native",
            "input_sha256":hash,
            "native_command_directory":directory,
            "source":"Brim.cpp:459-460,527-556,590-634,682-689",
        }),
    )
    .unwrap();
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}
