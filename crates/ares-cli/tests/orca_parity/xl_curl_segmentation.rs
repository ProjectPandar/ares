//! XL curl-annotation segmentation witness (tests-only RED for
//! `docs/superpowers/specs/2026-10-25-xl-curl-annotation-segmentation.md`).
//!
//! Stock Prusa XL 0.4 nozzle defaults + the `hole-to-polyhole` block mesh
//! (29.55 mm outer-wall edges). Upstream `estimate_malformations` runs
//! `estimate_points_properties<true, true, false, false>` with
//! `max_line_length = params.bridge_distance = 16.0`
//! (`SupportSpotsGenerator.cpp:170`, `SupportSpotsGenerator.hpp:38`), so
//! walls longer than 16 mm get annotation points at `j·t`,
//! `t = 16/len` (`ExtrusionProcessor.hpp:189-215`), and the windowed
//! curvature runs on the annotated sequence (`:219-278`). Ares
//! `CurlTracker::collect` (`overhang/curl.rs:148`) scores curvature on the
//! raw entity points: the 29.55 mm square segments pair two corner
//! curvatures, clear the 0.1 curl tolerance on the layer above the
//! elephant-foot-compensated first layer, and the next layer's artificial
//! curl distance forces 169/167/162 mm/s where the oracle stays at 170
//! (`G1 F10140` insert + 4 processed-quantized E lines at layer 3 — the
//! only two hunks in the whole file).
//!
//! The curl-off control (same project, `slowdown_for_curled_perimeters=0`)
//! and the Prusa MINI control (same mesh, stock MINI defaults whose
//! parameterization does not reach the curl path) must both pass: the
//! witness failure is attributable to the curl chain alone.

use crate::{OrcaRunner, golden, runner::stages};
use sha2::{Digest, Sha256};
use std::fs;

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn run_case(name: &str, expected_sha: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}")) else {
        return;
    };
    let fixture = crate::runner::repo_root()
        .join("tests/parity/xl-curl-segmentation")
        .join(format!("{name}.3mf"));
    let bytes = fs::read(&fixture).unwrap();
    assert_eq!(sha(&bytes), expected_sha, "fixture digest {name}");
    let root = tempfile::Builder::new()
        .prefix("xl-curl-segmentation-")
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
fn xl_curl_segmentation_witness_red() {
    run_case(
        "xl-witness",
        "e4ef60761982e53bff9525ae4aaf7fe5955d5ed2b4e5b8753561b08cbde9c842",
    );
}

#[test]
fn xl_curl_off_control() {
    run_case(
        "xl-curl-off",
        "063105d8c4375d70bbab5b8310239e202030b60763c85eec57306b1c7e36489a",
    );
}

#[test]
fn xl_curl_mini_control() {
    run_case(
        "mini-control",
        "4d283b4608f843740c02e0094540b18b5561efbcf69c6605c4b71b873747a099",
    );
}
