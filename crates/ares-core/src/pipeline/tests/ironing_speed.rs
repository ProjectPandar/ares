use crate::{
    PrintPathRole, SliceError, SliceOptions, gcode::format_gcode,
    pipeline::test_support::single_path_pipeline,
};
use serde_json::{Value, json};

#[test]
fn omitted_ironing_speed_defaults_to_orca_20_mm_s() {
    let options = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "top_surface_speed": 77,
        "initial_layer_infill_speed": 33,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }));
    let output = output_for_layer(&options, 1);

    assert!(output.contains(";SPEED:print:ironing:1,0:1200"));
    assert!(output.contains("G1 X1 Y0 E"));
    assert!(output.contains(" F1200"));
    assert!(!output.contains(";SPEED:print:ironing:1,0:4620"));
    assert!(!output.contains(";SPEED:print:ironing:1,0:1980"));
}

#[test]
fn ironing_speed_controls_non_first_layer_gcode_feedrate() {
    let options = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "ironing_speed": 15,
        "top_surface_speed": 77,
        "initial_layer_infill_speed": 33,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }));
    let output = output_for_layer(&options, 1);

    assert!(output.contains(";SPEED:print:ironing:1,0:900"));
    assert!(output.contains("G1 X1 Y0 E"));
    assert!(output.contains(" F900"));
    assert!(!output.contains(";SPEED:print:ironing:1,0:4620"));
    assert!(!output.contains(";SPEED:print:ironing:1,0:1980"));
}

#[test]
fn ironing_speed_accepts_numeric_string() {
    let options = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "ironing_speed": "18",
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }));
    let output = output_for_layer(&options, 1);

    assert!(output.contains(";SPEED:print:ironing:1,0:1080"));
    assert!(output.contains(" F1080"));
}

#[test]
fn filament_ironing_speed_overrides_non_first_layer_gcode_feedrate() {
    let options = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "ironing_speed": 15,
        "filament_ironing_speed": [25, 99],
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }));
    let output = output_for_layer(&options, 1);

    assert!(output.contains(";SPEED:print:ironing:1,0:1500"));
    assert!(output.contains(" F1500"));
    assert!(!output.contains(";SPEED:print:ironing:1,0:900"));
}

#[test]
fn filament_ironing_speed_nil_falls_back_to_ironing_speed() {
    let options = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "ironing_speed": 15,
        "filament_ironing_speed": ["nil", 25],
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }));
    let output = output_for_layer(&options, 1);

    assert!(output.contains(";SPEED:print:ironing:1,0:900"));
    assert!(output.contains(" F900"));
    assert!(!output.contains(";SPEED:print:ironing:1,0:1500"));
}

#[test]
fn filament_ironing_speed_accepts_scalar_and_numeric_string_forms() {
    let scalar = output_for_layer(
        &options(json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "ironing_speed": 15,
            "filament_ironing_speed": 22,
            "filament_max_volumetric_speed": 0.0,
            "slow_down_for_layer_cooling": false
        })),
        1,
    );
    let string = output_for_layer(
        &options(json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "ironing_speed": 15,
            "filament_ironing_speed": "24",
            "filament_max_volumetric_speed": 0.0,
            "slow_down_for_layer_cooling": false
        })),
        1,
    );

    assert!(scalar.contains(";SPEED:print:ironing:1,0:1320"));
    assert!(scalar.contains(" F1320"));
    assert!(string.contains(";SPEED:print:ironing:1,0:1440"));
    assert!(string.contains(" F1440"));
}

#[test]
fn first_layer_filament_ironing_speed_preserves_initial_layer_infill_speed() {
    let options = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "ironing_speed": 15,
        "filament_ironing_speed": 25,
        "initial_layer_infill_speed": 33,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }));
    let output = output_for_layer(&options, 0);

    assert!(output.contains(";SPEED:print:ironing:1,0:1980"));
    assert!(output.contains(" F1980"));
    assert!(!output.contains(";SPEED:print:ironing:1,0:1500"));
}

#[test]
fn first_layer_ironing_speed_preserves_initial_layer_infill_speed() {
    let options = options(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "ironing_speed": 15,
        "initial_layer_infill_speed": 33,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    }));
    let output = output_for_layer(&options, 0);

    assert!(output.contains(";SPEED:print:ironing:1,0:1980"));
    assert!(output.contains(" F1980"));
    assert!(!output.contains(";SPEED:print:ironing:1,0:900"));
}

#[test]
fn invalid_ironing_speed_values_reach_slice_error() {
    for value in [
        json!(0),
        json!(-1),
        json!(0.5),
        json!("0"),
        json!("0.5"),
        json!("80%"),
        json!("fast"),
        json!([]),
        json!({"value": 15}),
        json!(true),
        Value::Null,
    ] {
        let options = options(json!({ "ironing_speed": value }));
        let err = options.speed_options().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("ironing_speed"));
    }
}

#[test]
fn invalid_filament_ironing_speed_values_reach_slice_error() {
    for value in [
        json!(0),
        json!(-1),
        json!(0.5),
        json!("0"),
        json!("0.5"),
        json!("NaN"),
        json!("fast"),
        json!([]),
        json!({"value": 15}),
        json!(true),
        Value::Null,
    ] {
        let options = options(json!({ "filament_ironing_speed": value }));
        let err = options.speed_options().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("filament_ironing_speed"));
    }
}

fn output_for_layer(options: &SliceOptions, layer_id: usize) -> String {
    let pipeline = single_path_pipeline(options, PrintPathRole::Ironing, layer_id);
    String::from_utf8(format_gcode(&pipeline, options).unwrap()).unwrap()
}

fn options(value: Value) -> SliceOptions {
    serde_json::from_value(value).unwrap()
}
