//! Base InterlockingGenerator public Native/API/CLI contracts.

use crate::{OrcaRunner, golden, runner::stages};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, path::Path, process::Command};

const TARGET: &[u8] =
    include_bytes!("../../../../tests/parity/interlocking-base/two-extruder-beam0.3mf");
const BEAM_TARGET: &[u8] =
    include_bytes!("../../../../tests/parity/interlocking-base/two-extruder-beam1.3mf");
const CONTROL: &[u8] = include_bytes!("../../../../tests/parity/interlocking-base/single.3mf");
const TARGET_SHA: &str = "1dbedfe681a6efdd527c4f032ddfe26cfb9746e817711a5d59bc4889e5751486";
const BEAM_TARGET_SHA: &str = "04c7d2a58b7b21909b867542faad149a6b9d0b1bb976eab3fec57b7b60ff082c";
const CONTROL_SHA: &str = "1ac6d319469fbee245bdc9b44b1600980b5777e62a0e84f34fac0f798dc85651";
const BOUNDARY_ZERO: &[u8] =
    include_bytes!("../../../../tests/parity/interlocking-base/boundary-zero-beam1.3mf");
const THREE_OFF: &[u8] =
    include_bytes!("../../../../tests/parity/interlocking-base/three-region-beam0.3mf");
const SAME_OFF: &[u8] =
    include_bytes!("../../../../tests/parity/interlocking-base/same-extruder-beam0.3mf");
const SAME_ON: &[u8] =
    include_bytes!("../../../../tests/parity/interlocking-base/same-extruder-beam1.3mf");

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn two_extruder_beam0_materializes_all_volume_regions() {
    check("two-extruder-beam0", TARGET, TARGET_SHA, true);
}

#[test]
fn two_extruder_beam1_interlocking_matches_native() {
    check("two-extruder-beam1", BEAM_TARGET, BEAM_TARGET_SHA, true);
}

#[test]
fn single_volume_region_order_control_matches_native() {
    check("single-volume-control", CONTROL, CONTROL_SHA, false);
}

#[test]
fn zero_boundary_avoidance_matches_native() {
    check(
        "boundary-zero-beam1",
        BOUNDARY_ZERO,
        "2ec7ccd398bbf1ef2c629ca6fc5292aeeb09f3fb8e78c74fdb905fb680d26235",
        true,
    );
}

#[test]
fn three_regions_beam0_materialization_control_matches_native() {
    check(
        "three-region-beam0",
        THREE_OFF,
        "44cc50f8113599e99bcfc3095b8cb9ad2b3513c4b70448c6f79d7f3a37a68578",
        true,
    );
}

#[test]
fn same_extruder_beam0_control_matches_native() {
    check(
        "same-extruder-beam0",
        SAME_OFF,
        "02f02488d175f6f82157bc88d81ac2a5c953fc8db12928331dd4e0579208a1a5",
        false,
    );
}

#[test]
fn same_extruder_beam1_is_native_noop() {
    check(
        "same-extruder-beam1",
        SAME_ON,
        "f10cd5d45afb5b66a46af9e9293e35a5891991a8846a338b1bd733211eee0570",
        false,
    );
}

fn check(label: &str, project: &[u8], expected_sha: &str, assert_two_tools: bool) {
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    assert_eq!(sha(project), expected_sha, "frozen input SHA for {label}");
    let root = tempfile::Builder::new()
        .prefix(&format!("interlocking-base-{label}-"))
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
        "upstream": ["PrintObjectSlice.cpp:slices_to_regions", "Feature/Interlocking/InterlockingGenerator.cpp", "Feature/Interlocking/VoxelUtils.cpp", "PrintRegion.cpp:flow"]
    })).unwrap()).unwrap();
    stages::run(runner.bin(), args, stages::Stage::Slice, &native_dir).unwrap();
    fs::read(native_dir.join("plate_1.gcode")).unwrap()
}

fn executable(bytes: &[u8]) -> &str {
    let text = std::str::from_utf8(bytes).unwrap();
    &text[text.find("; EXECUTABLE_BLOCK_START").unwrap()
        ..text.find("; EXECUTABLE_BLOCK_END").unwrap()]
}

#[test]
fn native_beam_switch_is_observable_only_across_extruders() {
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let root = tempfile::Builder::new()
        .prefix("interlocking-base-native-admission-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    for (label, off, on, changes) in [
        ("original", TARGET, BEAM_TARGET, true),
        ("same-extruder", SAME_OFF, SAME_ON, false),
    ] {
        let mut references = Vec::new();
        for (phase, project) in [("off", off), ("on", on)] {
            let case = root.join(format!("{label}-{phase}"));
            fs::create_dir(&case).unwrap();
            let input = case.join("input.3mf");
            fs::write(&input, project).unwrap();
            references.push(native_slice(&runner, &case, &input, label, "native"));
        }
        let differs = executable(&references[0]) != executable(&references[1]);
        fs::write(root.join(format!("{label}-discrimination.json")), serde_json::to_vec_pretty(&json!({"expected_body_change": changes, "observed_body_change": differs, "off_sha256": sha(references[0].as_slice()), "on_sha256": sha(references[1].as_slice())})).unwrap()).unwrap();
        assert_eq!(differs, changes, "Native beam admission {label}: {root:?}");
    }
}
