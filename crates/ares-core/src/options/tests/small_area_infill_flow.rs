use super::super::*;
use crate::{PrintPathRole, SliceError};
use serde_json::json;

#[test]
fn omitted_small_area_flow_compensation_leaves_extrusion_delta_unscaled() {
    let options: SliceOptions = serde_json::from_value(json!({
        "line_width": 0.4,
        "filament_diameter": [2.0]
    }))
    .unwrap();
    let extrusion = options.extrusion_options().unwrap();

    assert_eq!(
        extrusion.small_area_flow_multiplier_for_segment(
            PrintPathRole::SolidInfill,
            false,
            0.1
        ),
        1.0
    );
}

#[test]
fn enabled_small_area_flow_compensation_uses_default_model_and_default_patterns() {
    let options: SliceOptions = serde_json::from_value(json!({
        "small_area_infill_flow_compensation": true,
        "line_width": 0.4,
        "filament_diameter": [2.0]
    }))
    .unwrap();
    let extrusion = options.extrusion_options().unwrap();

    let multiplier = extrusion.small_area_flow_multiplier_for_segment(
        PrintPathRole::SolidInfill,
        false,
        0.1,
    );

    assert!((multiplier - 0.246996362897).abs() < 1e-9);
}

#[test]
fn small_area_flow_compensation_accepts_string_list_model() {
    let options: SliceOptions = serde_json::from_value(json!({
        "small_area_infill_flow_compensation": true,
        "small_area_infill_flow_compensation_model": [
            "0,0",
            "0.5,0.5",
            "2,1"
        ],
        "line_width": 0.4,
        "filament_diameter": [2.0]
    }))
    .unwrap();
    let extrusion = options.extrusion_options().unwrap();

    assert!(
        extrusion.small_area_flow_multiplier_for_segment(
            PrintPathRole::SolidInfill,
            false,
            0.25
        ) < 0.5
    );
}

#[test]
fn small_area_flow_compensation_accepts_serialized_string_model() {
    let options: SliceOptions = serde_json::from_value(json!({
        "small_area_infill_flow_compensation": true,
        "small_area_infill_flow_compensation_model": "0,0\n0.5,0.5;2,1",
        "line_width": 0.4,
        "filament_diameter": [2.0]
    }))
    .unwrap();
    let extrusion = options.extrusion_options().unwrap();

    assert!(
        extrusion.small_area_flow_multiplier_for_segment(
            PrintPathRole::SolidInfill,
            false,
            0.25
        ) < 0.5
    );
}

#[test]
fn small_area_flow_compensation_ignores_empty_serialized_fragments() {
    let options: SliceOptions = serde_json::from_value(json!({
        "small_area_infill_flow_compensation": true,
        "small_area_infill_flow_compensation_model": "\n0,0;\n;0.5,0.5\n\n2,1;",
        "line_width": 0.4,
        "filament_diameter": [2.0]
    }))
    .unwrap();

    assert!(options.extrusion_options().is_ok());
}

#[test]
fn small_area_flow_compensation_rejects_invalid_bool() {
    for value in [json!("true"), json!(1), json!([])] {
        let options: SliceOptions = serde_json::from_value(json!({
            "small_area_infill_flow_compensation": value
        }))
        .unwrap();

        assert!(matches!(
            options.extrusion_options(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}

#[test]
fn small_area_flow_compensation_rejects_invalid_models() {
    for model in [
        json!(["1,0", "2,1"]),
        json!(["0,0", "", "2,1"]),
        json!(["0,0", "0,0.1", "2,1"]),
        json!(["0,0", "2,0.5", "1,1"]),
        json!(["0,0", "1,0.5", "2,0.4", "3,1"]),
        json!(["0,0", "1,0.5", "2,0.9"]),
        json!(["0,0", "abc,0.5", "2,1"]),
        json!([0, "2,1"]),
        json!({"model": "0,0;2,1"}),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "small_area_infill_flow_compensation_model": model
        }))
        .unwrap();

        assert!(matches!(
            options.extrusion_options(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}
