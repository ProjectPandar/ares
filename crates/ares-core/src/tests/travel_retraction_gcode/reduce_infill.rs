use super::*;

use crate::PrintPathRole;
use serde_json::{Value, json};

fn role_output(extra: Value, roles: Vec<PrintPathRole>) -> Result<String, SliceError> {
    let mut options_json = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 50,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false,
        "retract_when_changing_layer": false,
        "retraction_minimum_travel": 0.25,
        "z_hop": 0,
        "gcode_comments": true
    });
    options_json
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    let options: SliceOptions = serde_json::from_value(options_json).unwrap();
    let pipeline =
        crate::pipeline::layer_change_test_support::role_layers_pipeline(&options, vec![roles]);
    Ok(String::from_utf8(crate::gcode::format_gcode(&pipeline, &options)?).unwrap())
}

fn assert_retracts_before_sparse_travel(output: &str) {
    let travel = line_index(output, "G1 X2 Y0 F7200 ; travel");
    let retract = previous_line_index(output, travel, "G1 E-0.8 F1800 ; retract");
    let unretract = next_line_index(output, travel, "G1 E0.8 F1800 ; unretract");

    assert!(retract < travel);
    assert!(travel < unretract);
}

fn assert_no_travel_retract(output: &str) {
    assert!(output.lines().any(|line| line == "G1 X2 Y0 F7200 ; travel"));
    assert!(
        !output
            .lines()
            .any(|line| line == "G1 E-0.8 F1800 ; retract")
    );
    assert!(
        !output
            .lines()
            .any(|line| line == "G1 E0.8 F1800 ; unretract")
    );
}

#[test]
fn enabled_reduce_infill_retraction_suppresses_sparse_infill_travel_retract() {
    let output = role_output(
        json!({ "reduce_infill_retraction": true }),
        vec![PrintPathRole::SparseInfill, PrintPathRole::SparseInfill],
    )
    .unwrap();

    assert_no_travel_retract(&output);
}

#[test]
fn omitted_reduce_infill_retraction_preserves_sparse_infill_travel_retract() {
    let output = role_output(
        json!({}),
        vec![PrintPathRole::SparseInfill, PrintPathRole::SparseInfill],
    )
    .unwrap();

    assert_retracts_before_sparse_travel(&output);
}

#[test]
fn disabled_reduce_infill_retraction_preserves_sparse_infill_travel_retract() {
    let output = role_output(
        json!({ "reduce_infill_retraction": false }),
        vec![PrintPathRole::SparseInfill, PrintPathRole::SparseInfill],
    )
    .unwrap();

    assert_retracts_before_sparse_travel(&output);
}

#[test]
fn density_zero_preserves_sparse_infill_travel_retract() {
    let output = role_output(
        json!({
            "reduce_infill_retraction": true,
            "sparse_infill_density": 0
        }),
        vec![PrintPathRole::SparseInfill, PrintPathRole::SparseInfill],
    )
    .unwrap();

    assert_retracts_before_sparse_travel(&output);
}

#[test]
fn spiral_mode_density_normalization_preserves_sparse_infill_travel_retract() {
    let output = role_output(
        json!({
            "reduce_infill_retraction": true,
            "sparse_infill_density": 50,
            "spiral_mode": true
        }),
        vec![PrintPathRole::SparseInfill, PrintPathRole::SparseInfill],
    )
    .unwrap();

    assert_retracts_before_sparse_travel(&output);
}

#[test]
fn perimeter_target_preserves_travel_retract() {
    let output = role_output(
        json!({ "reduce_infill_retraction": true }),
        vec![
            PrintPathRole::SparseInfill,
            PrintPathRole::ExternalPerimeter,
        ],
    )
    .unwrap();

    assert_retracts_before_sparse_travel(&output);
}

#[test]
fn previous_perimeter_preserves_internal_infill_travel_retract() {
    let output = role_output(
        json!({ "reduce_infill_retraction": true }),
        vec![
            PrintPathRole::ExternalPerimeter,
            PrintPathRole::SparseInfill,
        ],
    )
    .unwrap();

    assert_retracts_before_sparse_travel(&output);
}

#[test]
fn skipped_reduce_infill_retraction_skips_travel_z_hop() {
    let output = role_output(
        json!({
            "reduce_infill_retraction": true,
            "z_hop": 0.4
        }),
        vec![PrintPathRole::SparseInfill, PrintPathRole::SparseInfill],
    )
    .unwrap();

    assert_no_travel_retract(&output);
    assert!(!output.lines().any(|line| line == "G1 Z0.6 F7200 ; lift Z"));
    assert!(
        !output
            .lines()
            .any(|line| line == "G1 Z0.2 F7200 ; restore layer Z")
    );
}

#[test]
fn invalid_reduce_infill_retraction_values_are_rejected_with_option_key() {
    for value in [json!([]), json!("true"), json!(1), json!([true, "bad"])] {
        let err = role_output(
            json!({ "reduce_infill_retraction": value }),
            vec![PrintPathRole::SparseInfill, PrintPathRole::SparseInfill],
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("reduce_infill_retraction"));
    }
}
