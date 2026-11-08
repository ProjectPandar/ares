//! FillAdaptive.cpp:304-323 per-printing-region spacing and multiline.

#[path = "adaptive_endpoint_touch.rs"]
mod adaptive_endpoint_touch;

use crate::{
    OrcaRunner, golden,
    runner::{native_stream_case, stages},
};
use sha2::{Digest, Sha256};
use std::fs;

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn run_parity_case(name: &str, expected_sha: &str) {
    native_stream_case::check(
        (
            "adaptive-supportcubic-offset",
            &[
                "Fill/FillAdaptive.cpp:641-647",
                "Line.cpp:121-126",
                "Line.hpp:123-148",
            ],
        ),
        name,
        include_bytes!(
            "../../../../tests/parity/adaptive-supportcubic-offset/mk4-supportcubic-d40.3mf"
        ),
        expected_sha,
        (false, |_, _| {}),
    );
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
    let cli = crate::runner::application::cli_command()
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
        .block_on(crate::runner::application::slice_project_plate_validated(
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
        "b2f1aba2bb357fb165c025770560e09d8d752bee57b6ad8f53ad9135763b93fd",
    );
}

#[test]
fn adaptive_octree_spacing_two_region_control() {
    run_case(
        "baseline",
        "4a57f8ccf5df885d4dee61292136cc892113e5cc323ccddfd2c672ca47d78527",
    );
}

#[test]
fn adaptive_octree_spacing_density_red() {
    run_case(
        "density",
        "f95bfc0f5106b37f815f53bf4ca07d3ea555fbab6f33352ff349f11f2c9a2f9b",
    );
}

#[test]
fn adaptive_octree_spacing_multiline_red() {
    run_case(
        "multiline",
        "7050c7d1916fa42d9a2ea6f9e8990f4aef447530423a4c175e11c3aef440240d",
    );
}

#[test]
fn adaptive_supportcubic_offset_mk4_d40() {
    run_parity_case(
        "mk4-supportcubic-d40",
        "a6ee6f899f0f640ff4aaec35b12175cc59f29aab6ef7e26fed22a1a7e803095e",
    );
}
