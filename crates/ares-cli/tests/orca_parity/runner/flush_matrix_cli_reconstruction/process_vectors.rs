//! PrintApply.cpp:1429; GCode.cpp:8244–8352: applied selected process vectors.
use super::{OrcaRunner, application, json, project};
use crate::runner::native_api_cli::{NativeCase, sha};
use std::{collections::BTreeMap, fs};

const SPEEDS: &[&str] = &[
    "bridge_speed",
    "gap_infill_speed",
    "initial_layer_infill_speed",
    "initial_layer_speed",
    "initial_layer_travel_speed",
    "internal_bridge_speed",
    "internal_solid_infill_speed",
    "overhang_1_4_speed",
    "overhang_2_4_speed",
    "overhang_3_4_speed",
    "overhang_4_4_speed",
    "small_perimeter_speed",
    "sparse_infill_speed",
    "support_interface_speed",
    "support_speed",
    "top_surface_speed",
    "travel_speed",
    "travel_speed_z",
];
const JERK: &[&str] = &["default_junction_deviation", "initial_layer_travel_jerk"];
const THRESHOLD_FLOW: &[&str] = &["small_perimeter_threshold", "top_solid_infill_flow_ratio"];

fn footer(text: &str) -> BTreeMap<&str, &str> {
    let (_, block) = text.rsplit_once("; CONFIG_BLOCK_START\n").unwrap();
    block
        .lines()
        .filter_map(|line| line.strip_prefix("; ")?.split_once(" = "))
        .collect()
}

fn check(case: &str, family: &str, keys: &[&str]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let changes = match case {
        "original" => vec![],
        "missing" => vec![("flush_volumes_matrix", None)],
        "high-flow-rebuild" => vec![
            (
                "nozzle_volume_type",
                Some(json!(["High Flow", "High Flow"])),
            ),
            ("flush_volumes_matrix", None),
        ],
        _ => unreachable!(),
    };
    let exported = project(&runner, case, &changes);
    let reference = NativeCase::slice(
        &runner,
        &exported.project,
        &format!("process-{family}-{case}-"),
    );
    let replay = NativeCase::slice(
        &runner,
        &exported.project,
        &format!("process-{family}-{case}-replay-"),
    );
    let native = footer(&reference.native);
    assert_eq!(
        native,
        footer(&replay.native),
        "Native footer repeatability"
    );
    if case == "high-flow-rebuild" {
        let bridge = native["bridge_speed"].split(',').collect::<Vec<_>>();
        assert!(
            bridge.windows(2).any(|pair| pair[0] != pair[1]),
            "nonuniform Native High Flow slots"
        );
    }
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(application::slice_project_plate_validated(
            &exported.project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ));
    if let Ok(bytes) = &api {
        fs::write(reference.root.join("ares.api.gcode"), bytes).unwrap();
    }
    let cli = application::cli_command()
        .args(["slice", "-o"])
        .arg(reference.root.join("ares.cli.gcode"))
        .arg(reference.root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(reference.root.join("ares.cli.stdout"), &cli.stdout).unwrap();
    fs::write(reference.root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    fs::write(reference.root.join("endpoints.json"), serde_json::to_vec_pretty(&json!({
        "input_sha256":sha(&exported.project), "case":case, "family":family,
        "api_error":api.as_ref().err().map(|error|format!("{error:?}")), "cli_code":cli.status.code(),
        "native_replay_root":replay.root,
    })).unwrap()).unwrap();
    let api = api.unwrap();
    assert!(cli.status.success(), "CLI: {:?}", reference.root);
    let cli = fs::read(reference.root.join("ares.cli.gcode")).unwrap();
    let api_text = std::str::from_utf8(&api).unwrap();
    let cli_text = std::str::from_utf8(&cli).unwrap();
    let api_footer = footer(api_text);
    let cli_footer = footer(cli_text);
    let rows = keys.iter().map(|key| {
        let expected = native[*key];
        json!({"key":key, "native":expected, "api":api_footer.get(key), "cli":cli_footer.get(key),
            "matches":api_footer.get(key)==Some(&expected) && cli_footer.get(key)==Some(&expected)})
    }).collect::<Vec<_>>();
    fs::write(reference.root.join("process-vectors.json"), serde_json::to_vec_pretty(&json!({
        "case":case, "family":family, "input_sha256":sha(&exported.project), "rows":rows,
        "native_footer":native, "api_footer":api_footer, "cli_footer":cli_footer,
        "api_cli_equal":api==cli,
        "complete_api_comparison":crate::golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), &api),
        "complete_cli_comparison":crate::golden::compare_ordered_bytes_generator_only(reference.native.as_bytes(), &cli),
        "whole_output_parity_claimed":false,
    })).unwrap()).unwrap();
    assert_eq!(api, cli, "raw API/CLI equality: {:?}", reference.root);
    assert!(
        rows.iter().all(|row| row["matches"] == true),
        "{family}/{case}: {rows:?}; {:?}",
        reference.root
    );
}

#[test]
fn native_process_speeds_retained_standard() {
    check("original", "speeds", SPEEDS);
}
#[test]
fn native_process_speeds_retained_missing_matrix() {
    check("missing", "speeds", SPEEDS);
}
#[test]
fn native_process_speeds_retained_high_flow() {
    check("high-flow-rebuild", "speeds", SPEEDS);
}
#[test]
fn native_process_jerk_retained_standard() {
    check("original", "jerk", JERK);
}
#[test]
fn native_process_jerk_retained_missing_matrix() {
    check("missing", "jerk", JERK);
}
#[test]
fn native_process_jerk_retained_high_flow() {
    check("high-flow-rebuild", "jerk", JERK);
}
#[test]
fn native_process_threshold_flow_retained_standard() {
    check("original", "threshold-flow", THRESHOLD_FLOW);
}
#[test]
fn native_process_threshold_flow_retained_missing_matrix() {
    check("missing", "threshold-flow", THRESHOLD_FLOW);
}
#[test]
fn native_process_threshold_flow_retained_high_flow() {
    check("high-flow-rebuild", "threshold-flow", THRESHOLD_FLOW);
}
