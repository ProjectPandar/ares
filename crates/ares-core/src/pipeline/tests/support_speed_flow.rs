use crate::{
    ExtrusionRole, PrintPathRole, SliceError, SliceOptions, gcode::format_gcode,
    pipeline::test_support::single_path_pipeline,
};
use serde_json::{Value, json};

mod support_line_width;

#[test]
fn support_speed_controls_non_first_layer_gcode_feedrate() {
    let options = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "support_speed": 37,
        "support_interface_speed": 111,
        "internal_solid_infill_speed": 88,
        "initial_layer_infill_speed": 19,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }));
    let output = output_for_layer(&options, 1);

    assert!(output.contains(";SPEED:print:support_material:1,0:2220"));
    assert!(output.contains("G1 X1 Y0 E"));
    assert!(output.contains(" F2220"));
    assert!(!output.contains(";SPEED:print:support_material:1,0:6660"));
    assert!(!output.contains(";SPEED:print:support_material:1,0:5280"));
    assert!(!output.contains(";SPEED:print:support_material:1,0:1140"));
}

#[test]
fn support_speed_accepts_numeric_string() {
    let options = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "support_speed": "41",
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }));
    let output = output_for_layer(&options, 1);

    assert!(output.contains(";SPEED:print:support_material:1,0:2460"));
    assert!(output.contains(" F2460"));
}

#[test]
fn omitted_support_speed_defaults_to_orca_80_mm_s() {
    let options = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "support_interface_speed": 111,
        "internal_solid_infill_speed": 99,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }));
    let output = output_for_layer(&options, 1);

    assert!(output.contains(";SPEED:print:support_material:1,0:4800"));
    assert!(output.contains(" F4800"));
    assert!(!output.contains(";SPEED:print:support_material:1,0:6660"));
    assert!(!output.contains(";SPEED:print:support_material:1,0:5940"));
}

#[test]
fn first_layer_support_speed_preserves_initial_layer_infill_speed() {
    let options = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "support_speed": 37,
        "initial_layer_infill_speed": 19,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }));
    let output = output_for_layer(&options, 0);

    assert!(output.contains(";SPEED:print:support_material:1,0:1140"));
    assert!(output.contains(" F1140"));
    assert!(!output.contains(";SPEED:print:support_material:1,0:2220"));
}

#[test]
fn support_flow_ratio_changes_extrusion_when_other_flow_ratios_enabled() {
    let low = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "set_other_flow_ratios": true,
        "support_flow_ratio": 0.5,
        "support_interface_flow_ratio": 1.75,
        "internal_solid_infill_flow_ratio": 1.25,
        "slow_down_for_layer_cooling": false
    }));
    let high = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "set_other_flow_ratios": true,
        "support_flow_ratio": 1.5,
        "support_interface_flow_ratio": 0.25,
        "internal_solid_infill_flow_ratio": 0.5,
        "slow_down_for_layer_cooling": false
    }));
    let low_output = output_for_layer(&low, 1);
    let high_output = output_for_layer(&high, 1);

    assert_delta_eq(
        first_extrusion_delta(&high_output),
        first_extrusion_delta(&low_output) * 3.0,
    );
}

#[test]
fn support_flow_ratio_accepts_numeric_string() {
    let low = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "set_other_flow_ratios": true,
        "support_flow_ratio": "0.5",
        "slow_down_for_layer_cooling": false
    }));
    let high = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "set_other_flow_ratios": true,
        "support_flow_ratio": "1.5",
        "slow_down_for_layer_cooling": false
    }));
    let low_output = output_for_layer(&low, 1);
    let high_output = output_for_layer(&high, 1);

    assert_delta_eq(
        first_extrusion_delta(&high_output),
        first_extrusion_delta(&low_output) * 3.0,
    );
}

#[test]
fn support_flow_ratio_is_ignored_when_other_flow_ratios_disabled() {
    let omitted = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "support_flow_ratio": 0.5,
        "slow_down_for_layer_cooling": false
    }));
    let disabled = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "set_other_flow_ratios": false,
        "support_flow_ratio": 1.5,
        "slow_down_for_layer_cooling": false
    }));
    let omitted_output = output_for_layer(&omitted, 1);
    let disabled_output = output_for_layer(&disabled, 1);

    assert_delta_eq(
        first_extrusion_delta(&omitted_output),
        first_extrusion_delta(&disabled_output),
    );
}

