use super::super::*;
use crate::{ExtrusionOptions, PrintPathRole, SliceError};
use serde_json::{Value, json};

#[test]
fn omitted_filament_flow_ratio_keeps_extrusion_unscaled() {
    let options: SliceOptions = serde_json::from_value(json!({
        "line_width": 0.4,
        "filament_diameter": [2.0]
    }))
    .unwrap();
    let extrusion = options.extrusion_options().unwrap();
    let base = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.0, 0.0), 0.0);

    let actual = extrusion
        .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
        .unwrap();
    let expected = base
        .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
        .unwrap();

    assert_eq!(
        (actual * 1_000_000.0).round(),
        (expected * 1_000_000.0).round()
    );
}

#[test]
fn parsed_filament_flow_ratio_reaches_extrusion_options() {
    for (value, expected_ratio) in [
        (json!(0.5), 0.5),
        (json!("0.75"), 0.75),
        (json!([1.5, 0.5]), 1.5),
        (json!("1.25;0.5"), 1.25),
        (json!("1.75,0.5"), 1.75),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "filament_flow_ratio": value,
            "line_width": 0.4,
            "filament_diameter": [2.0]
        }))
        .unwrap();
        let extrusion = options.extrusion_options().unwrap();
        let base = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.0, 0.0), 0.0);

        let actual = extrusion
            .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
            .unwrap();
        let expected = base
            .extrusion_per_mm(PrintPathRole::ExternalPerimeter, 0.2)
            .unwrap()
            * expected_ratio;

        assert_eq!(
            (actual * 1_000_000.0).round(),
            (expected * 1_000_000.0).round()
        );
    }
}

#[test]
fn rejects_invalid_filament_flow_ratio_values() {
    for value in [
        json!(0.0),
        json!(-0.1),
        json!("0"),
        json!("not-a-number"),
        json!("NaN"),
        json!("inf"),
        json!("-inf"),
        json!("1.0;0"),
        json!("1.0;bad"),
        json!(""),
        json!([]),
        json!([1.0, 0.0]),
        json!([1.0, "bad"]),
        json!({"value": 1.0}),
        json!(true),
        Value::Null,
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "filament_flow_ratio": value })).unwrap();

        let err = options.extrusion_options().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("filament_flow_ratio"));
    }
}
