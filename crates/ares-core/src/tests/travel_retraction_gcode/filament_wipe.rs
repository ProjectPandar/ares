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
fn filament_wipe_enables_travel_wipe_over_unprefixed_false() {
    let output = role_output(
        json!({
            "wipe": false,
            "filament_wipe": [true, false],
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
fn filament_wipe_disables_unprefixed_travel_wipe() {
    let output = role_output(
        json!({
            "wipe": true,
            "filament_wipe": false,
            "wipe_distance": 0.5,
            "retract_before_wipe": 50
        }),
        vec![PrintPathRole::SparseInfill, PrintPathRole::SparseInfill],
    )
    .unwrap();

    assert_ordered(
        &output,
        &[
            "G1 E-0.8 F1800 ; retract",
            "G1 X2 Y0 F7200 ; travel",
            "G1 E0.8 F1800 ; unretract",
        ],
    );
    assert!(!output.contains("wipe and retract"));
}

#[test]
fn serialized_filament_wipe_uses_first_value() {
    let output = role_output(
        json!({
            "wipe": false,
            "filament_wipe": "1,0",
            "wipe_distance": 0.5,
            "retract_before_wipe": 50
        }),
        vec![PrintPathRole::SparseInfill, PrintPathRole::SparseInfill],
    )
    .unwrap();

    assert!(output.contains("G1 X0.5 Y0 E-0.25 F3600 ; wipe and retract"));
}

#[test]
fn nil_filament_wipe_falls_back_to_unprefixed_wipe() {
    let output = role_output(
        json!({
            "wipe": true,
            "filament_wipe": [null, false],
            "wipe_distance": 0.5,
            "retract_before_wipe": 50
        }),
        vec![PrintPathRole::SparseInfill, PrintPathRole::SparseInfill],
    )
    .unwrap();

    assert!(output.contains("G1 X0.5 Y0 E-0.25 F3600 ; wipe and retract"));
}

#[test]
fn invalid_filament_wipe_values_are_rejected_with_option_key() {
    for value in [
        json!([]),
        json!("true"),
        json!("1,bad"),
        json!([true, "bad"]),
        json!([null, 1]),
    ] {
        let err = role_output(
            json!({
                "filament_wipe": value
            }),
            vec![PrintPathRole::SparseInfill, PrintPathRole::SparseInfill],
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(
            err.to_string().contains("filament_wipe"),
            "filament_wipe was missing from {err}"
        );
    }
}
