use crate::{
    PrintPathRole, SliceError, SliceOptions, gcode::format_gcode,
    pipeline::test_support::single_path_pipeline,
};
use serde_json::{Value, json};

#[test]
fn omitted_ironing_flow_defaults_to_orca_10_percent() {
    let default_flow = first_ironing_delta(&output_for_layer(&options(json!({})), 1));
    let full_flow = first_ironing_delta(&output_for_layer(
        &options(json!({ "ironing_flow": 100 })),
        1,
    ));

    assert_delta_eq(default_flow, full_flow * 0.10);
}

#[test]
fn ironing_flow_controls_ironing_gcode_extrusion_delta() {
    let low_flow = first_ironing_delta(&output_for_layer(
        &options(json!({ "ironing_flow": 25 })),
        1,
    ));
    let high_flow = first_ironing_delta(&output_for_layer(
        &options(json!({ "ironing_flow": 50 })),
        1,
    ));

    assert_delta_eq(high_flow, low_flow * 2.0);
}

#[test]
fn ironing_flow_accepts_numeric_string() {
    let string_flow = first_ironing_delta(&output_for_layer(
        &options(json!({ "ironing_flow": "25" })),
        1,
    ));
    let numeric_flow = first_ironing_delta(&output_for_layer(
        &options(json!({ "ironing_flow": 25 })),
        1,
    ));

    assert_delta_eq(string_flow, numeric_flow);
}

#[test]
fn ironing_flow_is_independent_from_top_solid_infill_flow_ratio() {
    let base = first_ironing_delta(&output_for_layer(
        &options(json!({ "ironing_flow": 25 })),
        1,
    ));
    let high_top_flow = first_ironing_delta(&output_for_layer(
        &options(json!({
            "ironing_flow": 25,
            "top_solid_infill_flow_ratio": 1.8
        })),
        1,
    ));

    assert_delta_eq(high_top_flow, base);
}

#[test]
fn filament_ironing_flow_overrides_ironing_flow() {
    let override_flow = first_ironing_delta(&output_for_layer(
        &options(json!({
            "ironing_flow": 10,
            "filament_ironing_flow": [25, 99]
        })),
        1,
    ));
    let explicit_flow = first_ironing_delta(&output_for_layer(
        &options(json!({ "ironing_flow": 25 })),
        1,
    ));
    let fallback_flow = first_ironing_delta(&output_for_layer(
        &options(json!({ "ironing_flow": 10 })),
        1,
    ));

    assert_delta_eq(override_flow, explicit_flow);
    assert!((override_flow - fallback_flow).abs() > 0.000002);
}

#[test]
fn filament_ironing_flow_nil_falls_back_to_ironing_flow() {
    let nil_flow = first_ironing_delta(&output_for_layer(
        &options(json!({
            "ironing_flow": 10,
            "filament_ironing_flow": ["nil", 25]
        })),
        1,
    ));
    let fallback_flow = first_ironing_delta(&output_for_layer(
        &options(json!({ "ironing_flow": 10 })),
        1,
    ));

    assert_delta_eq(nil_flow, fallback_flow);
}

#[test]
fn filament_ironing_flow_accepts_scalar_and_numeric_string_forms() {
    let scalar_flow = first_ironing_delta(&output_for_layer(
        &options(json!({ "filament_ironing_flow": 22 })),
        1,
    ));
    let string_flow = first_ironing_delta(&output_for_layer(
        &options(json!({ "filament_ironing_flow": "22" })),
        1,
    ));
    let explicit_flow = first_ironing_delta(&output_for_layer(
        &options(json!({ "ironing_flow": 22 })),
        1,
    ));

    assert_delta_eq(scalar_flow, explicit_flow);
    assert_delta_eq(string_flow, explicit_flow);
}

#[test]
fn invalid_ironing_flow_values_reach_slice_error() {
    for value in [
        json!(-0.1),
        json!(100.1),
        json!("NaN"),
        json!("101"),
        json!("25%"),
        json!("fast"),
        json!([]),
        json!({"value": 25}),
        json!(true),
        Value::Null,
    ] {
        let options = options(json!({ "ironing_flow": value }));
        let err = options.extrusion_options().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("ironing_flow"));
    }
}

#[test]
fn invalid_filament_ironing_flow_values_reach_slice_error() {
    for value in [
        json!(-0.1),
        json!(100.1),
        json!("NaN"),
        json!("101"),
        json!("25%"),
        json!("fast"),
        json!([]),
        json!({"value": 25}),
        json!(true),
        Value::Null,
    ] {
        let options = options(json!({ "filament_ironing_flow": value }));
        let err = options.extrusion_options().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("filament_ironing_flow"));
    }
}

fn output_for_layer(options: &SliceOptions, layer_id: usize) -> String {
    let pipeline = single_path_pipeline(options, PrintPathRole::Ironing, layer_id);
    String::from_utf8(format_gcode(&pipeline, options).unwrap()).unwrap()
}

fn options(extra: Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "top_surface_line_width": 0.4,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    let extra = extra.as_object().expect("test options must be an object");
    for (key, value_extra) in extra {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}

fn first_ironing_delta(gcode: &str) -> f64 {
    let mut previous_e = 0.0;
    for line in gcode.lines() {
        if let Some(e) = line
            .strip_prefix(";EXTRUSION:print:")
            .and_then(|line| line.rsplit_once(':').map(|(_, e)| e))
            .and_then(|e| e.parse::<f64>().ok())
        {
            if line.starts_with(";EXTRUSION:print:ironing:") {
                return e - previous_e;
            }
            previous_e = e;
        }
    }
    panic!("missing ironing extrusion");
}

fn assert_delta_eq(actual: f64, expected: f64) {
    assert!((actual - expected).abs() <= 0.000002);
}
