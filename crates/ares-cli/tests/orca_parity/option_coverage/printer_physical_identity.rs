//! PrintConfig.cpp:9004-9055,9634ff; GCode.cpp:2833-2847,4677-4687.
//! Failure inventory: IDs normalized away, wrong variant slot, physical/filament
//! maps conflated, inactive templates, stale materialization, API/CLI mismatch.
use crate::{
    golden,
    runner::{OrcaRunner, stages},
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, path::Path, process::Command};

#[path = "printer_physical_identity/input.rs"]
mod input;

const COMPLETE: &[&str] = &["1", "1", "2", "2"];
const REVERSED: &[&str] = &["2", "2", "1", "1"];

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn native(runner: &OrcaRunner, directory: &Path, project: &[u8]) -> Vec<u8> {
    fs::create_dir(directory).unwrap();
    let path = directory.join("input.3mf");
    fs::write(&path, project).unwrap();
    let output = directory.join("native");
    fs::create_dir(&output).unwrap();
    let args = [
        "--slice".into(),
        "0".into(),
        "--outputdir".into(),
        output.to_str().unwrap().into(),
        path.to_str().unwrap().into(),
    ];
    let producer = std::env::var_os("ARES_ORCA_NATIVE_BIN").unwrap();
    fs::write(
        directory.join("native.launch.json"),
        serde_json::to_vec_pretty(&json!({
            "input_sha256":sha(project),"native_binary":producer,
            "native_sha256":sha(&fs::read(&producer).unwrap()),
            "wrapper":runner.bin(),"args":args,
            "seed":std::env::var("ORCA_REPLAY_SEED").ok(),
            "cpu":std::env::var("ORCA_REPLAY_CPU").ok()
        }))
        .unwrap(),
    )
    .unwrap();
    stages::run(runner.bin(), args, stages::Stage::Slice, directory).unwrap();
    let result = fs::read(output.join("result.json")).unwrap();
    let metadata: serde_json::Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(metadata["return_code"], 0, "{directory:?}");
    let reference = fs::read(output.join("plate_1.gcode")).unwrap();
    assert!(!reference.is_empty());
    reference
}

fn extrusion_lines(bytes: &[u8]) -> Vec<&str> {
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .filter(|line| (line.starts_with("G0 ") || line.starts_with("G1 ")) && line.contains(" E"))
        .collect()
}

fn compare(label: &str, ids: &[&str], physical: &[&str]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    compare_with(&runner, label, ids, physical);
}

fn compare_with(runner: &OrcaRunner, label: &str, ids: &[&str], physical: &[&str]) {
    let root = tempfile::Builder::new()
        .prefix(&format!("printer-physical-{label}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let directory = root.join("case");
    let project = input::project(ids, physical);
    let reference = native(runner, &directory, &project);
    let text = std::str::from_utf8(&reference).unwrap();
    let start = if physical == ["0", "1"] {
        "; IDENTITY_START first0=0 first1=1"
    } else {
        "; IDENTITY_START first0=1 first1=0"
    };
    let layer = if physical == ["0", "1"] {
        "; IDENTITY_LAYER most_physical=1"
    } else {
        "; IDENTITY_LAYER most_physical=0"
    };
    assert_eq!(
        text.lines()
            .filter(|line| line.starts_with("; IDENTITY_START"))
            .collect::<Vec<_>>(),
        vec![start]
    );
    let layers = text
        .lines()
        .filter(|line| line.starts_with("; IDENTITY_LAYER"))
        .collect::<Vec<_>>();
    assert_eq!(layers, vec![layer; 50]);
    for marker in ["; G4_ACTIVE F0 N1", "; G4_ACTIVE F1 N2"] {
        assert!(
            text.lines().any(|line| line == marker),
            "{root:?}: {marker}"
        );
    }
    if ids == REVERSED {
        let baseline = native(
            runner,
            &root.join("baseline"),
            &input::project(COMPLETE, physical),
        );
        assert_ne!(
            extrusion_lines(&reference),
            extrusion_lines(&baseline),
            "printer ID variant selection must change Native E-motion, not only header text"
        );
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let public = runtime.block_on(crate::runner::application::slice_project_plate_validated(
        &project,
        ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    ));
    if let Ok(gcode) = &public {
        fs::write(directory.join("ares.validated.gcode"), gcode).unwrap();
    }
    let api_comparison = public
        .as_ref()
        .map_err(|error| format!("validated API: {error:?}"))
        .and_then(|gcode| golden::compare_ordered_bytes_generator_only(&reference, gcode));
    let output_dir = directory.join("ares");
    fs::create_dir(&output_dir).unwrap();
    let output = output_dir.join("plate_1.gcode");
    let binary = env!("CARGO_BIN_EXE_ares");
    let cli = Command::new(binary)
        .args(["slice", "-o"])
        .arg(&output)
        .arg(directory.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(directory.join("ares.stdout"), &cli.stdout).unwrap();
    fs::write(directory.join("ares.stderr"), &cli.stderr).unwrap();
    let cli_comparison = fs::read(&output)
        .map_err(|error| format!("CLI output: {error}"))
        .and_then(|gcode| golden::compare_ordered_bytes_generator_only(&reference, &gcode));
    fs::write(
        directory.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "label":label,"requested_ids":ids,"requested_physical_map":physical,
            "input_sha256":sha(&project),"native_sha256":sha(&reference),
            "validated_api_comparison":api_comparison,"cli_comparison":cli_comparison,
            "CLI_code":cli.status.code(),"CLI_binary":binary,
            "CLI_binary_sha256":sha(&fs::read(binary).unwrap()),
            "native_start_marker":start,"native_layer_markers":layers,
            "api_error":public.as_ref().err().map(|error| format!("{error:?}"))
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(cli.status.success(), "{root:?}: {cli:?}");
    api_comparison.unwrap_or_else(|error| panic!("{root:?}: {error}"));
    cli_comparison.unwrap_or_else(|error| panic!("{root:?}: {error}"));
}

#[test]
fn printer_physical_identity_p0_complete() {
    compare("P0", COMPLETE, &["0", "1"]);
}
#[test]
fn printer_physical_identity_p1_reversed() {
    compare("P1", REVERSED, &["0", "1"]);
}
#[test]
fn printer_physical_identity_p2_generated() {
    compare("P2", &["1", "2"], &["0", "1"]);
}
#[test]
fn printer_physical_identity_m0_physical_reverse() {
    compare("M0", COMPLETE, &["1", "0"]);
}
#[test]
fn printer_physical_identity_m1_independent() {
    compare("M1", REVERSED, &["1", "0"]);
}
