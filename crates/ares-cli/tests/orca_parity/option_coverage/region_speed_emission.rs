//! `GCode.cpp:6131-6145,6149-6175` (`extrude_perimeters`/`extrude_infill`)
//! apply the *current* region config via
//! `m_config.apply(region.config())` at `:6136`/`:6163` before emitting
//! that region's entities; the `:6516-6545` feedrate switch reads it. Ares
//! refreshes only `outer_wall_speed`/`small_perimeter_speed` per region,
//! so per-part `inner_wall_speed`/`sparse_infill_speed` overrides are
//! dropped: this RED freezes the two-region Native whole output.
//!
//! Fixture `tests/parity/interface-shells/two-region-prism.3mf`: one
//! object, two `normal_part` volumes sharing layers below z=6 (see
//! `docs/architecture/region-speed-emission-discovery.md`).

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
    include_bytes!("../../../../../tests/parity/interface-shells/two-region-prism.3mf");

/// `;TYPE:` marker of the block carrying the override.
struct Leg {
    label: &'static str,
    key: &'static str,
    value: &'static str,
    /// `;TYPE:` block the override must activate feedrate inside.
    block: &'static str,
    /// Feedrate word emitted at the overridden block start.
    feedrate: &'static str,
}

const LEGS: [Leg; 3] = [
    Leg {
        label: "control",
        key: "",
        value: "",
        block: "",
        feedrate: "",
    },
    Leg {
        label: "inner-wall-speed",
        key: "inner_wall_speed",
        value: "120",
        block: ";TYPE:Inner wall",
        feedrate: "G1 F7200",
    },
    Leg {
        label: "sparse-infill-speed",
        key: "sparse_infill_speed",
        value: "120",
        block: ";TYPE:Sparse infill",
        feedrate: "G1 F7200",
    },
];

/// Rewrites `Metadata/model_settings.config`, inserting the region
/// `metadata key` after part 2's `extruder` row. Verifies fixture shape.
fn project(leg: &Leg) -> Vec<u8> {
    let mut reader = zip::ZipArchive::new(Cursor::new(BASE)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/model_settings.config" {
            let text = String::from_utf8(bytes).unwrap();
            assert_eq!(
                text.matches("<part ").count(),
                2,
                "fixture must keep two model parts"
            );
            let anchor = "    <part id=\"2\" subtype=\"normal_part\">\n      <metadata key=\"name\" value=\"overlay-box.stl\"/>\n      <metadata key=\"extruder\" value=\"2\"/>\n";
            assert!(text.contains(anchor), "part 2 anchor must exist");
            let insertion = if leg.key.is_empty() {
                String::new()
            } else {
                format!(
                    "      <metadata key=\"{}\" value=\"{}\"/>\n",
                    leg.key, leg.value
                )
            };
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

/// Feedrate activity inside `leg.block` sections of the Native output:
/// the override must emit `leg.feedrate` inside at least one of the
/// overridden part's `;TYPE:` blocks (the untouched sibling part keeps
/// the base feedrate, so not every matching section carries it).
fn assert_native_feedrate_active(reference: &[u8], leg: &Leg) {
    if leg.block.is_empty() {
        return;
    }
    let text = String::from_utf8(reference.to_vec()).unwrap();
    assert!(
        text.contains(leg.block),
        "native reference lacks {} block for {}",
        leg.block,
        leg.label
    );
    let mut rest = text.as_str();
    let mut active = false;
    while let Some(start) = rest.find(leg.block) {
        let section_start = start + leg.block.len();
        let next = rest[section_start..]
            .find(";TYPE:")
            .map(|index| index + section_start);
        let section = match next {
            Some(index) => &rest[..index],
            None => rest,
        };
        if section.contains(leg.feedrate) {
            active = true;
            break;
        }
        rest = &rest[section_start..];
    }
    assert!(
        active,
        "native reference {} blocks lack {} for {}",
        leg.block, leg.feedrate, leg.label
    );
}

fn run_leg(leg: &Leg) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix(&format!("region-speed-{}-", leg.label))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let directory = root.join("case");
    let project = project(leg);
    let reference = native(&runner, &directory, &project);
    assert_native_feedrate_active(&reference, leg);

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
            "key":leg.key,"value":leg.value,
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

/// Control: unpatched fixture — must match Native byte-for-byte and stay
/// green before and after the bounded GREEN.
#[test]
fn orca_parity_region_speed_emission_control() {
    run_leg(&LEGS[0]);
}

/// Positive: part-level `inner_wall_speed=120` activates `G1 F7200` inside
/// the overridden part's `;TYPE:Inner wall` blocks in Native; Ares must
/// reproduce the whole Native output.
#[test]
fn orca_parity_region_speed_emission_inner_wall() {
    run_leg(&LEGS[1]);
}

/// Positive: part-level `sparse_infill_speed=120` activates `G1 F7200`
/// inside `;TYPE:Sparse infill` blocks in Native; Ares must reproduce the
/// whole Native output.
#[test]
fn orca_parity_region_speed_emission_sparse_infill() {
    run_leg(&LEGS[2]);
}
