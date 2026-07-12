use super::super::*;
use serde_json::{json, Value};

#[test]
fn wall_maximum_resolution_and_deviation_use_orca_defaults() {
    let perimeters = SliceOptions::default().perimeter_options().unwrap();

    assert_eq!(perimeters.wall_maximum_resolution_mm(), 0.5);
    assert_eq!(perimeters.wall_maximum_deviation_mm(), 0.025);
}

#[test]
fn parses_wall_maximum_resolution_and_deviation_numbers_and_numeric_strings() {
    let numeric: SliceOptions = serde_json::from_value(json!({
        "wall_maximum_resolution": 0.25,
        "wall_maximum_deviation": 0.035
    }))
    .unwrap();
    let string: SliceOptions = serde_json::from_value(json!({
        "wall_maximum_resolution": "0.125",
        "wall_maximum_deviation": "0.015"
    }))
    .unwrap();
    let boundaries: SliceOptions = serde_json::from_value(json!({
        "wall_maximum_resolution": 0.005,
        "wall_maximum_deviation": 0.05
    }))
    .unwrap();
    let lower_deviation_boundary: SliceOptions = serde_json::from_value(json!({
        "wall_maximum_deviation": 0.005
    }))
    .unwrap();

    let numeric = numeric.perimeter_options().unwrap();
    assert_eq!(numeric.wall_maximum_resolution_mm(), 0.25);
    assert_eq!(numeric.wall_maximum_deviation_mm(), 0.035);

    let string = string.perimeter_options().unwrap();
    assert_eq!(string.wall_maximum_resolution_mm(), 0.125);
    assert_eq!(string.wall_maximum_deviation_mm(), 0.015);

    let boundaries = boundaries.perimeter_options().unwrap();
    assert_eq!(boundaries.wall_maximum_resolution_mm(), 0.005);
    assert_eq!(boundaries.wall_maximum_deviation_mm(), 0.05);

    assert_eq!(
        lower_deviation_boundary
            .perimeter_options()
            .unwrap()
            .wall_maximum_deviation_mm(),
        0.005
    );
}

#[test]
fn accepts_wall_maximum_resolution_upper_boundary() {
    let options: SliceOptions = serde_json::from_value(json!({
        "wall_maximum_resolution": 0.5
    }))
    .unwrap();

    assert_eq!(
        options
            .perimeter_options()
            .unwrap()
            .wall_maximum_resolution_mm(),
        0.5
    );
}

#[test]
fn rejects_invalid_wall_maximum_resolution_and_deviation_values() {
    for (key, below_min, above_max) in [
        ("wall_maximum_resolution", json!(0.004), json!(0.501)),
        ("wall_maximum_deviation", json!(0.004), json!(0.051)),
    ] {
        for value in [
            below_min,
            above_max,
            json!("NaN"),
            json!("invalid"),
            json!(true),
            json!([1]),
            json!({ "value": 1 }),
            json!(null),
        ] {
            assert_invalid_value_mentions_key(key, value);
        }
    }
}

fn assert_invalid_value_mentions_key(key: &str, value: Value) {
    let options: SliceOptions = serde_json::from_value(json!({ key: value })).unwrap();

    assert!(matches!(
        options.perimeter_options(),
        Err(SliceError::InvalidInput(message)) if message.contains(key)
    ));
}
