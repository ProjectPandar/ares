//! `PrintObjectSlice.cpp:847` calls
//! `PrintObject::_transform_hole_to_polyholes()` (`PrintObject.cpp:189-295`)
//! inside `slice()` after `slice_volumes()` and before
//! `groupingVolumesForBrim`, converting almost-circular `expolygon.holes`
//! into `create_polyholes` output (`PrintObject.cpp:160-186`) when the
//! region config `hole_to_polyhole` (`PrintConfig.cpp:7093-7120`) is set.
//! Ares parses the keys (`region_fields.rs`) but has no consumer — this
//! RED freezes the Native whole output for the enabled/twisted positives
//! and the two non-converting controls.
//!
//! Fixture `tests/parity/hole-to-polyhole/block-hole.3mf`: 30x30x8 mm
//! block with a centered r=5 mm 64-gon hole (generated; see
//! `docs/architecture/hole-to-polyhole-spec.md`).

use super::*;
use crate::{golden, runner::stages};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Cursor, Read, Write},
    process::Command,
};

const BASE: &[u8] = include_bytes!("../../../../../tests/parity/hole-to-polyhole/block-hole.3mf");
/// Noisy 64-gon hole (vertex radius ±0.3 mm → ~0.6 mm vertex-spread):
/// misses the `5%` threshold (~0.25 mm bound, `Percent` branch of
/// `PrintObject.cpp:234`) but converts under `20%` (~1.0 mm bound).
const BASE_NOISY: &[u8] =
    include_bytes!("../../../../../tests/parity/hole-to-polyhole/block-hole-noisy.3mf");

/// `<part id="1">` metadata rows injected after `source_offset_z`.
struct Leg {
    label: &'static str,
    /// Fixture bytes this leg mutates and its Native baseline derives.
    fixture: &'static [u8],
    /// `(key, value)` pairs inserted into the part metadata.
    inject: &'static [(&'static str, &'static str)],
    /// Line that must appear in the Native output (polyhole vertex
    /// extrusion `G1 X130.802 Y125 E.05577`); `""` = leg is a control
    /// whose Native output must equal the plain fixture's.
    native_marker: &'static str,
}

const LEGS: [Leg; 6] = [
    Leg {
        label: "control",
        fixture: BASE,
        inject: &[],
        native_marker: "",
    },
    Leg {
        label: "enabled-twisted",
        fixture: BASE,
        inject: &[("hole_to_polyhole", "1")],
        native_marker: "G1 X130.802 Y125 E.05577",
    },
    Leg {
        label: "enabled-untwisted",
        fixture: BASE,
        inject: &[("hole_to_polyhole", "1"), ("hole_to_polyhole_twisted", "0")],
        native_marker: "G1 X130.802 Y125 E.05577",
    },
    Leg {
        label: "threshold-miss",
        fixture: BASE,
        inject: &[
            ("hole_to_polyhole", "1"),
            ("hole_to_polyhole_threshold", "0"),
        ],
        native_marker: "",
    },
    // Control on the noisy fixture: `hole_to_polyhole=1` + `5%` of
    // the ~5 mm mean radius (~0.25 mm bound) still misses the ~0.6 mm
    // vertex spread; Native output must equal the plain noisy fixture.
    // Discriminates the percent-branch unit bug: an implementation
    // that percent-scales the *coord-unit* mean radius accepts the
    // hole (astronomical bound) while upstream rejects it.
    Leg {
        label: "noisy-percent-miss",
        fixture: BASE_NOISY,
        inject: &[
            ("hole_to_polyhole", "1"),
            ("hole_to_polyhole_threshold", "5%"),
        ],
        native_marker: "",
    },
    // Positive on the noisy fixture: `20%` of ~5 mm (~1.0 mm bound)
    // beats the ~0.6 mm spread → upstream converts through the
    // `Percent` branch to a 16-gon polyhole. Marker is a polyhole
    // vertex absent from the control output.
    Leg {
        label: "noisy-percent-positive",
        fixture: BASE_NOISY,
        inject: &[
            ("hole_to_polyhole", "1"),
            ("hole_to_polyhole_threshold", "20%"),
        ],
        native_marker: "G1 X131.161 Y125 E.05388",
    },
];

