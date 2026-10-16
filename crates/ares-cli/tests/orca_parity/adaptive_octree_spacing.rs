//! FillAdaptive.cpp:314-354 object-level spacing and region-0 multiline.

use crate::{OrcaRunner, golden, runner::stages};
use sha2::{Digest, Sha256};
use std::{fs, process::Command};

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn run_case(name: &str, expected_sha: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let fixture = crate::runner::repo_root()
        .join("tests/parity/adaptive-octree-spacing")
        .join(format!("{name}.3mf"));
    let bytes = fs::read(fixture).unwrap();
    assert_eq!(sha(&bytes), expected_sha, "fixture digest {name}");
    let root = tempfile::Builder::new()
        .prefix("adaptive-octree-spacing-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    fs::write(root.join("input.3mf"), &bytes).unwrap();
    fs::create_dir(root.join("native")).unwrap();
    let args = vec![
        "--slice".into(),
        "0".into(),
        "--outputdir".into(),
        root.join("native").to_str().unwrap().into(),
        root.join("input.3mf").to_str().unwrap().into(),
    ];
    stages::run(runner.bin(), args, stages::Stage::Slice, &root).unwrap();
    let result = serde_json::from_slice::<serde_json::Value>(
        &fs::read(root.join("native/result.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(result["return_code"], 0, "Native admission {name} {root:?}");
    let native = fs::read(root.join("native/plate_1.gcode")).unwrap();
    let output = root.join("ares.gcode");
    let cli = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(&output)
        .arg(root.join("input.3mf"))
        .output()
        .unwrap();
    assert!(
        cli.status.success(),
        "CLI admission {name} {root:?}: {cli:?}"
    );
    let cli_bytes = fs::read(output).unwrap();
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project_plate_validated(
            &bytes,
            ares_core::GenerationMetadata::default(),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap_or_else(|error| panic!("public API admission {name} {root:?}: {error:?}"));
    golden::compare_ordered_bytes_generator_only(&native, &api)
        .unwrap_or_else(|error| panic!("API strict comparison {name} {root:?}: {error}"));
    golden::compare_ordered_bytes_generator_only(&native, &cli_bytes)
        .unwrap_or_else(|error| panic!("CLI strict comparison {name} {root:?}: {error}"));
    assert_eq!(
        api, cli_bytes,
        "public API/CLI raw equality {name} {root:?}"
    );
}

#[test]
fn adaptive_octree_spacing_single_region_control() {
    run_case(
        "control",
        "d6ea590827af8502bec7c2e2cc42139fbe879cb42e6949de954cde3430139ca9",
    );
}

#[test]
fn adaptive_octree_spacing_two_region_control() {
    run_case(
        "baseline",
        "d8194e73435fe1d29204fc0a9ac0f614c7152d65c75d6b7f697de4d64012a619",
    );
}

#[test]
fn adaptive_octree_spacing_density_red() {
    run_case(
        "density",
        "d64f5cbdf529de964b463c238f13939d200287ff4b909f906a935c972c3115be",
    );
}

#[test]
fn adaptive_octree_spacing_multiline_red() {
    run_case(
        "multiline",
        "70b0675b517a5bf6f27c2c7358b127c65a4d145e742eba6bf98b35a8cc0904d93",
    );
}
