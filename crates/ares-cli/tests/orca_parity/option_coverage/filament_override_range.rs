//! `DynamicPrintConfig::validate` (`PrintConfig.cpp:8866-8884`) builds a
//! `FullPrintConfig` and forwards to `Slic3r::validate(fpc)`
//! (`PrintConfig.cpp:10337-10565`), whose numeric range loop iterates
//! `fpc.keys()`. `FullPrintConfig` composes `PrintObjectConfig +
//! PrintRegionConfig + PrintConfig` (`PrintConfig.hpp:1660-1665`), and
//! `PrintConfig` derives `(MachineEnvelopeConfig, GCodeConfig)`
//! (`:1480-1481`) — so static `GCodeConfig` `filament_*` members are
//! range-checked, but the 16 dynamic `filament_extruder_override_keys`
//! created with `add_nullable` (`PrintConfig.cpp:63-84`, `:7288-7320`)
//! are not static members and escape the check.
//!
//! Native accepts `filament_retraction_distances_when_cut=5` despite
//! the declared `[10,18]` metadata (`PrintConfig.cpp:5225-5226`), while
//! rejecting `filament_max_volumetric_speed=-1` (`min=0`,
//! `PrintConfig.cpp:2550-2558`). Verified on official 2.4.2.
//!
//! Fixture `tests/parity/filament-override-range/base.3mf`: Ender-3
//! single-extruder project. Legs inject the key into
//! `Metadata/project_settings.config`.
//!
//! RED: `target-5` must FAIL pre-fix (`5 not in range [10,18]`) and
//! slice byte-equal post-fix. `control-15` (in-range) passes both.
//! `reject-mvs` proves the check still applies to static `GCodeConfig`
//! filament members — Native rejects `-1` so Ares must reject it too.

use super::*;
use crate::{golden, runner::stages};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Cursor, Read, Write},
    process::Command,
};

const BASE: &[u8] = include_bytes!("../../../../../tests/parity/filament-override-range/base.3mf");

/// `"filament_deretraction_speed": ["nil"]` is the insertion anchor for
/// the dynamic override key (present verbatim in the fixture).
const ANCHOR: &str = "\"filament_deretraction_speed\":";

struct Leg {
    label: &'static str,
    /// Complete `"key": [..]` JSON fragment injected after `ANCHOR`.
    fragment: &'static str,
    /// Exact substring the Native rejection stderr must carry; `None`
    /// when the leg slices successfully.
    native_reject: Option<&'static str>,
}

const LEGS: [Leg; 3] = [
    Leg {
        label: "target-5",
        fragment: "\"filament_retraction_distances_when_cut\": [\n    \"5\",\n    \"5\"\n  ],",
        native_reject: None,
    },
    Leg {
        label: "control-15",
        fragment: "\"filament_retraction_distances_when_cut\": [\n    \"15\",\n    \"15\"\n  ],",
        native_reject: None,
    },
    Leg {
        label: "reject-mvs",
        fragment: "\"filament_max_volumetric_speed\": [\n    \"-1\",\n    \"-1\"\n  ]",
        native_reject: Some("not in range"),
    },
];

