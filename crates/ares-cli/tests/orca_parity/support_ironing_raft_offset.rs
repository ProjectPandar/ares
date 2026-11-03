//! Support ironing with raft layers, owned by
//! `SupportCommon.cpp:1877-1908` and
//! `FillRectilinear.cpp:1548-1551`.
//!
//! The RED target keeps the Native-generated G-code live: the target is
//! expected to fail until Ares preserves the target contour point when a
//! non-Valid vertical link starts a replacement polyline. The no-raft fixture
//! is a strict control.

use crate::{OrcaRunner, golden, runner::stages};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;

const RAFT: &[u8] =
    include_bytes!("../../../../tests/parity/support-ironing-raft-offset/raft-two-layers.3mf");
const NO_RAFT: &[u8] =
    include_bytes!("../../../../tests/parity/support-ironing/overhang-organic.3mf");

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn strict_case(label: &str, input: &[u8], expected_sha: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    assert_eq!(sha(input), expected_sha, "frozen {label} input changed");
    let root = tempfile::Builder::new()
        .prefix(&format!("support-ironing-raft-{label}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let directory = root.join("case");
    fs::create_dir(&directory).unwrap();
    fs::write(directory.join("input.3mf"), input).unwrap();
    let native_dir = directory.join("native");
    fs::create_dir(&native_dir).unwrap();
    stages::run(
        runner.bin(),
        vec![
            "--slice".into(),
            "0".into(),
            "--outputdir".into(),
            native_dir.to_str().unwrap().into(),
            directory.join("input.3mf").to_str().unwrap().into(),
        ],
        stages::Stage::Slice,
        &directory,
    )
    .unwrap();
    let result: Value =
        serde_json::from_slice(&fs::read(native_dir.join("result.json")).unwrap()).unwrap();
    assert_eq!(result["return_code"], 0, "Native {label} did not complete");
    let native = fs::read(native_dir.join("plate_1.gcode")).unwrap();
    assert!(
        String::from_utf8_lossy(&native).contains(";TYPE:Ironing\n"),
        "Native {label} must emit support ironing"
    );

    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let api = runtime
        .block_on(crate::runner::application::slice_project_plate_validated(
            input,
            ares_core::GenerationMetadata::deterministic(2026, 10, 17, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap_or_else(|error| panic!("Ares API {label}: {error:?}"));
    let cli_path = directory.join("ares.cli.gcode");
    let cli = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(&cli_path)
        .arg(directory.join("input.3mf"))
        .output()
        .unwrap();
    assert!(cli.status.success(), "Ares CLI {label}: {cli:?}");
    let cli_bytes = fs::read(cli_path).unwrap();
    golden::compare_ordered_bytes_generator_only(&native, &api)
        .unwrap_or_else(|error| panic!("Ares API strict {label}: {error}"));
    golden::compare_ordered_bytes_generator_only(&native, &cli_bytes)
        .unwrap_or_else(|error| panic!("Ares CLI strict {label}: {error}"));
    assert_eq!(api, cli_bytes, "Ares API/CLI raw output {label}");
}

#[test]
fn support_ironing_raft_offset_native_strict_red() {
    strict_case(
        "raft-two-layers",
        RAFT,
        "e361fd095b95efe469220c3927d209538abdfaee1d206220c7d836c86ef78934",
    );
}

#[test]
fn support_ironing_raft_offset_no_raft_control() {
    strict_case(
        "no-raft-control",
        NO_RAFT,
        "9df149d8bda0fea5211def5c5057dfc00d7df1a439d99706b1e8a2b21d648010",
    );
}
