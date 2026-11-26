//! Whole-G-code parity for `BrimType::btLeadingEdgeOnly` on flat-bed printers
//! (`PrintConfig.hpp:466`, `Brim.cpp:506-508`, `PerimeterGenerator.cpp:3204-3212`,
//! `Support/SupportCommon.cpp:420-422`) and tilted belt printers (`BeltBrim.cpp`).

use crate::runner::{OrcaRunner, native_api_cli::NativeCase};
use ares_core::GenerationMetadata;
use serde_json::{Map, Value, json};
use std::io::{Cursor, Read, Write};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

const TWO_HOLES: &[u8] = include_bytes!("../../../../tests/parity/outer-only-brim/two-holes.3mf");
const CLIPPED_CONTACT: &[u8] =
    include_bytes!("../../../../tests/parity/outer-only-brim/clipped-contact.3mf");
const SUPPORT_OVERHANG: &[u8] = include_bytes!(
    "../../../../tests/parity/validation/brim_support_and_painted_ears/outer-inner.3mf"
);
const X1C_ADAPTIVE: &[u8] =
    include_bytes!("../../../../tests/parity/adaptive-cubic-bodies/x1c-adaptive.3mf");
const CUBE_CONTROL: &[u8] =
    include_bytes!("../../../../tests/parity/mesh-edge-pairing/cube-control.3mf");

fn patch_project(
    source: &[u8],
    settings: &[(&str, &str)],
    object_brim_type: Option<&str>,
) -> Vec<u8> {
    let mut reader = ZipArchive::new(Cursor::new(source)).unwrap();
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut entry = reader.by_index(index).unwrap();
        let name = entry.name().to_owned();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        if name == "Metadata/project_settings.config" {
            let mut config: Map<String, Value> = serde_json::from_slice(&bytes).unwrap();
            for &(key, value) in settings {
                config.insert(key.to_owned(), json!(value));
            }
            bytes = serde_json::to_vec(&config).unwrap();
        } else if name == "Metadata/model_settings.config"
            && let Some(brim_type) = object_brim_type
        {
            let xml = String::from_utf8(bytes).unwrap();
            let marker = "<metadata key=\"extruder\" value=\"1\"/>";
            assert_eq!(xml.matches(marker).count(), 1);
            let replacement =
                format!("{marker}\n    <metadata key=\"brim_type\" value=\"{brim_type}\"/>");
            bytes = xml.replacen(marker, &replacement, 1).into_bytes();
        }
        writer
            .start_file(name, SimpleFileOptions::default())
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn first_layer_slice(gcode: &str) -> &str {
    let after_first = gcode
        .split_once(";LAYER_CHANGE\n")
        .expect("first layer marker")
        .1;
    after_first
        .split_once(";LAYER_CHANGE\n")
        .map_or(after_first, |(first, _)| first)
}

fn has_brim_marker(gcode: &str) -> bool {
    gcode.contains(";TYPE:Brim\n") || gcode.contains("; FEATURE: Brim\n")
}

fn run_flat_case(
    prefix: &str,
    project: &[u8],
    expect_brim: bool,
    expect_outer_first: Option<bool>,
    expect_config_value: &str,
) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let native = NativeCase::slice(&runner, project, prefix);
    assert_eq!(
        has_brim_marker(&native.native),
        expect_brim,
        "native brim presence in {:?}",
        native.root
    );
    let expected_line = format!("; brim_type = {expect_config_value}");
    assert!(
        native.native.lines().any(|line| line == expected_line),
        "native config block missing {expected_line} in {:?}",
        native.root
    );
    if let Some(outer_first) = expect_outer_first {
        let layer0 = first_layer_slice(&native.native);
        let outer = layer0
            .find(";TYPE:Outer wall\n")
            .expect("native outer wall");
        let inner = layer0
            .find(";TYPE:Inner wall\n")
            .expect("native inner wall");
        assert_eq!(
            outer < inner,
            outer_first,
            "native layer-0 wall order in {:?}",
            native.root
        );
    }
    let (api, _) = native.compare(GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0));
    let text = std::str::from_utf8(&api).unwrap();
    assert_eq!(
        has_brim_marker(text),
        expect_brim,
        "ares brim presence in {:?}",
        native.root
    );
    assert!(
        text.lines().any(|line| line == expected_line),
        "ares config block missing {expected_line} in {:?}",
        native.root
    );
}

#[test]
fn leading_edge_only_classic_flat_bed_matches_native() {
    let project = patch_project(
        TWO_HOLES,
        &[
            ("brim_type", "leading_edge_only"),
            ("brim_width", "5"),
            ("brim_object_gap", "0.1"),
            ("wall_generator", "classic"),
            ("wall_sequence", "inner wall/outer wall"),
        ],
        None,
    );
    run_flat_case(
        "leading-edge-classic-",
        &project,
        true,
        Some(true),
        "leading_edge_only",
    );
}

