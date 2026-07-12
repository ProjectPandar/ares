use super::super::*;
use crate::SliceError;
use serde_json::json;

#[test]
fn omitted_slow_down_layers_defaults_to_zero() {
    let speeds = SliceOptions::default().speed_options().unwrap();

    assert_eq!(speeds.slow_down_layers(), 0);
}

#[test]
fn dont_slow_down_outer_wall_defaults_to_false() {
    let speeds = SliceOptions::default().speed_options().unwrap();

    assert!(!speeds.dont_slow_down_outer_wall());
}

#[test]
fn layer_time_slowdown_options_use_orca_defaults() {
    let speeds = SliceOptions::default().speed_options().unwrap();

    assert!(speeds.slow_down_for_layer_cooling());
    assert_eq!(speeds.slow_down_layer_time_s(), 5.0);
    assert_eq!(speeds.slow_down_min_speed_mm_s(), 10.0);
}

#[test]
fn layer_time_slowdown_options_accept_scalar_and_first_array_entry() {
    let options: SliceOptions = serde_json::from_value(json!({
        "slow_down_for_layer_cooling": [false, true],
        "slow_down_layer_time": ["12.5", 7],
        "slow_down_min_speed": [3.25, "4"]
    }))
    .unwrap();
    let speeds = options.speed_options().unwrap();

    assert!(!speeds.slow_down_for_layer_cooling());
    assert_eq!(speeds.slow_down_layer_time_s(), 12.5);
    assert_eq!(speeds.slow_down_min_speed_mm_s(), 3.25);

    let scalar: SliceOptions = serde_json::from_value(json!({
        "slow_down_for_layer_cooling": true,
        "slow_down_layer_time": 0,
        "slow_down_min_speed": "0"
    }))
    .unwrap();
    let scalar_speeds = scalar.speed_options().unwrap();

    assert!(scalar_speeds.slow_down_for_layer_cooling());
    assert_eq!(scalar_speeds.slow_down_layer_time_s(), 0.0);
    assert_eq!(scalar_speeds.slow_down_min_speed_mm_s(), 0.0);
}

#[test]
fn layer_time_slowdown_options_reject_invalid_values() {
    for (key, value) in [
        ("slow_down_for_layer_cooling", json!("true")),
        ("slow_down_for_layer_cooling", json!([])),
        ("slow_down_for_layer_cooling", json!([1])),
        ("slow_down_layer_time", json!(-0.1)),
        ("slow_down_layer_time", json!(1000.1)),
        ("slow_down_layer_time", json!("NaN")),
        ("slow_down_layer_time", json!([])),
        ("slow_down_layer_time", json!([false])),
        ("slow_down_min_speed", json!(-0.1)),
        ("slow_down_min_speed", json!("inf")),
        ("slow_down_min_speed", json!([])),
        ("slow_down_min_speed", json!([null])),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({ key: value })).unwrap();
        let err = options.speed_options().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key));
    }
}

#[test]
fn dont_slow_down_outer_wall_accepts_scalar_and_first_array_entry() {
    for (value, expected) in [
        (json!(true), true),
        (json!(false), false),
        (json!([true, false]), true),
        (json!([false, true]), false),
        (json!([true, "ignored"]), true),
        (json!([false, 1]), false),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "dont_slow_down_outer_wall": value })).unwrap();

        assert_eq!(
            options.speed_options().unwrap().dont_slow_down_outer_wall(),
            expected
        );
    }
}

#[test]
fn dont_slow_down_outer_wall_rejects_invalid_values() {
    for value in [
        json!(1),
        json!(0),
        json!("true"),
        json!([]),
        json!([1]),
        json!([null]),
        json!(["true"]),
        json!([{ "value": true }]),
        json!({ "value": true }),
        serde_json::Value::Null,
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "dont_slow_down_outer_wall": value })).unwrap();

        let err = options.speed_options().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("dont_slow_down_outer_wall"));
    }
}

#[test]
fn parsed_slow_down_layers_reaches_speed_options() {
    for (value, expected) in [(json!(0), 0), (json!(1), 1), (json!(5), 5), (json!("7"), 7)] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "slow_down_layers": value })).unwrap();

        let speeds = options.speed_options().unwrap();

        assert_eq!(speeds.slow_down_layers(), expected);
    }
}

#[test]
fn rejects_invalid_slow_down_layers_values() {
    for value in [
        json!(-1),
        json!(1.5),
        json!(1.0),
        json!("1.5"),
        json!("1.0"),
        json!("not-a-number"),
        json!("NaN"),
        json!("inf"),
        json!(4294967296_u64),
        json!(true),
        serde_json::Value::Null,
        json!([]),
        json!({ "value": 1 }),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "slow_down_layers": value })).unwrap();

        let err = options.speed_options().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("slow_down_layers"));
    }
}
