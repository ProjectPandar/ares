//! Extruder.cpp:199–253: shared filament config column, observed through public G-code.

use super::super::super::{native_api_cli::sha, native_stream_case, repo_root};
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::json;
use std::fs;

fn prefix<'a>(text: &'a str, marker: &str) -> Vec<&'a str> {
    let mut lines = text.lines().skip_while(|line| *line != marker);
    assert_eq!(lines.next(), Some(marker));
    let mut result = Vec::new();
    for line in lines {
        let command = line
            .split_once(';')
            .map_or(line, |(command, _)| command)
            .trim();
        if command.is_empty() {
            continue;
        }
        result.push(command);
        if command.starts_with("G1 E-") {
            return result;
        }
    }
    panic!("first layer has no plain retract");
}

fn check(label: &str, project: &[u8], marker: &str, expected_native: Option<&serde_json::Value>) {
    let (reference, _) = native_stream_case::qualify(
        (
            "retraction-column",
            &[
                "GCode.cpp:6609,10905,11239 set_config_index",
                "Print.cpp:5068-5070,5113-5147 filament config column",
                "Extruder.cpp:199-253 m_config_index getters",
                "GCodeWriter.cpp:1457-1466 ordinary retract",
            ],
        ),
        label,
        project,
        &sha(project),
        false,
    );
    let result = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            project,
            GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ConfigOverrides::default(),
            SlicerContext::new(0),
        ));
    let api = result.unwrap_or_else(|error| panic!("{error:?}; {:?}", reference.root));
    fs::write(reference.root.join("ares.api.gcode"), &api).unwrap();
    let native_prefix = prefix(&reference.native, marker);
    let api_prefix = prefix(std::str::from_utf8(&api).unwrap(), marker);
    let whole =
        crate::golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), &api);
    fs::write(
        reference.root.join("retraction-column-comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "case":label, "layer_marker":marker, "input_sha256":sha(project),
            "native_sha256":sha(reference.native.as_bytes()), "api_sha256":sha(&api),
            "native_prefix":native_prefix, "api_prefix":api_prefix,
            "prefix_matches":native_prefix == api_prefix,
            "strict_whole_output_comparison":whole,
            "whole_output_parity_claimed":false,
            "scope":"exact ordered literal commands from first declared layer marker through first G1 E-; no command suppression, number normalization, or resynchronization"
        }))
        .unwrap(),
    )
    .unwrap();
    if let Some(expected) = expected_native {
        assert_eq!(
            json!(native_prefix),
            *expected,
            "Native premise; {:?}",
            reference.root
        );
    } else {
        assert_eq!(
            native_prefix.last(),
            Some(&"G1 E-.7 F1860"),
            "single-head premise; {:?}",
            reference.root
        );
    }
    assert_eq!(api_prefix, native_prefix, "{label}; {:?}", reference.root);
}

fn fixture(name: &str, expected_native: serde_json::Value) {
    if crate::OrcaRunner::from_env().unwrap().is_none() {
        return;
    }
    let root = repo_root().join("tests/parity/per-head-retraction-column");
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("provenance.json")).unwrap()).unwrap();
    let case = manifest["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == name)
        .unwrap();
    let project = fs::read(root.join(case["input"].as_str().unwrap())).unwrap();
    assert_eq!(sha(&project), case["sha256"].as_str().unwrap());
    check(name, &project, "; CHANGE_LAYER", Some(&expected_native));
}

macro_rules! fixture_cases {
    ($($name:ident => $expected:expr),+ $(,)?) => {$ (
        #[test]
        fn $name() { fixture(stringify!($name), json!($expected)); }
    )+};
}

