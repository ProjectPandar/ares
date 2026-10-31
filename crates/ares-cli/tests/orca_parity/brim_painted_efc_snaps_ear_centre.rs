//! Brim.cpp btPainted EFC snap: painted-efc.3mf enables
//! brim_use_efc_outline with elefant_foot_compensation 0.1 / 1 layer /
//! no raft (Brim.cpp:56-62), so make_brim_ears snaps the painted ear
//! centre to the closest vertex of the EFC-adjusted bottom outline
//! (:392-411 via closest_point_on_matching_island :99-108); the skirt
//! follows the changed brim hull.
use std::fs;

use serde_json::json;
use sha2::{Digest, Sha256};

use crate::runner::{OrcaRunner, stages};

const INPUT: &[u8] = include_bytes!(
    "../../../../tests/parity/validation/brim_support_and_painted_ears/painted-efc.3mf"
);
const INPUT_SHA256: &str = "c92ffdef749533af50bdb063c9d7d838f8802460f4564260f5985297c9750a68";

#[test]
fn brim_painted_efc_snaps_ear_centre_matches_native() {
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
        .prefix("brim-painted-efc-")
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
        "brim-painted-efc",
        Ok(INPUT),
        Ok(&reference),
        json!({
            "kind":"brim_painted_efc_native",
            "input_sha256":hash,
            "native_command_directory":directory,
            "source":"Brim.cpp:56-62,354-412,392-411,99-108,112-118",
        }),
    )
    .unwrap();
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}
