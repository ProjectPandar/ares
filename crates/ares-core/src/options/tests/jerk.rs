use super::super::*;
use serde_json::json;

#[test]
fn jerk_options_use_orca_defaults() {
    let jerk = SliceOptions::default().jerk_options().unwrap();
    assert_eq!(jerk.default_mm_s, 0.0);
    assert_eq!(jerk.outer_wall_mm_s, 9.0);
    assert_eq!(jerk.inner_wall_mm_s, 9.0);
    assert_eq!(jerk.infill_mm_s, 9.0);
    assert_eq!(jerk.initial_layer_mm_s, 9.0);
    assert_eq!(jerk.travel_mm_s, 12.0);
    assert_eq!(jerk.initial_layer_travel_mm_s, 12.0);
}

#[test]
fn parses_jerk_options() {
    let options: SliceOptions = serde_json::from_value(json!({
        "default_jerk": "8",
        "outer_wall_jerk": 7,
        "inner_wall_jerk": "4",
        "infill_jerk": 5,
        "initial_layer_jerk": 6,
        "travel_jerk": 11,
        "initial_layer_travel_jerk": "50%"
    }))
    .unwrap();

    let jerk = options.jerk_options().unwrap();

    assert_eq!(jerk.default_mm_s, 8.0);
    assert_eq!(jerk.outer_wall_mm_s, 7.0);
    assert_eq!(jerk.inner_wall_mm_s, 4.0);
    assert_eq!(jerk.infill_mm_s, 5.0);
    assert_eq!(jerk.initial_layer_mm_s, 6.0);
    assert_eq!(jerk.travel_mm_s, 11.0);
    assert_eq!(jerk.initial_layer_travel_mm_s, 5.5);
}

#[test]
fn default_junction_deviation_uses_orca_default() {
    assert_eq!(
        SliceOptions::default().default_junction_deviation().unwrap(),
        0.0
    );
}

#[test]
fn parses_default_junction_deviation_numbers_and_numeric_strings() {
    let numeric: SliceOptions =
        serde_json::from_value(json!({ "default_junction_deviation": 0.025 })).unwrap();
    let string: SliceOptions =
        serde_json::from_value(json!({ "default_junction_deviation": "0.125" })).unwrap();

    assert_eq!(numeric.default_junction_deviation().unwrap(), 0.025);
    assert_eq!(string.default_junction_deviation().unwrap(), 0.125);
}

#[test]
fn rejects_invalid_default_junction_deviation_values() {
    for value in [
        json!(-0.001),
        json!(0.301),
        json!("bad"),
        json!("inf"),
        json!(true),
        json!(null),
        json!([0.01]),
        json!({ "value": 0.01 }),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "default_junction_deviation": value })).unwrap();

        let err = options.default_junction_deviation().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("default_junction_deviation"));
    }
}
