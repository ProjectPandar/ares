use super::super::*;
use serde_json::{json, Value};

#[test]
fn wall_transition_parameters_use_orca_defaults() {
    let perimeters = SliceOptions::default().perimeter_options().unwrap();

    assert_eq!(perimeters.wall_transition_length_percent(), 100.0);
    assert_eq!(perimeters.wall_transition_filter_deviation_percent(), 25.0);
    assert_eq!(perimeters.wall_transition_length_mm(), 0.4);
    assert_eq!(perimeters.wall_transition_filter_deviation_mm(), 0.1);
    assert_eq!(perimeters.wall_transition_angle_degrees(), 10.0);
    assert_eq!(perimeters.wall_distribution_count(), 1);
}

#[test]
fn parses_wall_transition_parameter_numbers_and_numeric_strings() {
    let numeric: SliceOptions = serde_json::from_value(json!({
        "wall_transition_length": 150,
        "wall_transition_filter_deviation": 125,
        "wall_transition_angle": 45,
        "wall_distribution_count": 3
    }))
    .unwrap();
    let string: SliceOptions = serde_json::from_value(json!({
        "wall_transition_length": "175",
        "wall_transition_filter_deviation": "135",
        "wall_transition_angle": "35",
        "wall_distribution_count": "4"
    }))
    .unwrap();

    let numeric = numeric.perimeter_options().unwrap();
    assert_eq!(numeric.wall_transition_length_percent(), 150.0);
    assert_eq!(numeric.wall_transition_filter_deviation_percent(), 125.0);
    assert_eq!(numeric.wall_transition_angle_degrees(), 45.0);
    assert_eq!(numeric.wall_distribution_count(), 3);

    let string = string.perimeter_options().unwrap();
    assert_eq!(string.wall_transition_length_percent(), 175.0);
    assert_eq!(string.wall_transition_filter_deviation_percent(), 135.0);
    assert_eq!(string.wall_transition_angle_degrees(), 35.0);
    assert_eq!(string.wall_distribution_count(), 4);
}

#[test]
fn converts_wall_transition_percentages_from_minimum_nozzle_diameter() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.6, 0.25, 0.4],
        "wall_transition_length": 120,
        "wall_transition_filter_deviation": 80
    }))
    .unwrap();

    let perimeters = options.perimeter_options().unwrap();

    assert_eq!(perimeters.wall_transition_length_percent(), 120.0);
    assert_eq!(perimeters.wall_transition_filter_deviation_percent(), 80.0);
    assert_eq!(perimeters.wall_transition_length_mm(), 0.3);
    assert_eq!(perimeters.wall_transition_filter_deviation_mm(), 0.2);
}

#[test]
fn rejects_invalid_wall_transition_percent_values() {
    for key in [
        "wall_transition_length",
        "wall_transition_filter_deviation",
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

#[test]
fn rejects_invalid_wall_transition_angle_values() {
    for value in [
        json!(0),
        json!(60),
        json!(-1),
        json!("NaN"),
        json!("invalid"),
        json!(true),
        json!([10]),
        json!({ "value": 10 }),
        json!(null),
    ] {
        assert_invalid_value_mentions_key("wall_transition_angle", value);
    }
}

#[test]
fn rejects_invalid_wall_distribution_count_values() {
    for value in [
        json!(0),
        json!(-1),
        json!(1.5),
        json!(2_147_483_648_u64),
        json!("invalid"),
        json!(true),
        json!([1]),
        json!({ "value": 1 }),
        json!(null),
    ] {
        assert_invalid_value_mentions_key("wall_distribution_count", value);
    }
}

fn assert_invalid_value_mentions_key(key: &str, value: Value) {
    let options: SliceOptions = serde_json::from_value(json!({ key: value })).unwrap();

    assert!(matches!(
        options.perimeter_options(),
        Err(SliceError::InvalidInput(message)) if message.contains(key)
    ));
}
