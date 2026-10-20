//! Endpoint-contact simplification through public Native/API/CLI streams.

use crate::{OrcaRunner, golden, runner::stages};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, path::Path, process::Command};

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn beam_disabled_height025() {
    check(
        "height025-beam0",
        include_bytes!("../../../../tests/parity/endpoint-contact/height025-beam0.3mf"),
        "b1c6d193f0b990473cc857fdbde50a4ddfc7a739aff30bf3ce3810529a0e9529",
        true,
    );
}

#[test]
fn original_odd_kernel_residual() {
    check(
        "odd-kernel-beam1",
        include_bytes!("../../../../tests/parity/endpoint-contact/odd-kernel-beam1.3mf"),
        "180d44d516f560510bb19a6584e6f41cd092cb16788a6a4091d7a32c69d841b7",
        true,
    );
}

#[test]
fn single_volume_control() {
    check(
        "single",
        include_bytes!("../../../../tests/parity/endpoint-contact/single.3mf"),
        "1ac6d319469fbee245bdc9b44b1600980b5777e62a0e84f34fac0f798dc85651",
        false,
    );
}

fn check(label: &str, project: &[u8], expected_sha: &str, assert_two_tools: bool) {
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    assert_eq!(sha(project), expected_sha, "frozen input SHA for {label}");
    let root = tempfile::Builder::new()
        .prefix(&format!("endpoint-contact-{label}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input = root.join("input.3mf");
    fs::write(&input, project).unwrap();
    let reference = native_slice(&runner, &root, &input, label, "native");
    let replay = native_slice(&runner, &root, &input, label, "native-replay");
    let first_native =
        golden::normalize_one_generator_line(&reference, golden::GeneratorKind::Orca).unwrap();
    let replay_native =
        golden::normalize_one_generator_line(&replay, golden::GeneratorKind::Orca).unwrap();
    let native_replay = golden::first_difference(&first_native, &replay_native)
        .map_or(Ok(()), |difference| Err(difference.to_string()));
    native_replay
        .as_ref()
        .unwrap_or_else(|error| panic!("Native nondeterminism {label}: {error}; {root:?}"));
    let native = std::str::from_utf8(&reference).unwrap();
    if assert_two_tools {
        assert!(
            native.lines().any(|line| line.starts_with("T0")),
            "Native target must use T0: {root:?}"
        );
        assert!(
            native.lines().any(|line| line.starts_with("T1")),
            "Native target must use T1: {root:?}"
        );
    }
    fs::write(root.join("native.gcode"), &reference).unwrap();
    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project_plate_validated(
            project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(root.join("ares.api.gcode"), &api).unwrap();
    let output = root.join("ares.cli.gcode");
    let cli = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(&output)
        .arg(&input)
        .output()
        .unwrap();
    fs::write(root.join("ares.cli.stdout"), &cli.stdout).unwrap();
    fs::write(root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    assert!(cli.status.success(), "Ares CLI {label}: {root:?}");
    let cli_bytes = fs::read(&output).unwrap();
    assert_eq!(
        api, cli_bytes,
        "Ares API/CLI raw equality {label}: {root:?}"
    );
    let api_comparison = golden::compare_ordered_bytes_generator_only(&reference, &api);
    let cli_comparison = golden::compare_ordered_bytes_generator_only(&reference, &cli_bytes);
    fs::write(
        root.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "case": label, "input_sha256": sha(project), "native_sha256": sha(&reference),
            "native_replay_comparison": native_replay,
            "api_comparison": api_comparison, "cli_comparison": cli_comparison,
            "api_cli_equal": api == cli_bytes,
            "strict_comparator": "golden::compare_ordered_bytes_generator_only"
        }))
        .unwrap(),
    )
    .unwrap();
    api_comparison.unwrap_or_else(|error| panic!("{label}: {error}; artifacts {root:?}"));
    cli_comparison.unwrap_or_else(|error| panic!("{label}: {error}; artifacts {root:?}"));
}

fn native_slice(
    runner: &OrcaRunner,
    root: &Path,
    input: &Path,
    label: &str,
    phase: &str,
) -> Vec<u8> {
    let native_dir = root.join(phase);
    fs::create_dir(&native_dir).unwrap();
    let args = vec![
        "--slice".into(),
        "0".into(),
        "--outputdir".into(),
        native_dir.to_string_lossy().into_owned(),
        input.to_string_lossy().into_owned(),
    ];
    fs::write(root.join(format!("{phase}-launch.json")), serde_json::to_vec_pretty(&json!({
        "case": label, "input_sha256": sha(&fs::read(input).unwrap()), "wrapper": runner.bin(), "args": args,
        "upstream": ["GCode/AvoidCrossingPerimeters.cpp:FirstIntersectionVisitor/simplify_travel", "Geometry.hpp:segments_intersect"]
    })).unwrap()).unwrap();
    stages::run(runner.bin(), args, stages::Stage::Slice, &native_dir).unwrap();
    fs::read(native_dir.join("plate_1.gcode")).unwrap()
}
