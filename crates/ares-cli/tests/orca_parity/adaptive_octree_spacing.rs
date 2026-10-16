//! FillAdaptive.cpp:314-354 object-level spacing and region-0 multiline.

use crate::{golden, runner::stages, OrcaRunner};
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
    assert!(cli.status.success(), "CLI admission {name} {root:?}: {cli:?}");
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
    assert_eq!(api, cli_bytes, "public API/CLI raw equality {name} {root:?}");
}

#[test]
fn adaptive_octree_spacing_single_region_control() {
    run_case(
        "control",
        "415e618032003375cc36ce51f36c8b6bdf50d55544cc84b9f08da05459ee8d5f",
    );
}

#[test]
fn adaptive_octree_spacing_two_region_control() {
    run_case(
        "baseline",
        "aaebe0fc41772e80571913f1271696c9f078cafa9cdfd29cc074a1cef53d868c",
    );
}

#[test]
fn adaptive_octree_spacing_density_red() {
    run_case(
        "density",
        "94cd069c6f422caeaa0b64f9dbb1c899aaf89cf481f003531e966a35afbac5e0",
    );
}

#[test]
fn adaptive_octree_spacing_multiline_red() {
    run_case(
        "multiline",
        "edfb949d54e162bd52eeefa539b74065435e7661378b9d84371385a36293efe3",
    );
}
