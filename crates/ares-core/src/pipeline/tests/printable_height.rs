use crate::{SliceError, SliceOptions, run_slicing_pipeline, slice};
use serde_json::json;

use super::square_pyramid_ascii_stl;

#[test]
fn printable_height_rejects_planned_layer_above_machine_height() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "printable_height": 0.3
    }))
    .unwrap();

    let err = run_slicing_pipeline(square_pyramid_ascii_stl(), &options).unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("printable_height"));
    assert!(err.to_string().contains("0.4"));
    assert!(err.to_string().contains("0.3"));
}

#[test]
fn printable_height_accepts_planned_layer_at_machine_height() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "printable_height": "0.4"
    }))
    .unwrap();

    let pipeline = run_slicing_pipeline(square_pyramid_ascii_stl(), &options).unwrap();

    assert_eq!(pipeline.layers().last().unwrap().print_z(), 0.4);
}

#[tokio::test]
async fn public_slice_rejects_printable_height_before_gcode_output() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "machine_start_gcode": ";START",
        "printable_height": 0.3
    }))
    .unwrap();

    let err = slice(square_pyramid_ascii_stl(), options)
        .await
        .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("printable_height"));
}

#[test]
fn printable_height_rejects_invalid_values_without_placeholder_use() {
    for value in [json!(-0.1), json!("abc"), json!(["0.4"])] {
        let options: SliceOptions = serde_json::from_value(json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "printable_height": value
        }))
        .unwrap();

        let err = run_slicing_pipeline(square_pyramid_ascii_stl(), &options).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("printable_height"));
    }
}

#[test]
fn extruder_printable_height_rejects_planned_layer_above_first_extruder_height() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "printable_height": 10.0,
        "extruder_printable_height": [0.3]
    }))
    .unwrap();

    let err = run_slicing_pipeline(square_pyramid_ascii_stl(), &options).unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("extruder_printable_height"));
    assert!(err.to_string().contains("0.4"));
    assert!(err.to_string().contains("0.3"));
}

#[test]
fn extruder_printable_height_accepts_planned_layer_at_first_extruder_height() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "printable_height": 10.0,
        "extruder_printable_height": ["0.4"]
    }))
    .unwrap();

    let pipeline = run_slicing_pipeline(square_pyramid_ascii_stl(), &options).unwrap();

    assert_eq!(pipeline.layers().last().unwrap().print_z(), 0.4);
}

#[test]
fn extruder_printable_height_zero_and_null_keep_global_height_limit() {
    for value in [json!([0]), json!([null]), json!(null), json!([])] {
        let options: SliceOptions = serde_json::from_value(json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "printable_height": 0.4,
            "extruder_printable_height": value
        }))
        .unwrap();

        let pipeline = run_slicing_pipeline(square_pyramid_ascii_stl(), &options).unwrap();

        assert_eq!(pipeline.layers().last().unwrap().print_z(), 0.4);
    }
}

#[test]
fn extruder_printable_height_rejects_invalid_first_values() {
    for value in [json!(-0.1), json!("abc"), json!([1000.1]), json!([false])] {
        let options: SliceOptions = serde_json::from_value(json!({
            "layer_height": 0.2,
            "initial_layer_height": 0.2,
            "printable_height": 10.0,
            "extruder_printable_height": value
        }))
        .unwrap();

        let err = run_slicing_pipeline(square_pyramid_ascii_stl(), &options).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("extruder_printable_height"));
    }
}

#[tokio::test]
async fn public_slice_rejects_extruder_printable_height_before_gcode_output() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "machine_start_gcode": ";START",
        "printable_height": 10.0,
        "extruder_printable_height": [0.3]
    }))
    .unwrap();

    let err = slice(square_pyramid_ascii_stl(), options)
        .await
        .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("extruder_printable_height"));
}
