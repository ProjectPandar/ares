use super::role_fan_gcode_support::*;
use super::*;

#[test]
fn support_interface_fan_speed_overrides_and_restores_layer_baseline() {
    let options = options(json!({
        "fan_min_speed": 40,
        "fan_max_speed": 40,
        "full_fan_speed_layer": 1,
        "close_fan_the_first_x_layers": 0,
        "support_material_interface_fan_speed": 65
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[
            PrintPathRole::ExternalPerimeter,
            PrintPathRole::SupportMaterialInterface,
            PrintPathRole::SparseInfill,
        ],
    );

    assert_eq!(
        fan_lines(&output),
        vec!["M106 S102", "M106 S166", "M106 S102"]
    );
    assert_line_before(
        &output,
        "M106 S166",
        ";EXTRUSION:print:support_material_interface:",
    );
    assert_line_before_last(&output, "M106 S102", ";EXTRUSION:print:sparse_infill:");
}

#[test]
fn ironing_fan_speed_turns_fan_on_without_baseline_then_restores_off() {
    let options = options(json!({
        "fan_max_speed": 0,
        "close_fan_the_first_x_layers": 0,
        "ironing_fan_speed": 15
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[PrintPathRole::Ironing, PrintPathRole::SparseInfill],
    );

    assert_eq!(fan_lines(&output), vec!["M106 S38", "M106 S0"]);
    assert_line_before(&output, "M106 S38", ";EXTRUSION:print:ironing:");
    assert_line_before(&output, "M106 S0", ";EXTRUSION:print:sparse_infill:");
}

#[test]
fn default_support_interface_and_ironing_fan_speeds_emit_no_override() {
    let options = options(json!({
        "fan_max_speed": 0,
        "close_fan_the_first_x_layers": 0
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[
            PrintPathRole::SupportMaterialInterface,
            PrintPathRole::Ironing,
            PrintPathRole::SparseInfill,
        ],
    );

    assert!(fan_lines(&output).is_empty());
}

#[test]
fn close_fan_first_layers_suppresses_support_interface_and_ironing_overrides() {
    let options = options(json!({
        "fan_max_speed": 0,
        "close_fan_the_first_x_layers": 1,
        "support_material_interface_fan_speed": 65,
        "ironing_fan_speed": 15
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[
            PrintPathRole::SupportMaterialInterface,
            PrintPathRole::Ironing,
            PrintPathRole::SparseInfill,
        ],
    );

    assert!(fan_lines(&output).is_empty());
}

#[test]
fn support_interface_and_ironing_fans_do_not_depend_on_overhang_bridge_fan_gate() {
    let options = options(json!({
        "enable_overhang_bridge_fan": false,
        "fan_max_speed": 0,
        "close_fan_the_first_x_layers": 0,
        "support_material_interface_fan_speed": 65,
        "ironing_fan_speed": 15
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[
            PrintPathRole::SupportMaterialInterface,
            PrintPathRole::SparseInfill,
            PrintPathRole::Ironing,
            PrintPathRole::SparseInfill,
        ],
    );

    assert_eq!(
        fan_lines(&output),
        vec!["M106 S166", "M106 S0", "M106 S38", "M106 S0"]
    );
    assert_line_before(
        &output,
        "M106 S166",
        ";EXTRUSION:print:support_material_interface:",
    );
    assert_line_before_last_prefix(&output, "M106 S38", ";EXTRUSION:print:ironing:");
}

#[test]
fn support_interface_and_ironing_fans_are_not_ramp_scaled() {
    let options = options(json!({
        "fan_max_speed": 0,
        "full_fan_speed_layer": 4,
        "close_fan_the_first_x_layers": 0,
        "overhang_fan_speed": 100,
        "support_material_interface_fan_speed": 80,
        "ironing_fan_speed": 20
    }));
    let output = role_sequence_output_with_roles(
        &options,
        &[
            PrintPathRole::Bridge,
            PrintPathRole::SparseInfill,
            PrintPathRole::SupportMaterialInterface,
            PrintPathRole::SparseInfill,
            PrintPathRole::Ironing,
            PrintPathRole::SparseInfill,
        ],
    );

    assert_eq!(
        fan_lines(&output),
        vec![
            "M106 S63",
            "M106 S0",
            "M106 S204",
            "M106 S0",
            "M106 S51",
            "M106 S0"
        ]
    );
    assert_line_before(&output, "M106 S63", ";EXTRUSION:print:bridge:");
    assert_line_before(
        &output,
        "M106 S204",
        ";EXTRUSION:print:support_material_interface:",
    );
    assert_line_before_last_prefix(&output, "M106 S51", ";EXTRUSION:print:ironing:");
}

#[test]
fn invalid_support_interface_fan_speed_values_reach_slice_error() {
    assert_invalid_role_fan_values("support_material_interface_fan_speed");
}

#[test]
fn invalid_ironing_fan_speed_values_reach_slice_error() {
    assert_invalid_role_fan_values("ironing_fan_speed");
}

fn assert_invalid_role_fan_values(key: &str) {
    for value in [
        json!(-2),
        json!(101),
        json!(1.5),
        json!(""),
        json!("NaN"),
        json!("1.5"),
        json!("55;1.5"),
        json!("40;101"),
        json!([]),
        json!([40, "bad"]),
        json!([40, 1.5]),
        json!([40, 101]),
        json!({"value": 40}),
        json!(true),
        serde_json::Value::Null,
    ] {
        let options = options(json!({ key: value }));
        let pipeline = role_sequence_pipeline(&options);

        let err = crate::gcode::format_gcode(&pipeline, &options).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key));
    }
}