/// Rewrites `Metadata/project_settings.config`: for `reject-mvs`
/// replaces the existing `filament_max_volumetric_speed` array body;
/// for the other legs injects `fragment` immediately before `ANCHOR`.
/// The fixture's JSON is pretty-printed (one array element per line),
/// so fragments are emitted in the same style.
fn project(leg: &Leg) -> Vec<u8> {
    let mut reader = zip::ZipArchive::new(Cursor::new(BASE)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut text = String::from_utf8(bytes).unwrap();
            assert!(text.contains(ANCHOR), "fixture must carry {ANCHOR}");
            if leg.label == "reject-mvs" {
                let existing = "\"filament_max_volumetric_speed\": [\n    \"12\",\n    \"12\"\n  ]";
                assert!(text.contains(existing), "fixture must carry {existing}");
                text = text.replacen(existing, leg.fragment, 1);
            } else {
                text = text.replacen(ANCHOR, &format!("{}\n  {ANCHOR}", leg.fragment), 1);
            }
            bytes = text.into_bytes();
        }
        writer
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::default().compression_method(file.compression()),
            )
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// `--slice 0 --outputdir` Native reference; requires exit0 + nonempty
/// gcode.
fn native(runner: &OrcaRunner, directory: &std::path::Path, project: &[u8]) -> Vec<u8> {
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
    stages::run(runner.bin(), args, stages::Stage::Slice, directory).unwrap();
    let result = fs::read(output.join("result.json")).unwrap();
    let metadata: serde_json::Value = serde_json::from_slice(&result).unwrap();
    assert_eq!(metadata["return_code"], 0, "{directory:?}");
    let reference = fs::read(output.join("plate_1.gcode")).unwrap();
    assert!(
        !reference.is_empty(),
        "{directory:?}: native reference must be non-empty"
    );
    reference
}

/// Runs the Native slicer expecting a validation rejection; returns the
/// captured stderr for marker assertions.
fn native_expect_reject(
    runner: &OrcaRunner,
    directory: &std::path::Path,
    project: &[u8],
) -> String {
    fs::create_dir_all(directory).unwrap();
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
    let error = stages::run(runner.bin(), args, stages::Stage::Slice, directory)
        .expect_err("native must reject this project");
    String::from_utf8_lossy(
        &error
            .captured
            .as_ref()
            .map(|captured| captured.stderr.clone())
            .unwrap_or_default(),
    )
    .into_owned()
}

