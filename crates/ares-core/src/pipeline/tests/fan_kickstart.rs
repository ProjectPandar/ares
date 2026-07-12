use super::role_fan_gcode_support::*;
use super::*;

#[test]
fn zero_fan_kickstart_preserves_baseline_fan_sequence() {
    let options = options(json!({
        "fan_min_speed": 40,
        "fan_max_speed": 40,
        "full_fan_speed_layer": 1,
        "close_fan_the_first_x_layers": 0,
        "fan_kickstart": 0.0
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[
            PrintPathRole::ExternalPerimeter,
            PrintPathRole::SparseInfill,
        ],
    );

    assert_eq!(fan_lines(&output), vec!["M106 S102"]);
}

#[test]
fn fan_kickstart_pulses_before_baseline_target_and_restores_after_move_time() {
    let options = options(json!({
        "fan_min_speed": 40,
        "fan_max_speed": 40,
        "full_fan_speed_layer": 1,
        "close_fan_the_first_x_layers": 0,
        "fan_kickstart": 0.01
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[
            PrintPathRole::ExternalPerimeter,
            PrintPathRole::SparseInfill,
        ],
    );

    assert_eq!(fan_lines(&output), vec!["M106 S255", "M106 S102"]);
    assert_line_before(&output, "M106 S255", ";EXTRUSION:print:external_perimeter:");
    assert_line_before(&output, "M106 S102", ";EXTRUSION:print:sparse_infill:");
}

#[test]
fn fan_kickstart_wraps_role_override_and_then_restores_baseline_without_pulse() {
    let options = options(json!({
        "fan_min_speed": 40,
        "fan_max_speed": 40,
        "full_fan_speed_layer": 1,
        "close_fan_the_first_x_layers": 0,
        "overhang_fan_speed": 75,
        "fan_kickstart": 0.01
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
        vec![
            "M106 S255",
            "M106 S102",
            "M106 S255",
            "M106 S191",
            "M106 S102"
        ]
    );
    assert_line_before(&output, "M106 S191", ";EXTRUSION:print:sparse_infill:");
    assert_line_before_last(&output, "M106 S102", ";EXTRUSION:print:sparse_infill:");
}

#[test]
fn fan_kickstart_skips_small_upshift_and_downshift() {
    let options = options(json!({
        "fan_min_speed": 40,
        "fan_max_speed": 40,
        "full_fan_speed_layer": 1,
        "close_fan_the_first_x_layers": 0,
        "overhang_fan_speed": 50,
        "fan_kickstart": 0.01
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
        vec!["M106 S255", "M106 S102", "M106 S127", "M106 S102"]
    );
}

#[test]
fn fan_kickstart_replaces_pending_target_for_larger_upshift() {
    let options = options(json!({
        "fan_max_speed": 0,
        "close_fan_the_first_x_layers": 0,
        "overhang_fan_speed": 60,
        "internal_bridge_fan_speed": 90,
        "fan_kickstart": 10.0
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[PrintPathRole::Bridge, PrintPathRole::InternalBridge],
    );

    assert_eq!(fan_lines(&output), vec!["M106 S255", "M106 S229"]);
}

#[test]
fn fan_kickstart_cancels_pending_target_for_fan_off() {
    let options = options(json!({
        "fan_max_speed": 0,
        "close_fan_the_first_x_layers": 0,
        "overhang_fan_speed": 60,
        "fan_kickstart": 10.0
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[PrintPathRole::Bridge, PrintPathRole::SparseInfill],
    );

    assert_eq!(fan_lines(&output), vec!["M106 S255", "M106 S0"]);
}

#[test]
fn invalid_fan_kickstart_reaches_slice_error() {
    let options = options(json!({ "fan_kickstart": -0.1 }));
    let pipeline = role_sequence_pipeline(&options);

    let err = crate::gcode::format_gcode(&pipeline, &options).unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("fan_kickstart"));
}

#[test]
fn zero_fan_speedup_time_preserves_default_role_fan_output() {
    let default_options = options(json!({
        "fan_max_speed": 0,
        "close_fan_the_first_x_layers": 0,
        "overhang_fan_speed": 75
    }));
    let explicit_zero_options = options(json!({
        "fan_max_speed": 0,
        "close_fan_the_first_x_layers": 0,
        "overhang_fan_speed": 75,
        "fan_speedup_time": 0.0
    }));
    let roles = [
        PrintPathRole::SparseInfill,
        PrintPathRole::Bridge,
        PrintPathRole::SparseInfill,
    ];
    let explicit_zero_output = role_sequence_output_with_roles(&explicit_zero_options, &roles);
    let default_output = role_sequence_output_with_roles(&default_options, &roles);

    assert_eq!(
        without_option_count(&explicit_zero_output),
        without_option_count(&default_output)
    );
}

#[test]
fn fan_speedup_time_moves_bridge_upshift_before_previous_same_layer_move() {
    let options = options(json!({
        "fan_max_speed": 0,
        "close_fan_the_first_x_layers": 0,
        "overhang_fan_speed": 75,
        "fan_speedup_time": 0.2
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[
            PrintPathRole::SparseInfill,
            PrintPathRole::Bridge,
            PrintPathRole::SparseInfill,
        ],
    );

    assert_line_after(&output, "M106 S191", ";EXTRUSION:print:sparse_infill:");
    assert_line_before(&output, "M106 S191", ";EXTRUSION:travel:bridge:");
    assert_line_before_last_prefix(&output, "M106 S0", ";EXTRUSION:print:sparse_infill:");
}

#[test]
fn fan_speedup_time_keeps_external_perimeter_override_at_current_move_by_default() {
    let options = options(json!({
        "fan_max_speed": 0,
        "close_fan_the_first_x_layers": 0,
        "overhang_fan_speed": 75,
        "overhang_fan_threshold": "0%",
        "fan_speedup_time": 0.2
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[
            PrintPathRole::SparseInfill,
            PrintPathRole::ExternalPerimeter,
            PrintPathRole::SparseInfill,
        ],
    );

    assert_line_after(&output, "M106 S191", ";EXTRUSION:print:sparse_infill:");
    assert_line_before(&output, "M106 S191", ";EXTRUSION:print:external_perimeter:");
}

#[test]
fn fan_speedup_overhangs_false_moves_external_perimeter_override_before_previous_move() {
    let options = options(json!({
        "fan_max_speed": 0,
        "close_fan_the_first_x_layers": 0,
        "overhang_fan_speed": 75,
        "overhang_fan_threshold": "0%",
        "fan_speedup_time": 0.2,
        "fan_speedup_overhangs": false
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[
            PrintPathRole::SparseInfill,
            PrintPathRole::ExternalPerimeter,
            PrintPathRole::SparseInfill,
        ],
    );

    assert_line_after(&output, "M106 S191", ";EXTRUSION:print:sparse_infill:");
    assert_line_before(
        &output,
        "M106 S191",
        ";EXTRUSION:travel:external_perimeter:",
    );
    assert_line_before_last_prefix(&output, "M106 S0", ";EXTRUSION:print:sparse_infill:");
}

#[test]
fn fan_speedup_time_preserves_kickstart_pulse_before_early_target_restore() {
    let options = options(json!({
        "fan_max_speed": 0,
        "close_fan_the_first_x_layers": 0,
        "overhang_fan_speed": 75,
        "fan_speedup_time": 0.2,
        "fan_kickstart": 0.01
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[
            PrintPathRole::SparseInfill,
            PrintPathRole::Bridge,
            PrintPathRole::SparseInfill,
        ],
    );

    assert_eq!(
        fan_lines(&output),
        vec!["M106 S255", "M106 S191", "M106 S0"]
    );
    assert_line_after(&output, "M106 S255", ";EXTRUSION:print:sparse_infill:");
    assert_line_before(&output, "M106 S255", ";EXTRUSION:travel:bridge:");
    assert_line_after(&output, "M106 S191", ";EXTRUSION:print:bridge:");
    assert_line_before_last_prefix(&output, "M106 S191", ";EXTRUSION:travel:sparse_infill:");
}
