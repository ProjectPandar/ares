//! Brim.cpp btPainted ears parity: painted.3mf carries
//! Metadata/brim_ear_points.txt (bbs_3mf.cpp:2962-3010/7583-7594)
//! consumed by make_brim_ears (Brim.cpp:354-412) via :537/:551.
use std::fs;

use serde_json::json;
use sha2::{Digest, Sha256};

use crate::runner::{OrcaRunner, stages};

const INPUT: &[u8] =
    include_bytes!("../../../../tests/parity/validation/brim_support_and_painted_ears/painted.3mf");
const INPUT_SHA256: &str = "b29dcedcd68bb564677a7f5f4b560f12c9b54039a6f8ba22eaeac09a75c203e7";

#[test]
fn brim_painted_ears_from_ear_points_matches_native() {
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
        .prefix("brim-painted-")
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
        "brim-painted",
        Ok(INPUT),
        Ok(&reference),
        json!({
            "kind":"brim_painted_ears_from_ear_points_native",
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