fn run_leg(leg: &Leg) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix(&format!("filament-override-range-{}-", leg.label))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let directory = root.join("case");
    let project = project(leg);

    if let Some(marker) = leg.native_reject {
        let stderr = native_expect_reject(&runner, &directory.join("native-check"), &project);
        assert!(
            stderr.contains(marker),
            "{:?}: native rejection must carry `{marker}`, got {stderr:?}",
            directory
        );
        // Ares must reject the same input with the same range error.
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let error = runtime
            .block_on(ares_core::slice_project_plate_validated(
                &project,
                ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
                None,
                &ares_core::ConfigOverrides::default(),
                ares_core::SlicerContext::new(0),
            ))
            .expect_err("Ares must reject a value Native rejects");
        assert!(
            format!("{error:?}").contains("not in range"),
            "{:?}: Ares rejection must carry `not in range`, got {error:?}",
            directory
        );
        fs::write(
            directory.join("comparison.json"),
            serde_json::to_vec_pretty(&json!({
                "label": leg.label,
                "native_rejection_stderr": stderr,
                "ares_error": format!("{error:?}"),
            }))
            .unwrap(),
        )
        .unwrap();
        return;
    }

    let reference = native(&runner, &directory, &project);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let public = runtime.block_on(ares_core::slice_project_plate_validated(
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
    let raw_equals_cli = match (&public, fs::read(&output)) {
        (Ok(api), Ok(cli)) => api == &cli,
        _ => false,
    };
    fs::write(
        directory.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "label":leg.label,
            "input_sha256":sha(&project),"native_sha256":sha(&reference),
            "validated_api_comparison":api_comparison,"cli_comparison":cli_comparison,
            "raw_api_equals_cli":raw_equals_cli,
            "CLI_code":cli.status.code(),
            "api_error":public.as_ref().err().map(|error| format!("{error:?}"))
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(cli.status.success(), "{root:?}: {cli:?}");
    api_comparison.unwrap_or_else(|error| panic!("{root:?}: {error}"));
    cli_comparison.unwrap_or_else(|error| panic!("{root:?}: {error}"));
    assert!(raw_equals_cli, "{root:?}: raw API must equal CLI output");
}

/// Target: `filament_retraction_distances_when_cut=["5"]` — a dynamic
/// `filament_extruder_override_keys` member (`PrintConfig.cpp:82-83`);
/// Native accepts despite the `[10,18]` metadata because `fpc.apply`
/// drops the dynamic key before the range loop. Ares must slice and
/// match byte-for-byte. FAILS pre-fix (`5 not in range [10,18]`).
#[test]
fn filament_override_range_target_5() {
    run_leg(&LEGS[0]);
}

/// Control: `=["15"]` — in-range value on the same dynamic key; both
/// Native and Ares accept, output must be byte-equal pre- and post-fix.
#[test]
fn filament_override_range_control_15() {
    run_leg(&LEGS[1]);
}

/// `--overrides` leg (review follow-up): the public override boundary
/// (`overrides.rs::validate_entry` → `validate_numeric_range`) ports
/// `ConfigBase::set_deserialize` (`Config.cpp:591-620`), which never
/// applies min/max — upstream's only range gate is `Slic3r::validate`
/// where `fpc.apply` already dropped dynamic keys. Native reference is
/// the same effective config via `project_settings` (target-5 project);
/// Ares applies `{"filament_retraction_distances_when_cut":["5","5"]}`
/// over the unpatched fixture and must produce identical bytes.
/// FAILS pre-fix (`5 not in range [10,18]` at the boundary).
#[test]
fn filament_override_range_overrides_cli_5() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("filament-override-range-overrides-5-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let directory = root.join("case");
    // Same effective config, through the project_settings path.
    let reference = native(&runner, &directory, &project(&LEGS[0]));

    let output_dir = directory.join("ares");
    fs::create_dir(&output_dir).unwrap();
    let output = output_dir.join("plate_1.gcode");
    let binary = env!("CARGO_BIN_EXE_ares");
    let overrides = "{\"filament_retraction_distances_when_cut\":[\"5\",\"5\"]}";
    let input = directory.join("unpatched.3mf");
    fs::write(&input, BASE).unwrap();
    let cli = Command::new(binary)
        .args(["slice", "--overrides", overrides, "-o"])
        .arg(&output)
        .arg(&input)
        .output()
        .unwrap();
    fs::write(directory.join("ares.stdout"), &cli.stdout).unwrap();
    fs::write(directory.join("ares.stderr"), &cli.stderr).unwrap();
    let comparison = fs::read(&output)
        .map_err(|error| format!("CLI output: {error}"))
        .and_then(|gcode| golden::compare_ordered_bytes_generator_only(&reference, &gcode));
    fs::write(
        directory.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "label":"overrides-5","native_sha256":sha(&reference),
            "cli_comparison":comparison,"CLI_code":cli.status.code(),
            "cli_stderr":String::from_utf8_lossy(&cli.stderr)
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(cli.status.success(), "{root:?}: {cli:?}");
    comparison.unwrap_or_else(|error| panic!("{root:?}: {error}"));

    // Static `GCodeConfig` filament members stay range-checked through the
    // same boundary (`filament_max_volumetric_speed=-1` → reject).
    let reject = Command::new(binary)
        .args([
            "slice",
            "--overrides",
            "{\"filament_max_volumetric_speed\":[\"-1\"]}",
            "-o",
        ])
        .arg(&output)
        .arg(&input)
        .output()
        .unwrap();
    assert!(
        !reject.status.success()
            && String::from_utf8_lossy(&reject.stderr).contains("not in range"),
        "{root:?}: static filament_* --overrides must still reject, got {reject:?}"
    );
}

/// Rejection control: `filament_max_volumetric_speed=["-1"]` — a static
/// `GCodeConfig` member with `min=0` (`PrintConfig.cpp:2550-2558`).
/// Native rejects with `not in range`; Ares must reject identically
/// both pre- and post-fix — the skip must NOT widen to all `filament_*`.
#[test]
fn filament_override_range_reject_mvs() {
    run_leg(&LEGS[2]);
}
