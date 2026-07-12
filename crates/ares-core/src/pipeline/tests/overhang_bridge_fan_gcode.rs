use super::role_fan_gcode_support::*;
use super::*;

#[test]
fn bridge_fan_speed_overrides_and_restores_layer_baseline() {
    let options = options(json!({
        "fan_min_speed": 40,
        "fan_max_speed": 40,
        "full_fan_speed_layer": 1,
        "close_fan_the_first_x_layers": 0,
        "overhang_fan_speed": 75
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[
            PrintPathRole::ExternalPerimeter,
            PrintPathRole::Bridge,
            PrintPathRole::SparseInfill,
        ],
    );

    assert_eq!(
        fan_lines(&output),
        vec!["M106 S102", "M106 S191", "M106 S102"]
    );
    assert_line_before(&output, "M106 S191", ";EXTRUSION:print:bridge:");
    assert_line_before_last(&output, "M106 S102", ";EXTRUSION:print:sparse_infill:");
}

#[test]
fn default_overhang_speed_not_above_baseline_emits_no_bridge_override() {
    let options = options(json!({
        "fan_min_speed": 100,
        "fan_max_speed": 100,
        "full_fan_speed_layer": 1,
        "close_fan_the_first_x_layers": 0
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[
            PrintPathRole::ExternalPerimeter,
            PrintPathRole::Bridge,
            PrintPathRole::SparseInfill,
        ],
    );

    assert_eq!(fan_lines(&output), vec!["M106 S255"]);
}

#[test]
fn default_threshold_applies_overhang_fan_to_overhang_perimeter() {
    let options = options(json!({
        "fan_max_speed": 0,
        "close_fan_the_first_x_layers": 0,
        "overhang_fan_speed": 75
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[
            PrintPathRole::OverhangPerimeter,
            PrintPathRole::SparseInfill,
        ],
    );

    assert_eq!(fan_lines(&output), vec!["M106 S191", "M106 S0"]);
    assert_line_before(&output, "M106 S191", ";EXTRUSION:print:overhang_perimeter:");
    assert_line_before(&output, "M106 S0", ";EXTRUSION:print:sparse_infill:");
}

#[test]
fn zero_threshold_applies_overhang_fan_to_external_perimeter() {
    let options = options(json!({
        "fan_max_speed": 0,
        "close_fan_the_first_x_layers": 0,
        "overhang_fan_speed": 75,
        "overhang_fan_threshold": "0%"
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[
            PrintPathRole::ExternalPerimeter,
            PrintPathRole::SparseInfill,
        ],
    );

    assert_eq!(fan_lines(&output), vec!["M106 S191", "M106 S0"]);
    assert_line_before(&output, "M106 S191", ";EXTRUSION:print:external_perimeter:");
    assert_line_before(&output, "M106 S0", ";EXTRUSION:print:sparse_infill:");
}

#[test]
fn intermediate_threshold_preserves_overhang_without_forcing_external_perimeter() {
    let options = options(json!({
        "fan_max_speed": 0,
        "close_fan_the_first_x_layers": 0,
        "overhang_fan_speed": 75,
        "overhang_fan_threshold": "50%"
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[
            PrintPathRole::ExternalPerimeter,
            PrintPathRole::OverhangPerimeter,
            PrintPathRole::SparseInfill,
        ],
    );

    assert_eq!(fan_lines(&output), vec!["M106 S191", "M106 S0"]);
    assert_line_before(&output, "M106 S191", ";EXTRUSION:print:overhang_perimeter:");
    assert_line_before(&output, "M106 S0", ";EXTRUSION:print:sparse_infill:");
}

#[test]
fn invalid_overhang_fan_threshold_reaches_slice_error() {
    let options = options(json!({ "overhang_fan_threshold": "33%" }));
    let pipeline = role_sequence_pipeline(&options);

    let err = crate::gcode::format_gcode(&pipeline, &options).unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("overhang_fan_threshold"));
}

#[test]
fn overhang_speed_below_baseline_emits_no_bridge_override() {
    let options = options(json!({
        "fan_min_speed": 80,
        "fan_max_speed": 80,
        "full_fan_speed_layer": 1,
        "close_fan_the_first_x_layers": 0,
        "overhang_fan_speed": 75
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[
            PrintPathRole::ExternalPerimeter,
            PrintPathRole::Bridge,
            PrintPathRole::SparseInfill,
        ],
    );

    assert_eq!(fan_lines(&output), vec!["M106 S204"]);
}

#[test]
fn disabled_overhang_bridge_fan_suppresses_bridge_and_internal_bridge_overrides() {
    let options = options(json!({
        "enable_overhang_bridge_fan": false,
        "fan_min_speed": 40,
        "fan_max_speed": 40,
        "full_fan_speed_layer": 1,
        "close_fan_the_first_x_layers": 0,
        "overhang_fan_speed": 75,
        "internal_bridge_fan_speed": 75
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[
            PrintPathRole::ExternalPerimeter,
            PrintPathRole::Bridge,
            PrintPathRole::InternalBridge,
            PrintPathRole::SparseInfill,
        ],
    );

    assert_eq!(fan_lines(&output), vec!["M106 S102"]);
}

#[test]
fn bridge_fan_speed_turns_fan_on_without_baseline_then_restores_off() {
    let options = options(json!({
        "fan_max_speed": 0,
        "close_fan_the_first_x_layers": 0,
        "overhang_fan_speed": 75
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[PrintPathRole::Bridge, PrintPathRole::SparseInfill],
    );

    assert_eq!(fan_lines(&output), vec!["M106 S191", "M106 S0"]);
}

#[test]
fn long_layer_zero_baseline_turns_fan_off_after_role_override() {
    let options = options(json!({
        "fan_min_speed": 20,
        "fan_max_speed": 100,
        "full_fan_speed_layer": 1,
        "close_fan_the_first_x_layers": 0,
        "fan_cooling_layer_time": 0.0,
        "overhang_fan_speed": 75
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[PrintPathRole::Bridge, PrintPathRole::SparseInfill],
    );

    assert_eq!(fan_lines(&output), vec!["M106 S191", "M106 S0"]);
    assert_line_before(&output, "M106 S191", ";EXTRUSION:print:bridge:");
    assert_line_before(&output, "M106 S0", ";EXTRUSION:print:sparse_infill:");
}

#[test]
fn reduce_fan_stop_start_frequency_restores_minimum_after_role_override() {
    let options = options(json!({
        "reduce_fan_stop_start_freq": true,
        "fan_min_speed": 20,
        "fan_max_speed": 100,
        "full_fan_speed_layer": 1,
        "close_fan_the_first_x_layers": 0,
        "fan_cooling_layer_time": 0.0,
        "overhang_fan_speed": 75
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[PrintPathRole::Bridge, PrintPathRole::SparseInfill],
    );

    assert_eq!(
        fan_lines(&output),
        vec!["M106 S51", "M106 S191", "M106 S51"]
    );
    assert_line_before(&output, "M106 S191", ";EXTRUSION:print:bridge:");
    assert_line_before_last(&output, "M106 S51", ";EXTRUSION:print:sparse_infill:");
}

#[test]
fn bridge_fan_speed_is_ramp_adjusted() {
    let options = options(json!({
        "fan_max_speed": 0,
        "full_fan_speed_layer": 4,
        "close_fan_the_first_x_layers": 0,
        "overhang_fan_speed": 100
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[PrintPathRole::Bridge, PrintPathRole::SparseInfill],
    );

    assert_eq!(fan_lines(&output), vec!["M106 S63", "M106 S0"]);
}

#[test]
fn close_fan_first_layers_suppresses_bridge_override() {
    let options = options(json!({
        "fan_max_speed": 0,
        "close_fan_the_first_x_layers": 1,
        "overhang_fan_speed": 75
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[PrintPathRole::Bridge, PrintPathRole::SparseInfill],
    );

    assert!(fan_lines(&output).is_empty());
}