/// Rewrites `Metadata/model_settings.config`, inserting the leg's
/// metadata after part 1's `source_offset_z` row. Verifies fixture shape.
fn project(leg: &Leg) -> Vec<u8> {
    let mut reader = zip::ZipArchive::new(Cursor::new(leg.fixture)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/model_settings.config" {
            let text = String::from_utf8(bytes).unwrap();
            assert_eq!(
                text.matches("<part ").count(),
                1,
                "fixture must keep one model part"
            );
            let anchor = "      <metadata key=\"source_offset_z\" value=\"0\"/>\n";
            assert!(text.contains(anchor), "part 1 anchor must exist");
            let insertion: String = leg
                .inject
                .iter()
                .map(|(key, value)| format!("      <metadata key=\"{key}\" value=\"{value}\"/>\n"))
                .collect();
            bytes = text
                .replacen(anchor, &format!("{anchor}{insertion}"), 1)
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
    assert!(!reference.is_empty());
    reference
}

/// Native-first assertions (spec condition 5): a positive leg must
/// contain the polyhole vertex line — proof upstream converted the
/// hole; a control leg must byte-match the unpatched fixture's Native
/// output modulo the `; generated by` header line.
fn assert_native(reference: &[u8], control_reference: &[u8], leg: &Leg) {
    let text = String::from_utf8(reference.to_vec()).unwrap();
    if leg.native_marker.is_empty() {
        assert_eq!(
            strip_generator(reference),
            strip_generator(control_reference),
            "{}: native control must be byte-equal to the plain fixture's output",
            leg.label
        );
        assert!(
            !text.contains("G1 X130.802 Y125 E.05577"),
            "{}: native control must not contain a polyhole vertex",
            leg.label
        );
    } else {
        assert!(
            text.contains(leg.native_marker),
            "{}: native output lacks polyhole vertex {}",
            leg.label,
            leg.native_marker
        );
    }
}

fn strip_generator(bytes: &[u8]) -> Vec<u8> {
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    text.lines()
        .filter(|line| !line.starts_with("; generated by"))
        .map(|line| format!("{line}\n"))
        .collect::<String>()
        .into_bytes()
}

fn run_leg(leg: &Leg, control_reference: &[u8]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix(&format!("hole-to-polyhole-{}-", leg.label))
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
    // raw API == CLI: when both produced bytes they must be identical.
    let raw_equals_cli = match (&public, fs::read(&output)) {
        (Ok(api), Ok(cli)) => api == &cli,
        _ => false,
    };
    fs::write(
        directory.join("comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "label":leg.label,
            "inject":leg.inject,
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
/// Native-first control/positive contract against a shared baseline.
fn control_native(fixture: &'static [u8]) -> Vec<u8> {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return Vec::new();
    };
    let root = tempfile::Builder::new()
        .prefix("hole-to-polyhole-baseline-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let baseline = Leg {
        label: "baseline",
        fixture,
        inject: &[],
        native_marker: "",
    };
    native(&runner, &root.join("baseline"), &project(&baseline))
}

/// Control: unpatched fixture — Native output is the baseline; Ares
/// must match byte-for-byte before and after the bounded GREEN.
#[test]
fn hole_to_polyhole_control() {
    let control = control_native(BASE);
    if control.is_empty() {
        return;
    }
    run_leg(&LEGS[0], &control);
}

/// Positive: `hole_to_polyhole=1` (twisted default 1) — Native emits a
/// 5-variant rotated polyhole; the vertex extrusion
/// `G1 X130.802 Y125 E.05577` must be present. Ares must reproduce the
/// whole Native output.
#[test]
fn hole_to_polyhole_enabled_twisted() {
    let control = control_native(BASE);
    if control.is_empty() {
        return;
    }
    run_leg(&LEGS[1], &control);
}

/// Positive: `hole_to_polyhole=1` + `hole_to_polyhole_twisted=0` —
/// Native emits a single untwisted polyhole (same vertex marker, no
/// per-layer rotation). Ares must reproduce the whole Native output.
#[test]
fn hole_to_polyhole_enabled_untwisted() {
    let control = control_native(BASE);
    if control.is_empty() {
        return;
    }
    run_leg(&LEGS[2], &control);
}

/// Control: `hole_to_polyhole=1` + `hole_to_polyhole_threshold=0` —
/// the variation bound collapses to `SCALED_EPSILON`, the 64-gon's
/// edge-midpoint spread (~0.009 mm) fails detection, and Native keeps
/// the circular hole; Native output must equal the plain control. Ares
/// must match byte-for-byte.
#[test]
fn hole_to_polyhole_threshold_miss() {
    let control = control_native(BASE);
    if control.is_empty() {
        return;
    }
    run_leg(&LEGS[3], &control);
}

/// Control on the noisy fixture: `hole_to_polyhole_threshold=5%` —
/// ~0.25 mm bound still misses the ~0.6 mm vertex spread, so Native
/// emits the circular hole; output must equal the unpatched noisy
/// fixture byte-for-byte. This leg discriminates a percent-branch unit
/// bug (percent of the *coord-unit* mean radius yields an astronomical
/// bound that wrongly converts the hole).
#[test]
fn hole_to_polyhole_noisy_percent_miss() {
    let control = control_native(BASE_NOISY);
    if control.is_empty() {
        return;
    }
    run_leg(&LEGS[4], &control);
}

/// Positive on the noisy fixture: `hole_to_polyhole_threshold=20%` —
/// ~1.0 mm bound beats the ~0.6 mm vertex spread; upstream converts
/// through the `Percent` branch of `get_abs_value`
/// (`PrintObject.cpp:234`, percent of the *unscaled* mean radius) to a
/// 16-gon polyhole. Marker `G1 X131.161 Y125 E.05388` is a polyhole
/// vertex absent from the control. Ares must reproduce the whole
/// Native output.
#[test]
fn hole_to_polyhole_noisy_percent() {
    let control = control_native(BASE_NOISY);
    if control.is_empty() {
        return;
    }
    run_leg(&LEGS[5], &control);
}
