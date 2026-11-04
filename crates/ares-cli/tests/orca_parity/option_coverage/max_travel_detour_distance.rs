//! `AvoidCrossingPerimeters::travel_to`
//! (`AvoidCrossingPerimeters.cpp:1291-1302`) caps the detour a routed
//! `reduce_crossing_wall` travel may take: when
//! `detour = routed_length - direct_length` exceeds
//! `percent ? direct*0.01*value : scale_(value)` the route collapses to
//! the straight `{start, end}` and `could_be_wipe_disabled` is forced
//! false. The option `max_travel_detour_distance`
//! (`PrintConfig.hpp:1489`, `coFloatOrPercent`, default `"0"`) is
//! parsed into `ProcessPrintSourceOptions` (`print_source.rs`) but
//! Ares' avoid-crossing router (`motion/path/avoid_crossing/*`) has no
//! consumer — every capped leg must diverge from Ares output while
//! matching Native exactly.
//!
//! The second upstream cap site (`AvoidCrossingPerimeters.cpp:1576`)
//! sits inside a `#if 0` block (:1340-:1729) — dead code, deferred as
//! untriggerable.
//!
//! Fixture `tests/parity/max-travel-detour/detour-3parts.3mf`: three
//! parts — two 15x15x8 blocks flanking a 2x90x8 wall — with
//! `reduce_crossing_wall=1` baked into `project_settings.config`.
//! Travels between the blocks detour around the wall boundary, so
//! small caps revert them to straight lines.
//!
//! Legs (targets: `1`mm, `2%`, `10%`; controls: `0` default and
//! `200`mm — the 200mm cap never engages): Native-first — each target
//! output carries its marker; controls must byte-match the unpatched
//! fixture modulo the generator/config lines; strict whole-output
//! comparison, API == CLI, both modes.

use super::*;
use crate::{golden, runner::stages};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Cursor, Read, Write},
    process::Command,
};

const BASE: &[u8] =
    include_bytes!("../../../../../tests/parity/max-travel-detour/detour-3parts.3mf");

struct Leg {
    label: &'static str,
    /// Replacement for `"max_travel_detour_distance": "0"` inside
    /// `Metadata/project_settings.config`.
    value: &'static str,
    /// Line that must appear in the Native output; `""` = control leg
    /// whose Native output must equal the unpatched fixture's.
    native_marker: &'static str,
}

const LEGS: [Leg; 5] = [
    Leg {
        label: "control-default",
        value: "0",
        native_marker: "",
    },
    Leg {
        label: "control-200mm",
        value: "200",
        native_marker: "",
    },
    // cap=1mm: the longest wall detours exceed 1mm and collapse to
    // straight lines; `G1 X73.613 Y73.694 Z7` is one such reverted
    // travel (Z-hop + single waypoint, absent in the uncapped output).
    Leg {
        label: "cap-1mm",
        value: "1",
        native_marker: "G1 X73.613 Y73.694 Z7",
    },
    // cap=2% (~0.9mm of direct length): essentially all detours revert;
    // `G1 X74.203 Y72.607 Z1.4` is unique to this output.
    Leg {
        label: "cap-2pct",
        value: "2%",
        native_marker: "G1 X74.203 Y72.607 Z1.4",
    },
    // cap=10% (~4.4mm): a partial revert set; same Z7-hop marker as
    // cap-1mm but the leg must still match the *whole* Native output.
    Leg {
        label: "cap-10pct",
        value: "10%",
        native_marker: "G1 X73.613 Y73.694 Z7",
    },
];