fixture_cases! {
    x2d_pla_power_loss_comments => ["G1 E-.4 F1800"],
    x2d_pla_generated_printer_ids_incomplete_map => ["G92 E0","G1 E-.4 F1800"],
    x2d_pla_power_loss_by_layer => ["G1 E-.4 F1800"],
    x2d_pla_power_loss_printer_configuration => ["G1 E-.4 F1800"],
    x2d_pla_concentric_ironing_auto_map => ["G1 E-3 F1200"],
    x2d_pla_bowden_inner_wall_nil_override => ["G92 E0","G1 E-3 F1200"],
    x2d_pla_direct_inner_wall => ["G92 E0","G1 E-.4 F1800"],
    x2d_pla_bowden_inner_wall => ["G92 E0","G1 E-3 F1200"],
    x2d_pla_bowden_nozzle_placeholder => ["G92 E0","G1 E-3 F1200"],
    x2d_pla_direct_nozzle_slot => ["G92 E0","G1 E-.4 F1800"],
    x2d_pla_bowden_nozzle_slot => ["G92 E0","G1 E-3 F1200"],
    x2d_pla_bowden_multi_value_overlay => ["G92 E0","G1 E-3 F1200"],
    x2d_pla_bowden_nil_overlay => ["G92 E0","G1 E-3 F1200"],
    x2d_pla_bowden_singleton_overlay => ["G92 E0","G1 E-3 F1200"],
    x2d_pla_direct_distinct_overlay_source => ["G92 E0","G1 E-.4 F1800"],
    x2d_pla_direct_equal_overlay_source => ["G92 E0","G1 E-.4 F1800"],
    x2d_pla_auto_map_toolchange_boundary => ["G1 E-3 F1200"],
    x2d_pla_auto_map_object_boundary => ["G1 E-.4 F1800"],
    x2d_pla_variant_order_stream => ["G1 E-3 F1200"],
    x2d_pla_variant_order_extrusion_speed => ["G1 E-3 F1200"],
    x2d_pla_toolchange_without_virtual_move_comments => ["G1 E-3 F1200"],
    x2d_pla_toolchange_with_virtual_move_comments => ["G1 E-3 F1200"],
}

#[test]
fn ender3_single_head_nil_retraction_overrides() {
    if crate::OrcaRunner::from_env().unwrap().is_none() {
        return;
    }
    let source = fs::read(repo_root().join("tests/parity/orca_cli_ender3.3mf")).unwrap();
    let project = super::input::settings_variant(
        &source,
        json!({
            "retraction_length":["0.7"], "filament_retraction_length":["nil"],
            "retraction_speed":["31"], "filament_retraction_speed":["nil"],
            "deretraction_speed":["29"], "filament_deretraction_speed":["nil"],
            "retract_when_changing_layer":["1"], "filament_retract_when_changing_layer":["nil"],
            "wipe":["0"], "filament_wipe":["nil"], "use_firmware_retraction":"0",
            "use_relative_e_distances":"1"
        })
        .as_object()
        .unwrap(),
    );
    let settings = super::super::super::application::settings(&project).unwrap();
    assert_eq!(settings["nozzle_diameter"].as_array().unwrap().len(), 1);
    check(
        "ender3-single-head-nil-retraction-overrides",
        &project,
        ";LAYER_CHANGE",
        None,
    );
}

/// Every emitted move precedes the config block, which is excluded until multi-extruder config
/// serialization matches.
fn before_config_block(gcode: &[u8]) -> &[u8] {
    let marker = b"; CONFIG_BLOCK_START";
    let end = gcode
        .windows(marker.len())
        .position(|window| window == marker)
        .unwrap();
    &gcode[..end]
}

#[test]
fn swapped_filament_map_wipe_budget() {
    if crate::OrcaRunner::from_env().unwrap().is_none() {
        return;
    }
    let source = fs::read(
        repo_root().join("tests/parity/j1-used-extruder-follows-emitted-tools/both-used.3mf"),
    )
    .unwrap();
    let project = super::input::settings_variant(
        &source,
        json!({
            "filament_map":["2","1"],
            "filament_retraction_speed":["30","20"],
            "filament_wipe_distance":["1","2"]
        })
        .as_object()
        .unwrap(),
    );
    let (reference, _) = native_stream_case::qualify(
        (
            "retraction-column",
            &[
                "GCode.cpp:578,584 calculateWipeRetractionLengths",
                "GCode.cpp:699 Wipe::wipe",
                "Extruder.cpp:31-34 set_config_index",
            ],
        ),
        "swapped-filament-map-wipe-budget",
        &project,
        &sha(&project),
        true,
    );
    assert!(
        reference
            .native
            .lines()
            .any(|line| line == "; filament_map = 2,1"),
        "Native must keep the saved map; {:?}",
        reference.root
    );
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            &project,
            GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ConfigOverrides::default(),
            SlicerContext::new(0),
        ))
        .unwrap_or_else(|error| panic!("{error:?}; {:?}", reference.root));
    fs::write(reference.root.join("ares.api.gcode"), &api).unwrap();
    let difference = crate::golden::compare_ordered_bytes_generator_only(
        before_config_block(reference.native.as_bytes()),
        before_config_block(&api),
    );
    fs::write(
        reference.root.join("wipe-budget-comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "before_config_block_difference":difference,
            "strict_whole_output_comparison":crate::golden::compare_ordered_bytes_generator_only(
                reference.native.as_bytes(),
                &api,
            ),
            "whole_output_parity_claimed":false,
        }))
        .unwrap(),
    )
    .unwrap();
    difference.unwrap_or_else(|error| panic!("{error}; {:?}", reference.root));
}
