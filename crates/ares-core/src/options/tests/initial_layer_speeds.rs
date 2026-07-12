use super::super::SliceOptions;
use crate::{PrintPathRole, ToolpathMoveKind};
use serde_json::json;

#[test]
fn defaults_match_orca_initial_layer_speed_definitions() {
    let speeds = SliceOptions::default().speed_options().unwrap();

    assert_eq!(
        speeds.speed_for_layer(ToolpathMoveKind::Print, PrintPathRole::ExternalPerimeter, true),
        30.0
    );
    assert_eq!(
        speeds.speed_for_layer(ToolpathMoveKind::Print, PrintPathRole::SparseInfill, true),
        60.0
    );
    assert_eq!(
        speeds.speed_for_layer(ToolpathMoveKind::Travel, PrintPathRole::ExternalPerimeter, true),
        speeds.speed_for_role(ToolpathMoveKind::Travel, PrintPathRole::ExternalPerimeter)
    );
}

#[test]
fn parses_initial_layer_speed_options() {
    let options: SliceOptions = serde_json::from_value(json!({
        "travel_speed": 120,
        "initial_layer_speed": "25",
        "initial_layer_infill_speed": 35,
        "initial_layer_travel_speed": "50%"
    }))
    .unwrap();
    let speeds = options.speed_options().unwrap();

    assert_eq!(
        speeds.speed_for_layer(ToolpathMoveKind::Print, PrintPathRole::ExternalPerimeter, true),
        25.0
    );
    assert_eq!(
        speeds.speed_for_layer(ToolpathMoveKind::Print, PrintPathRole::SparseInfill, true),
        35.0
    );
    assert_eq!(
        speeds.speed_for_layer(ToolpathMoveKind::Travel, PrintPathRole::ExternalPerimeter, true),
        60.0
    );
}

#[test]
fn parses_numeric_initial_layer_travel_speed() {
    let options: SliceOptions = serde_json::from_value(json!({
        "travel_speed": 120,
        "initial_layer_travel_speed": 45
    }))
    .unwrap();
    let speeds = options.speed_options().unwrap();

    assert_eq!(
        speeds.speed_for_layer(ToolpathMoveKind::Travel, PrintPathRole::ExternalPerimeter, true),
        45.0
    );
}

#[test]
fn rejects_invalid_initial_layer_speed_options() {
    for (key, value) in [
        ("initial_layer_speed", json!(0)),
        ("initial_layer_speed", json!(-1)),
        ("initial_layer_speed", json!("not-a-number")),
        ("initial_layer_speed", json!("NaN")),
        ("initial_layer_speed", json!(true)),
        ("initial_layer_infill_speed", json!(0)),
        ("initial_layer_infill_speed", json!(-1)),
        ("initial_layer_infill_speed", json!("not-a-number")),
        ("initial_layer_infill_speed", json!("NaN")),
        ("initial_layer_infill_speed", json!(true)),
        ("initial_layer_travel_speed", json!(0)),
        ("initial_layer_travel_speed", json!("-1")),
        ("initial_layer_travel_speed", json!("bad%")),
        ("initial_layer_travel_speed", json!("NaN")),
        ("initial_layer_travel_speed", json!("inf")),
        ("initial_layer_travel_speed", json!(true)),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({ key: value })).unwrap();
        assert!(options.speed_options().is_err(), "{key} should be rejected");
    }
}

#[test]
fn parses_travel_speed_z_options() {
    let default_speeds = SliceOptions::default().speed_options().unwrap();
    assert_eq!(default_speeds.travel_speed_z_mm_s(), 0.0);

    let numeric: SliceOptions = serde_json::from_value(json!({
        "travel_speed_z": 25
    }))
    .unwrap();
    assert_eq!(numeric.speed_options().unwrap().travel_speed_z_mm_s(), 25.0);

    let string: SliceOptions = serde_json::from_value(json!({
        "travel_speed_z": "35"
    }))
    .unwrap();
    assert_eq!(string.speed_options().unwrap().travel_speed_z_mm_s(), 35.0);
}

#[test]
fn rejects_invalid_travel_speed_z_options() {
    for value in [
        json!(-1),
        json!("NaN"),
        json!("inf"),
        json!("50%"),
        json!("fast"),
        json!(true),
        json!(null),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "travel_speed_z": value })).unwrap();
        assert!(
            options.speed_options().is_err(),
            "travel_speed_z value should be rejected"
        );
    }
}
