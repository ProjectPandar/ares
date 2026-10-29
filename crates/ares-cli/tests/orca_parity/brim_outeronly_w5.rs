//! Brim.cpp explicit-width OuterOnly single-object parity; approved SPEC
//! 8d8813365 (RED scope: byte-parity leg; GREEN requires the boundary
//! SPEC amendment naming the one upstream function being ported).
use std::fs;

use serde_json::json;
use sha2::{Digest, Sha256};

use crate::runner::{OrcaRunner, stages};

const INPUT: &[u8] =
    include_bytes!("../../../../tests/parity/validation/w5-brim/outer-only-w5.3mf");
const INPUT_SHA256: &str = "a4a661702406d16509ed439e704661cc8cc0e8f0bcc39695ed86218a2f26c9bc";

#[test]
fn brim_outer_only_width5_matches_native() {
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
        .prefix("brim-outeronly-w5-")
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
    let text = std::str::from_utf8(&reference).unwrap();
    assert!(text.contains(";TYPE:Brim\n"), "Native brim activation");
    let outcome = crate::artifacts::compare(
        &root,
        "brim-outeronly-w5",
        Ok(INPUT),
        Ok(&reference),
        json!({
            "kind":"brim_outeronly_w5_native",
            "input_sha256":hash,
            "native_command_directory":directory,
            "source":"Brim.cpp:421-725,813-877,890-975",
        }),
    )
    .unwrap();
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}
