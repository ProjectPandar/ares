//! Model.cpp bed polygon and Brim.cpp final printable-region subtraction.
use std::fs;

use serde_json::json;
use sha2::{Digest, Sha256};

use crate::runner::{OrcaRunner, stages};

#[path = "inner_only_brim.rs"]
mod inner_only;

#[path = "outer_inner_brim.rs"]
mod outer_inner;

#[test]
fn brim_bed_region_kobra3_02_outer_matches_native() {
    verify(
        "kobra3-0.2-outer",
        include_bytes!("../../../../tests/parity/validation/kobra3-centered/0.2.3mf"),
        "d63ac67f57d8cfcc36d4c8cbf62fb11ad436f7893454c8bba2760f52ec61c51c",
    );
}

#[test]
fn brim_bed_region_kobra3_04_outer_matches_native() {
    verify(
        "kobra3-0.4-outer",
        include_bytes!("../../../../tests/parity/validation/kobra3-centered/0.4-outer-only.3mf"),
        "7a6e8d8fb0eef2a5467f23ea979b9e9822c42b97b8cc97743ac29cb29080c49c",
    );
}

#[test]
fn brim_bed_region_kobra3_06_outer_matches_native() {
    verify(
        "kobra3-0.6-outer",
        include_bytes!("../../../../tests/parity/validation/kobra3-centered/0.6.3mf"),
        "9bcf3a21d416f94a707eba82149fba6eb58cd815918a6369c98d5e345e3a3c06",
    );
}

#[test]
fn brim_bed_region_kobra3_08_outer_matches_native() {
    verify(
        "kobra3-0.8-outer",
        include_bytes!("../../../../tests/parity/validation/kobra3-centered/0.8.3mf"),
        "0dc37a07daedf1aa01a48832a2ff3b4e70f6e367075274391e42884a98db1b0c",
    );
}

fn verify(label: &str, input: &[u8], hash: &str) {
    let actual_hash = Sha256::digest(input)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(actual_hash, hash);
    compare_native(label, input, true);
}

pub(super) fn compare_native(label: &str, input: &[u8], expect_brim: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let hash = Sha256::digest(input)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let root = crate::artifacts::root_from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix("brim-bed-native-")
        .tempdir_in(&root)
        .unwrap()
        .keep();
    let path = directory.join("input.3mf");
    let output = directory.join("orca");
    fs::write(&path, input).unwrap();
    fs::create_dir(&output).unwrap();
    stages::run(
        runner.bin(),
        [
            "--slice",
            "0",
            "--outputdir",
            output.to_str().unwrap(),
            path.to_str().unwrap(),
        ]
        .into_iter()
        .map(str::to_owned),
        stages::Stage::Slice,
        &directory,
    )
    .unwrap();
    let reference = fs::read(output.join("plate_1.gcode")).unwrap();
    let has_brim = reference
        .windows(b";TYPE:Brim\n".len())
        .any(|w| w == b";TYPE:Brim\n");
    assert_eq!(has_brim, expect_brim, "{label}: Native brim activation");
    let outcome = crate::artifacts::compare(
        &root,
        label,
        Ok(input),
        Ok(&reference),
        json!({
            "kind":"fresh_brim_bed_native",
            "input_sha256":hash,
            "native_command_directory":directory,
            "source":"Model.cpp:3014-3026; Brim.cpp:636-686",
        }),
    )
    .unwrap();
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}