#[test]
fn support_filament_changes_support_material_width_metadata_and_e_delta() {
    let first = options(json!({
        "nozzle_diameter": [0.4, 0.8],
        "filament_diameter": [1.75, 2.85],
        "support_filament": 0,
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0,
        "support_line_width": 0,
        "slow_down_for_layer_cooling": false
    }));
    let second = options(json!({
        "nozzle_diameter": [0.4, 0.8],
        "filament_diameter": [1.75, 2.85],
        "support_filament": 2,
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0,
        "support_line_width": 0,
        "slow_down_for_layer_cooling": false
    }));

    let first_pipeline = single_path_pipeline(&first, PrintPathRole::SupportMaterial, 1);
    let second_pipeline = single_path_pipeline(&second, PrintPathRole::SupportMaterial, 1);
    let first_width = first_pipeline.layer_extrusion_moves()[1].moves()[1]
        .effective_line_width_mm()
        .unwrap();
    let second_width = second_pipeline.layer_extrusion_moves()[1].moves()[1]
        .effective_line_width_mm()
        .unwrap();

    assert_delta_eq(first_width, 0.45);
    assert_delta_eq(second_width, 0.9);
    assert!(
        first_extrusion_delta(
            &String::from_utf8(format_gcode(&second_pipeline, &second).unwrap()).unwrap()
        ) < first_extrusion_delta(
            &String::from_utf8(format_gcode(&first_pipeline, &first).unwrap()).unwrap()
        )
    );
}

#[test]
fn first_layer_support_flow_ratio_composes_with_first_layer_flow_ratio() {
    let non_first_layer = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "set_other_flow_ratios": true,
        "support_flow_ratio": 1.5,
        "first_layer_flow_ratio": 0.5,
        "slow_down_for_layer_cooling": false
    }));
    let first_layer = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "set_other_flow_ratios": true,
        "support_flow_ratio": 1.5,
        "first_layer_flow_ratio": 0.5,
        "slow_down_for_layer_cooling": false
    }));
    let non_first_layer_output = output_for_layer(&non_first_layer, 1);
    let first_layer_output = output_for_layer(&first_layer, 0);

    assert_delta_eq(
        first_extrusion_delta(&first_layer_output),
        first_extrusion_delta(&non_first_layer_output) * 0.5,
    );
}

#[test]
fn support_material_maps_to_libslic3r_extrusion_role() {
    assert_eq!(
        ExtrusionRole::from_print_path_role(PrintPathRole::SupportMaterial),
        ExtrusionRole::SupportMaterial
    );
}

#[test]
fn invalid_support_speed_values_reach_slice_error() {
    for value in [
        json!(0),
        json!(-1),
        json!(0.5),
        json!("0"),
        json!("0.5"),
        json!("80%"),
        json!("fast"),
        json!([]),
        json!(true),
        Value::Null,
    ] {
        assert_invalid_option("support_speed", value);
    }
}

#[test]
fn invalid_support_flow_ratio_values_reach_slice_error() {
    for value in [
        json!(-0.1),
        json!(2.1),
        json!("2.1"),
        json!("50%"),
        json!("fast"),
        json!([]),
        json!(true),
        Value::Null,
    ] {
        assert_invalid_option("support_flow_ratio", value);
    }
}

fn output_for_layer(options: &SliceOptions, layer_id: usize) -> String {
    let pipeline = single_path_pipeline(options, PrintPathRole::SupportMaterial, layer_id);
    String::from_utf8(format_gcode(&pipeline, options).unwrap()).unwrap()
}

fn first_extrusion_delta(gcode: &str) -> f64 {
    let mut previous_e = 0.0;
    for original in gcode.lines() {
        let Some(rest) = original.strip_prefix(";EXTRUSION:print:") else {
            continue;
        };
        let Some((role_and_segment, e)) = rest.rsplit_once(':') else {
            continue;
        };
        let Ok(e) = e.parse::<f64>() else {
            continue;
        };
        if role_and_segment.starts_with("support_material:") {
            return e - previous_e;
        }
        previous_e = e;
    }
    panic!("missing support material extrusion");
}

fn assert_invalid_option(key: &str, value: Value) {
    let options = options(json!({ key: value }));
    let err = match key {
        "support_speed" => options.speed_options().unwrap_err(),
        "support_flow_ratio" | "support_line_width" => options.extrusion_options().unwrap_err(),
        _ => unreachable!(),
    };

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains(key));
}

fn assert_delta_eq(actual: f64, expected: f64) {
    assert!((actual - expected).abs() <= 0.000002);
}

fn options(value: Value) -> SliceOptions {
    let mut value = value.as_object().unwrap().clone();
    value
        .entry("enable_support".to_owned())
        .or_insert(json!(true));
    serde_json::from_value(Value::Object(value)).unwrap()
}