#[test]
fn leading_edge_only_arachne_flat_bed_matches_native() {
    let project = patch_project(
        TWO_HOLES,
        &[
            ("brim_type", "leading_edge_only"),
            ("brim_width", "5"),
            ("wall_generator", "arachne"),
            ("wall_sequence", "inner wall/outer wall"),
        ],
        None,
    );
    run_flat_case(
        "leading-edge-arachne-",
        &project,
        true,
        Some(true),
        "leading_edge_only",
    );
}

#[test]
fn leading_edge_only_efc_outline_matches_native() {
    let project = patch_project(
        TWO_HOLES,
        &[
            ("brim_type", "leading_edge_only"),
            ("brim_width", "5"),
            ("elefant_foot_compensation", "0.2"),
            ("brim_use_efc_outline", "1"),
        ],
        None,
    );
    run_flat_case(
        "leading-edge-efc-",
        &project,
        true,
        None,
        "leading_edge_only",
    );
}

#[test]
fn leading_edge_only_clipped_contact_retention_matches_native() {
    let project = patch_project(CLIPPED_CONTACT, &[("brim_type", "leading_edge_only")], None);
    run_flat_case(
        "leading-edge-clipped-",
        &project,
        true,
        None,
        "leading_edge_only",
    );
}

#[test]
fn leading_edge_only_support_mask_and_trimming_match_native() {
    let tree = patch_project(
        SUPPORT_OVERHANG,
        &[
            ("brim_type", "leading_edge_only"),
            ("brim_width", "8"),
            ("brim_object_gap", "1.0"),
        ],
        None,
    );
    run_flat_case(
        "leading-edge-tree-support-",
        &tree,
        true,
        None,
        "leading_edge_only",
    );

    let normal = patch_project(
        SUPPORT_OVERHANG,
        &[
            ("brim_type", "leading_edge_only"),
            ("brim_width", "8"),
            ("brim_object_gap", "1.5"),
            ("enable_support", "1"),
            ("support_type", "normal(auto)"),
            ("raft_first_layer_expansion", "3"),
        ],
        None,
    );
    run_flat_case(
        "leading-edge-normal-support-",
        &normal,
        true,
        None,
        "leading_edge_only",
    );
}

#[test]
fn leading_edge_only_x1c_flat_bed_matches_native() {
    let project = patch_project(
        X1C_ADAPTIVE,
        &[("brim_type", "leading_edge_only"), ("brim_width", "5")],
        None,
    );
    run_flat_case(
        "leading-edge-x1c-",
        &project,
        true,
        None,
        "leading_edge_only",
    );
}

#[test]
fn leading_edge_only_zero_width_and_object_override_match_native() {
    let zero_width = patch_project(
        TWO_HOLES,
        &[
            ("brim_type", "leading_edge_only"),
            ("brim_width", "0"),
            ("wall_sequence", "inner wall/outer wall"),
        ],
        None,
    );
    run_flat_case(
        "leading-edge-zero-width-",
        &zero_width,
        false,
        Some(false),
        "leading_edge_only",
    );

    let object_override = patch_project(
        TWO_HOLES,
        &[("brim_type", "no_brim"), ("brim_width", "5")],
        Some("leading_edge_only"),
    );
    run_flat_case(
        "leading-edge-object-override-",
        &object_override,
        true,
        Some(true),
        "no_brim",
    );
}

#[test]
fn leading_edge_only_belt_printer_matches_native() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project = patch_project(
        CUBE_CONTROL,
        &[
            ("belt_printer", "1"),
            ("belt_slice_rotation", "x"),
            ("belt_slice_rotation_angle", "45"),
            ("gcode_remap_x", "rev_x"),
            ("gcode_remap_y", "pos_z"),
            ("gcode_remap_z", "pos_y"),
            ("enable_arc_fitting", "disabled"),
            ("brim_type", "leading_edge_only"),
            ("brim_width", "5"),
        ],
        None,
    );
    let native = NativeCase::slice(&runner, &project, "leading-edge-belt-");
    assert_eq!(
        native.native.matches(";TYPE:Brim\n").count(),
        2,
        "expected leading-edge-only belt brim markers in {:?}",
        native.root
    );
    let (api, _) = native.compare(GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0));
    assert_eq!(
        std::str::from_utf8(&api)
            .unwrap()
            .matches(";TYPE:Brim\n")
            .count(),
        2,
        "expected leading-edge-only belt brim markers in Ares output {:?}",
        native.root
    );
}
