use super::super::*;
use serde_json::{Value, json};

#[test]
fn defaults_and_accepts_first_numeric_value() {
    let ramp = SliceOptions::default().part_cooling_fan_ramp().unwrap();
    assert_eq!(ramp.fan_cooling_layer_time_s(), 60.0);

    for value in [
        json!(12.5),
        json!("12.5"),
        json!([12.5, 60.0]),
        json!("12.5;60"),
        json!("12.5,60"),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "fan_cooling_layer_time": value })).unwrap();
        assert_eq!(
            options
                .part_cooling_fan_ramp()
                .unwrap()
                .fan_cooling_layer_time_s(),
            12.5
        );
    }
}

#[test]
fn rejects_invalid_values() {
    for value in [
        json!(-0.1),
        json!(1000.1),
        json!("NaN"),
        json!(""),
        json!("12;"),
        json!([]),
        json!([false]),
        json!({"value": 12}),
        json!(true),
        Value::Null,
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "fan_cooling_layer_time": value })).unwrap();
        let err = options.part_cooling_fan_ramp().unwrap_err();
        assert!(err.to_string().contains("fan_cooling_layer_time"));
    }
}

#[test]
fn reduce_fan_stop_start_frequency_defaults_and_accepts_bool_forms() {
    let default_false: SliceOptions = serde_json::from_value(json!({
        "fan_min_speed": 20,
        "fan_max_speed": 100,
        "fan_cooling_layer_time": 60,
        "close_fan_the_first_x_layers": 0,
        "full_fan_speed_layer": 1
    }))
    .unwrap();
    assert_eq!(
        default_false
            .part_cooling_fan_ramp()
            .unwrap()
            .speed_for_layer_time(0, Some(60.0)),
        Some(0)
    );

    for value in [json!(true), json!([true, false])] {
        let options: SliceOptions = serde_json::from_value(json!({
            "reduce_fan_stop_start_freq": value,
            "fan_min_speed": 20,
            "fan_max_speed": 100,
            "fan_cooling_layer_time": 60,
            "close_fan_the_first_x_layers": 0,
            "full_fan_speed_layer": 1
        }))
        .unwrap();

        assert_eq!(
            options
                .part_cooling_fan_ramp()
                .unwrap()
                .speed_for_layer_time(0, Some(60.0)),
            Some(20)
        );
    }
}

#[test]
fn rejects_invalid_reduce_fan_stop_start_frequency_values() {
    for value in [
        json!("true"),
        json!(1),
        json!([]),
        json!([1]),
        json!({"value": true}),
        Value::Null,
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "reduce_fan_stop_start_freq": value })).unwrap();
        let err = options.part_cooling_fan_ramp().unwrap_err();
        assert!(err.to_string().contains("reduce_fan_stop_start_freq"));
    }
}

#[test]
fn selects_full_interpolated_and_long_layer_baseline() {
    let options: SliceOptions = serde_json::from_value(json!({
        "fan_min_speed": 20,
        "fan_max_speed": 100,
        "slow_down_layer_time": 5,
        "fan_cooling_layer_time": 60,
        "full_fan_speed_layer": 1,
        "close_fan_the_first_x_layers": 0
    }))
    .unwrap();
    let ramp = options.part_cooling_fan_ramp().unwrap();

    assert_eq!(ramp.speed_for_layer_time(0, Some(4.0)), Some(100));
    assert_eq!(ramp.speed_for_layer_time(0, Some(32.5)), Some(60));
    assert_eq!(ramp.speed_for_layer_time(0, Some(60.0)), Some(0));
    assert_eq!(ramp.speed_for_layer(0), Some(100));
}

#[test]
fn reduce_fan_stop_start_frequency_long_layer_minimum_is_ramped() {
    let options: SliceOptions = serde_json::from_value(json!({
        "reduce_fan_stop_start_freq": true,
        "fan_min_speed": 40,
        "fan_max_speed": 100,
        "slow_down_layer_time": 5,
        "fan_cooling_layer_time": 10,
        "full_fan_speed_layer": 4,
        "close_fan_the_first_x_layers": 0
    }))
    .unwrap();

    assert_eq!(
        options
            .part_cooling_fan_ramp()
            .unwrap()
            .speed_for_layer_time(0, Some(10.0)),
        Some(10)
    );
}

#[test]
fn preserves_close_zero_max_and_full_layer_ramp() {
    for (options, expected) in [
        (json!({"close_fan_the_first_x_layers": 1}), None),
        (json!({"fan_max_speed": 0, "close_fan_the_first_x_layers": 0}), None),
        (json!({"full_fan_speed_layer": 4, "close_fan_the_first_x_layers": 0}), Some(25)),
    ] {
        let mut value = json!({
            "fan_min_speed": 20,
            "fan_max_speed": 100,
            "slow_down_layer_time": 5,
            "fan_cooling_layer_time": 60
        });
        for (key, extra) in options.as_object().unwrap() {
            value[key] = extra.clone();
        }
        let options: SliceOptions = serde_json::from_value(value).unwrap();
        assert_eq!(
            options
                .part_cooling_fan_ramp()
                .unwrap()
                .speed_for_layer_time(0, Some(4.0)),
            expected
        );
    }
}
