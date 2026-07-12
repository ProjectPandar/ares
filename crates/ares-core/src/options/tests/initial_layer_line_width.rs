use super::super::SliceOptions;
use crate::{PrintPathRole, SliceError};
use serde_json::json;

#[test]
fn parses_initial_layer_line_width_for_first_layer_extrusion_width() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4],
        "filament_diameter": [2.0],
        "line_width": 0.4,
        "initial_layer_line_width": 0.6
    }))
    .unwrap();

    let extrusion = options.extrusion_options().unwrap();
    let normal = extrusion
        .extrusion_per_mm_for_layer(PrintPathRole::ExternalPerimeter, 0.2, false)
        .unwrap();
    let first = extrusion
        .extrusion_per_mm_for_layer(PrintPathRole::ExternalPerimeter, 0.2, true)
        .unwrap();

    assert!(first > normal);
}

#[test]
fn parses_initial_layer_line_width_percent_over_nozzle() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4],
        "filament_diameter": [2.0],
        "line_width": 0.4,
        "initial_layer_line_width": "150%"
    }))
    .unwrap();
    let percent = options.extrusion_options().unwrap();
    let explicit: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4],
        "filament_diameter": [2.0],
        "line_width": 0.4,
        "initial_layer_line_width": 0.6
    }))
    .unwrap();

    let percent_e = percent
        .extrusion_per_mm_for_layer(PrintPathRole::ExternalPerimeter, 0.2, true)
        .unwrap();
    let explicit_e = explicit
        .extrusion_options()
        .unwrap()
        .extrusion_per_mm_for_layer(PrintPathRole::ExternalPerimeter, 0.2, true)
        .unwrap();

    assert!((percent_e - explicit_e).abs() <= 1e-12);
}

#[test]
fn zero_initial_layer_line_width_keeps_line_width_fallback() {
    let omitted: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4],
        "filament_diameter": [2.0],
        "line_width": 0.4
    }))
    .unwrap();
    let zero: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.4],
        "filament_diameter": [2.0],
        "line_width": 0.4,
        "initial_layer_line_width": 0
    }))
    .unwrap();

    assert_eq!(
        omitted.extrusion_options().unwrap(),
        zero.extrusion_options().unwrap()
    );
}

#[test]
fn rejects_invalid_initial_layer_line_width_values() {
    for value in [
        json!(-0.1),
        json!("not-a-number"),
        json!("NaN"),
        json!("inf"),
        json!("-inf"),
        json!(true),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "initial_layer_line_width": value })).unwrap();

        assert!(matches!(
            options.extrusion_options(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}
