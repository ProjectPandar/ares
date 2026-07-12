use super::*;

use serde_json::{Value, json};

use crate::PrintPathRole;

fn role_output(extra: Value, roles: Vec<PrintPathRole>) -> Result<String, SliceError> {
    let mut options_json = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 0,
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

fn assert_ordered(output: &str, expected: &[&str]) {
    let lines = output.lines().collect::<Vec<_>>();
    let mut start = 0;
    for expected_line in expected {
        let offset = lines[start..]
            .iter()
            .position(|line| line == expected_line)
            .unwrap_or_else(|| panic!("{expected_line} missing after {start}\n{output}"));
        start += offset + 1;
    }
}

#[test]
fn travel_retraction_wipe_splits_before_and_during_wipe() {
    let output = role_output(
        json!({
            "wipe": true,
            "wipe_distance": 0.5,
            "retract_before_wipe": 50
        }),
        vec![PrintPathRole::SparseInfill, PrintPathRole::SparseInfill],
    )
    .unwrap();

    assert_ordered(
        &output,
        &[
            "G1 X1 Y0 E0.03385 ; extrude",
            "G1 E-0.55 F1800 ; retract",
            "G1 X0.5 Y0 E-0.25 F3600 ; wipe and retract",
            "G1 X2 Y0 F7200 ; travel",
            "G1 E0.8 F1800 ; unretract",
        ],
    );
}

#[test]
fn travel_retraction_wipe_with_full_before_wipe_still_moves_without_e() {
    let output = role_output(
        json!({
            "wipe": true,
            "wipe_distance": 0.5,
            "retract_before_wipe": 100
        }),
        vec![PrintPathRole::SparseInfill, PrintPathRole::SparseInfill],
    )
    .unwrap();

    assert_ordered(
        &output,
        &[
            "G1 E-0.8 F1800 ; retract",
            "G1 X0.5 Y0 F3600 ; wipe and retract",
            "G1 X2 Y0 F7200 ; travel",
            "G1 E0.8 F1800 ; unretract",
        ],
    );
}

#[test]
fn travel_retraction_wipe_moves_speed_limited_excess_before_wipe() {
    let output = role_output(
        json!({
            "wipe": true,
            "wipe_distance": 0.25,
            "retract_before_wipe": 0
        }),
        vec![PrintPathRole::SparseInfill, PrintPathRole::SparseInfill],
    )
    .unwrap();

    assert_ordered(
        &output,
        &[
            "G1 E-0.675 F1800 ; retract",
            "G1 X0.75 Y0 E-0.125 F3600 ; wipe and retract",
            "G1 X2 Y0 F7200 ; travel",
            "G1 E0.8 F1800 ; unretract",
        ],
    );
}

#[test]
fn travel_retraction_wipe_clamps_to_previous_segment_length() {
    let output = role_output(
        json!({
            "wipe": true,
            "wipe_distance": 2.0,
            "retract_before_wipe": 0,
            "role_based_wipe_speed": false,
            "wipe_speed": 10
        }),
        vec![PrintPathRole::SparseInfill, PrintPathRole::SparseInfill],
    )
    .unwrap();

    assert_ordered(
        &output,
        &[
            "G1 X0 Y0 E-0.8 F600 ; wipe and retract",
            "G1 X2 Y0 F7200 ; travel",
            "G1 E0.8 F1800 ; unretract",
        ],
    );
}

#[test]
fn travel_retraction_wipe_speed_percent_uses_travel_speed_when_not_role_based() {
    let output = role_output(
        json!({
            "wipe": true,
            "wipe_distance": 0.5,
            "retract_before_wipe": 50,
            "role_based_wipe_speed": false,
            "wipe_speed": "80%"
        }),
        vec![PrintPathRole::SparseInfill, PrintPathRole::SparseInfill],
    )
    .unwrap();

    assert_ordered(
        &output,
        &[
            "G1 E-0.64375 F1800 ; retract",
            "G1 X0.5 Y0 E-0.15625 F5760 ; wipe and retract",
            "G1 X2 Y0 F7200 ; travel",
            "G1 E0.8 F1800 ; unretract",
        ],
    );
}

#[test]
fn travel_retraction_wipe_speed_clamps_to_ten_mm_s() {
    let output = role_output(
        json!({
            "wipe": true,
            "wipe_distance": 0.5,
            "retract_before_wipe": 50,
            "role_based_wipe_speed": false,
            "wipe_speed": 5
        }),
        vec![PrintPathRole::SparseInfill, PrintPathRole::SparseInfill],
    )
    .unwrap();

    assert_ordered(
        &output,
        &[
            "G1 E-0.4 F1800 ; retract",
            "G1 X0.5 Y0 E-0.4 F600 ; wipe and retract",
            "G1 X2 Y0 F7200 ; travel",
            "G1 E0.8 F1800 ; unretract",
        ],
    );
}

#[test]
fn disabled_or_suppressed_wipe_preserves_current_travel_retraction() {
    for extra in [
        json!({ "wipe": false, "wipe_distance": 0.5, "retract_before_wipe": 50 }),
        json!({ "wipe": true, "wipe_distance": 0, "retract_before_wipe": 50 }),
        json!({
            "wipe": true,
            "wipe_distance": 0.5,
            "retract_before_wipe": 50,
            "retraction_minimum_travel": 100
        }),
        json!({
            "wipe": true,
            "wipe_distance": 0.5,
            "retract_before_wipe": 50,
            "reduce_infill_retraction": true,
            "sparse_infill_density": 50
        }),
    ] {
        let output = role_output(
            extra,
            vec![PrintPathRole::SparseInfill, PrintPathRole::SparseInfill],
        )
        .unwrap();

        assert!(!output.contains("wipe and retract"));
    }
}

#[test]
fn travel_retraction_wipe_invalid_runtime_values_are_rejected_with_option_key() {
    for (key, value) in [
        ("wipe", json!("true")),
        ("wipe", json!([true, "bad"])),
        ("wipe_distance", json!([])),
        ("wipe_distance", json!([0.5, -0.1])),
        ("wipe_distance", json!("0.5,bad")),
        ("retract_before_wipe", json!([])),
        ("retract_before_wipe", json!([50, 101])),
        ("retract_before_wipe", json!("50,bad")),
        ("role_based_wipe_speed", json!("true")),
        ("role_based_wipe_speed", json!([true, "bad"])),
        ("wipe_speed", json!(true)),
        ("wipe_speed", json!("-1")),
        ("wipe_speed", json!("bad%")),
        ("wipe_speed", json!("NaN")),
    ] {
        let err = role_output(
            json!({ key: value }),
            vec![PrintPathRole::SparseInfill, PrintPathRole::SparseInfill],
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key), "{key} missing from {err}");
    }
}