/// Rewrites `Metadata/project_settings.config`, replacing the leg's
/// `"max_travel_detour_distance"` JSON value. Verifies fixture shape.
fn project(leg: &Leg) -> Vec<u8> {
    let mut reader = zip::ZipArchive::new(Cursor::new(BASE)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let text = String::from_utf8(bytes).unwrap();
            let anchor = "\"max_travel_detour_distance\": \"0\"";
            assert!(
                text.contains(anchor),
                "project settings must carry cap=0 default"
            );
            bytes = text
                .replacen(
                    anchor,
                    &format!("\"max_travel_detour_distance\": \"{}\"", leg.value),
                    1,
                )
                .into_bytes();
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

/// `--slice 0 --outputdir` Native reference; requires exit0 + nonempty gcode.
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

/// Native-first assertions: a target leg must contain its marker —
/// proof upstream applied the cap; a control leg must byte-match the
/// unpatched fixture's Native output modulo the `; generated by` and
/// `; max_travel_detour_distance` lines (the option key is echoed into
/// the config block, so the injected value itself differs).
fn assert_native(reference: &[u8], control_reference: &[u8], leg: &Leg) {
    let text = String::from_utf8(reference.to_vec()).unwrap();
    if leg.native_marker.is_empty() {
        assert_eq!(
            strip_volatile(reference),
            strip_volatile(control_reference),
            "{}: native control must be byte-equal to the plain fixture's output",
            leg.label
        );
    } else {
        assert!(
            text.contains(leg.native_marker),
            "{}: native output lacks marker {}",
            leg.label,
            leg.native_marker
        );
    }
}

fn strip_volatile(bytes: &[u8]) -> Vec<u8> {
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    text.lines()
        .filter(|line| {
            !line.starts_with("; generated by") && !line.starts_with("; max_travel_detour_distance")
        })
        .map(|line| format!("{line}\n"))
        .collect::<String>()
        .into_bytes()
}

fn run_leg(leg: &Leg, control_reference: &[u8]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix(&format!("max-travel-detour-{}-", leg.label))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let directory = root.join("case");
    let project = project(leg);
    let reference = native(&runner, &directory, &project);
    assert_native(&reference, control_reference, leg);

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
    let raw_equals_cli = match (&public, fs::read(&output)) {
        (Ok(api), Ok(cli)) => api == &cli,
        _ => false,
    };
    fs::write(
        directory.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "label":leg.label,
            "value":leg.value,
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

/// Runs the unpatched fixture once so each leg can assert its
/// Native-first control/target contract against a shared baseline.
/// `None` only when the Orca runner is unavailable.
fn control_native() -> Option<Vec<u8>> {
    let runner = OrcaRunner::from_env().unwrap()?;
    let root = tempfile::Builder::new()
        .prefix("max-travel-detour-baseline-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    Some(native(&runner, &root.join("baseline"), &project(&LEGS[0])))
}

/// Control: unpatched fixture (`reduce_crossing_wall=1`, cap `0`) —
/// Native output is the baseline; Ares must match byte-for-byte
/// (uncapped path is already parity-correct).
#[test]
fn max_travel_detour_control_default() {
    let Some(control) = control_native() else {
        return;
    };
    run_leg(&LEGS[0], &control);
}

/// Control: `cap=200`mm — upstream computes the cap but every detour
/// is shorter; output must equal the control baseline byte-for-byte.
#[test]
fn max_travel_detour_control_200mm() {
    let Some(control) = control_native() else {
        return;
    };
    run_leg(&LEGS[1], &control);
}

/// Target: `cap=1`mm — the long wall detours collapse to straight
/// travels (`G1 X73.613 Y73.694 Z7`). Ares must reproduce the whole
/// Native output.
#[test]
fn max_travel_detour_cap_1mm() {
    let Some(control) = control_native() else {
        return;
    };
    run_leg(&LEGS[2], &control);
}

/// Target: `cap=2%` — the Percent branch
/// (`direct_length * 0.01 * value`, `AvoidCrossingPerimeters.cpp:1296`)
/// reverts nearly all detours; marker `G1 X74.203 Y72.607 Z1.4`.
#[test]
fn max_travel_detour_cap_2pct() {
    let Some(control) = control_native() else {
        return;
    };
    run_leg(&LEGS[3], &control);
}

/// Target: `cap=10%` — partial revert set through the same Percent
/// branch; marker `G1 X73.613 Y73.694 Z7`.
#[test]
fn max_travel_detour_cap_10pct() {
    let Some(control) = control_native() else {
        return;
    };
    run_leg(&LEGS[4], &control);
}
