use super::role_fan_gcode_support::*;
use super::*;

#[test]
fn internal_bridge_fan_speed_overrides_and_restores_layer_baseline() {
    let options = options(json!({
        "fan_min_speed": 40,
        "fan_max_speed": 40,
        "full_fan_speed_layer": 1,
        "close_fan_the_first_x_layers": 0,
        "internal_bridge_fan_speed": 75
    }));
    let output = role_sequence_output(&options);

    assert_eq!(
        fan_lines(&output),
        vec!["M106 S102", "M106 S191", "M106 S102"]
    );
    assert_line_before(&output, "M106 S191", ";EXTRUSION:print:internal_bridge:");
    assert_line_after(&output, "M106 S191", ";EXTRUSION:print:external_perimeter:");
    assert_line_before_last(&output, "M106 S102", ";EXTRUSION:print:sparse_infill:");
}

#[test]
fn internal_bridge_fan_speed_turns_fan_off_after_override_without_baseline() {
    let options = options(json!({
        "fan_max_speed": 0,
        "close_fan_the_first_x_layers": 0,
        "internal_bridge_fan_speed": 75
    }));
    let output = role_sequence_output(&options);

    assert_eq!(fan_lines(&output), vec!["M106 S191", "M106 S0"]);
    assert_line_before(&output, "M106 S191", ";EXTRUSION:print:internal_bridge:");
    assert_line_before(&output, "M106 S0", ";EXTRUSION:print:sparse_infill:");
}

#[test]
fn explicit_internal_bridge_fan_speed_is_not_ramp_adjusted() {
    let options = options(json!({
        "fan_max_speed": 0,
        "full_fan_speed_layer": 4,
        "close_fan_the_first_x_layers": 0,
        "internal_bridge_fan_speed": 75
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[PrintPathRole::InternalBridge, PrintPathRole::SparseInfill],
    );

    assert_eq!(fan_lines(&output), vec!["M106 S191", "M106 S0"]);
}

#[test]
fn fallback_internal_bridge_fan_speed_uses_ramped_overhang_speed() {
    let options = options(json!({
        "fan_max_speed": 0,
        "full_fan_speed_layer": 4,
        "close_fan_the_first_x_layers": 0,
        "overhang_fan_speed": 100,
        "internal_bridge_fan_speed": -1
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[PrintPathRole::InternalBridge, PrintPathRole::SparseInfill],
    );

    assert_eq!(fan_lines(&output), vec!["M106 S63", "M106 S0"]);
}

#[test]
fn invalid_internal_bridge_fan_speed_reaches_slice_error() {
    let options = options(json!({ "internal_bridge_fan_speed": 101 }));
    let pipeline = role_sequence_pipeline(&options);

    let err = crate::gcode::format_gcode(&pipeline, &options).unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("internal_bridge_fan_speed"));
}
