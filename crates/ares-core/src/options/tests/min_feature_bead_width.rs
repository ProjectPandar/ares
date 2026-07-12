use super::super::*;
use serde_json::{json, Value};

#[test]
fn min_feature_and_bead_width_use_orca_defaults() {
    let perimeters = SliceOptions::default().perimeter_options().unwrap();

    assert_eq!(perimeters.min_feature_size_percent(), 25.0);
    assert_eq!(perimeters.initial_layer_min_bead_width_percent(), 85.0);
    assert_eq!(perimeters.min_bead_width_percent(), 85.0);
    assert_eq!(perimeters.min_feature_size_mm(), 0.1);
    assert_eq!(perimeters.initial_layer_min_bead_width_mm(), 0.34);
    assert_eq!(perimeters.min_bead_width_mm(), 0.34);
}

#[test]
fn parses_min_feature_and_bead_width_numbers_and_numeric_strings() {
    let numeric: SliceOptions = serde_json::from_value(json!({
        "min_feature_size": 125,
        "initial_layer_min_bead_width": 150,
        "min_bead_width": 175
    }))
    .unwrap();
    let string: SliceOptions = serde_json::from_value(json!({
        "min_feature_size": "35",
        "initial_layer_min_bead_width": "95",
        "min_bead_width": "105"
    }))
    .unwrap();
    let zero: SliceOptions = serde_json::from_value(json!({
        "min_feature_size": 0,
        "initial_layer_min_bead_width": 0,
        "min_bead_width": 0
    }))
    .unwrap();

    let numeric = numeric.perimeter_options().unwrap();
    assert_eq!(numeric.min_feature_size_percent(), 125.0);
    assert_eq!(numeric.initial_layer_min_bead_width_percent(), 150.0);
    assert_eq!(numeric.min_bead_width_percent(), 175.0);

    let string = string.perimeter_options().unwrap();
    assert_eq!(string.min_feature_size_percent(), 35.0);
    assert_eq!(string.initial_layer_min_bead_width_percent(), 95.0);
    assert_eq!(string.min_bead_width_percent(), 105.0);

    let zero = zero.perimeter_options().unwrap();
    assert_eq!(zero.min_feature_size_percent(), 0.0);
    assert_eq!(zero.initial_layer_min_bead_width_percent(), 0.0);
    assert_eq!(zero.min_bead_width_percent(), 0.0);
}

#[test]
fn converts_min_feature_and_bead_width_percentages_from_minimum_nozzle_diameter() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.6, 0.25, 0.4],
        "min_feature_size": 120,
        "initial_layer_min_bead_width": 160,
        "min_bead_width": 80
    }))
    .unwrap();

    let perimeters = options.perimeter_options().unwrap();

    assert_eq!(perimeters.min_feature_size_mm(), 0.3);
    assert_eq!(perimeters.initial_layer_min_bead_width_mm(), 0.4);
    assert_eq!(perimeters.min_bead_width_mm(), 0.2);
    assert_eq!(perimeters.min_bead_width_mm_for_layer(0), 0.4);
    assert_eq!(perimeters.min_bead_width_mm_for_layer(1), 0.2);
}

#[test]
fn rejects_invalid_min_feature_and_bead_width_values() {
    for key in [
        "min_feature_size",
        "initial_layer_min_bead_width",
        "min_bead_width",
    ] {
        for value in [
            json!(-0.1),
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
