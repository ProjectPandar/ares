use super::super::*;
use crate::options::part_cooling_fan::InternalBridgeFanSpeed;
use serde_json::{Value, json};

#[test]
fn part_cooling_fan_ramp_defaults_to_full_speed_first_layer() {
    let ramp = SliceOptions::default().part_cooling_fan_ramp().unwrap();

    assert_eq!(ramp.speed_for_layer(0), None);
    assert_eq!(ramp.speed_for_layer(1), Some(100));
}

#[test]
fn part_cooling_fan_ramp_accepts_scalar_string_array_and_separated_forms() {
    for (min_speed, max_speed, full_layer, expected) in [
        (json!(25), json!(75), json!(3), [8, 33, 75]),
        (json!("25"), json!("75"), json!("3"), [8, 33, 75]),
        (json!([25, 99]), json!([75, 88]), json!([3, 4]), [8, 33, 75]),
        (json!("25;99"), json!("75;88"), json!("3;4"), [8, 33, 75]),
        (json!("25,99"), json!("75,88"), json!("3,4"), [8, 33, 75]),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "fan_min_speed": min_speed,
            "fan_max_speed": max_speed,
            "full_fan_speed_layer": full_layer,
            "close_fan_the_first_x_layers": 0
        }))
        .unwrap();
        let ramp = options.part_cooling_fan_ramp().unwrap();

        assert_eq!(
            [
                ramp.speed_for_layer(0).unwrap(),
                ramp.speed_for_layer(1).unwrap(),
                ramp.speed_for_layer(2).unwrap()
            ],
            expected
        );
    }
}

#[test]
fn part_cooling_fan_ramp_zero_max_speed_suppresses_commands() {
    let options: SliceOptions = serde_json::from_value(json!({
        "fan_min_speed": 20,
        "fan_max_speed": 0,
        "full_fan_speed_layer": 3
    }))
    .unwrap();
    let ramp = options.part_cooling_fan_ramp().unwrap();

    assert_eq!(ramp.speed_for_layer(0), None);
    assert_eq!(ramp.speed_for_layer(1), None);
}

#[test]
fn part_cooling_fan_ramp_full_speed_layer_one_uses_max_on_first_layer() {
    let options: SliceOptions = serde_json::from_value(json!({
        "fan_min_speed": 25,
        "fan_max_speed": 75,
        "full_fan_speed_layer": 1,
        "close_fan_the_first_x_layers": 0
    }))
    .unwrap();
    let ramp = options.part_cooling_fan_ramp().unwrap();

    assert_eq!(ramp.speed_for_layer(0), Some(75));
    assert_eq!(ramp.speed_for_layer(1), Some(75));
}

#[test]
fn part_cooling_fan_ramp_normalizes_min_speed_above_max_speed() {
    let options: SliceOptions = serde_json::from_value(json!({
        "fan_min_speed": 90,
        "fan_max_speed": 40,
        "full_fan_speed_layer": 4,
        "close_fan_the_first_x_layers": 0
    }))
    .unwrap();
    let ramp = options.part_cooling_fan_ramp().unwrap();

    assert_eq!(ramp.speed_for_layer(0), Some(10));
    assert_eq!(ramp.speed_for_layer(1), Some(20));
    assert_eq!(ramp.speed_for_layer(3), Some(40));
}

#[test]
fn part_cooling_fan_ramp_suppresses_configured_first_layers() {
    let options: SliceOptions = serde_json::from_value(json!({
        "fan_min_speed": 20,
        "fan_max_speed": 60,
        "full_fan_speed_layer": 4,
        "close_fan_the_first_x_layers": 1
    }))
    .unwrap();
    let ramp = options.part_cooling_fan_ramp().unwrap();

    assert_eq!(
        [
            ramp.speed_for_layer(0),
            ramp.speed_for_layer(1),
            ramp.speed_for_layer(2),
            ramp.speed_for_layer(3)
        ],
        [None, Some(11), Some(31), Some(60)]
    );
}

#[test]
fn part_cooling_fan_ramp_uses_max_after_close_threshold_when_full_speed_is_not_later() {
    let options: SliceOptions = serde_json::from_value(json!({
        "fan_min_speed": 20,
        "fan_max_speed": 60,
        "full_fan_speed_layer": 1,
        "close_fan_the_first_x_layers": 2
    }))
    .unwrap();
    let ramp = options.part_cooling_fan_ramp().unwrap();

    assert_eq!(
        [
            ramp.speed_for_layer(0),
            ramp.speed_for_layer(1),
            ramp.speed_for_layer(2)
        ],
        [None, None, Some(60)]
    );
}

#[test]
fn part_cooling_fan_ramp_rejects_invalid_percent_values() {
    for (key, value) in [
        ("fan_min_speed", json!(-1)),
        ("fan_min_speed", json!(101)),
        ("fan_min_speed", json!("")),
        ("fan_min_speed", json!("20;")),
        ("fan_min_speed", json!([])),
        ("fan_min_speed", json!(["20"])),
        ("fan_min_speed", json!(true)),
        ("fan_min_speed", Value::Null),
        ("fan_max_speed", json!(-1)),
        ("fan_max_speed", json!(101)),
        ("fan_max_speed", json!("")),
        ("fan_max_speed", json!("75;")),
        ("fan_max_speed", json!([])),
        ("fan_max_speed", json!(["75"])),
        ("fan_max_speed", json!({"value": 75})),
        ("fan_max_speed", Value::Null),
    ] {
        let mut options = serde_json::Map::new();
        options.insert("fan_min_speed".to_owned(), json!(20));
        options.insert("fan_max_speed".to_owned(), json!(100));
        options.insert("full_fan_speed_layer".to_owned(), json!(0));
        options.insert(key.to_owned(), value);
        let options: SliceOptions = serde_json::from_value(Value::Object(options)).unwrap();

        let err = options.part_cooling_fan_ramp().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key));
    }
}

#[test]
fn part_cooling_fan_ramp_rejects_invalid_full_speed_layer_values() {
    for value in [
        json!(-1),
        json!(1.5),
        json!("1.5"),
        json!(""),
        json!("3;"),
        json!([]),
        json!(["3"]),
        json!({"value": 3}),
        json!(true),
        Value::Null,
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "fan_min_speed": 20,
            "fan_max_speed": 100,
            "full_fan_speed_layer": value
        }))
        .unwrap();

        let err = options.part_cooling_fan_ramp().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("full_fan_speed_layer"));
    }
}

#[test]
fn part_cooling_fan_ramp_rejects_invalid_close_fan_first_layer_values() {
    for value in [
        json!(-1),
        json!(1.5),
        json!("1.5"),
        json!(""),
        json!("3;"),
        json!([]),
        json!(["3"]),
        json!({"value": 3}),
        json!(true),
        Value::Null,
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "fan_min_speed": 20,
            "fan_max_speed": 100,
            "full_fan_speed_layer": 0,
            "close_fan_the_first_x_layers": value
        }))
        .unwrap();

        let err = options.part_cooling_fan_ramp().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("close_fan_the_first_x_layers"));
    }
}

#[test]
fn part_cooling_fan_min_pwm_defaults_to_zero() {
    let options = SliceOptions::default();

    assert_eq!(options.part_cooling_fan_min_pwm().unwrap(), 0);
}

#[test]
fn part_cooling_fan_min_pwm_accepts_scalar_integer_percent() {
    for (value, expected) in [
        (json!(0), 0),
        (json!(30), 30),
        (json!(30.0), 30),
        (json!(100), 100),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "part_cooling_fan_min_pwm": value })).unwrap();

        assert_eq!(options.part_cooling_fan_min_pwm().unwrap(), expected);
    }
}

#[test]
fn part_cooling_fan_min_pwm_rejects_invalid_values() {
    for value in [
        json!(-1),
        json!(101),
        json!(1.5),
        json!("30"),
        json!([30]),
        json!({"value": 30}),
        json!(true),
        Value::Null,
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "part_cooling_fan_min_pwm": value })).unwrap();

        let err = options.part_cooling_fan_min_pwm().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("part_cooling_fan_min_pwm"));
    }
}

#[test]
fn fan_kickstart_defaults_to_zero() {
    let ramp = SliceOptions::default().part_cooling_fan_ramp().unwrap();

    assert_eq!(ramp.fan_kickstart_s(), 0.0);
}

#[test]
fn fan_kickstart_accepts_non_negative_scalar_seconds() {
    for (value, expected) in [
        (json!(0), 0.0),
        (json!(0.25), 0.25),
        (json!(2), 2.0),
        (json!(3600.5), 3600.5),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "fan_kickstart": value })).unwrap();

        assert_eq!(
            options.part_cooling_fan_ramp().unwrap().fan_kickstart_s(),
            expected
        );
    }
}

#[test]
fn fan_kickstart_rejects_invalid_values() {
    for value in [
        json!(-0.1),
        json!("0.1"),
        json!([0.1]),
        json!({"value": 0.1}),
        json!(true),
        Value::Null,
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "fan_kickstart": value })).unwrap();

        let err = options.part_cooling_fan_ramp().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("fan_kickstart"));
    }
}

#[test]
fn internal_bridge_fan_speed_defaults_to_overhang_fallback() {
    let options = SliceOptions::default();

    assert_eq!(
        options.internal_bridge_fan_speed().unwrap(),
        InternalBridgeFanSpeed::OverhangFallback
    );
}

#[test]
fn internal_bridge_fan_speed_accepts_first_percent_value() {
    for (value, expected) in [
        (json!(-1), InternalBridgeFanSpeed::OverhangFallback),
        (json!(0), InternalBridgeFanSpeed::Fixed(0)),
        (json!(35), InternalBridgeFanSpeed::Fixed(35)),
        (json!(100), InternalBridgeFanSpeed::Fixed(100)),
        (json!([42, 84]), InternalBridgeFanSpeed::Fixed(42)),
        (json!("55;75"), InternalBridgeFanSpeed::Fixed(55)),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "internal_bridge_fan_speed": value })).unwrap();

        assert_eq!(options.internal_bridge_fan_speed().unwrap(), expected);
    }
}

#[test]
fn internal_bridge_fan_speed_rejects_invalid_values() {
    for value in [
        json!(-2),
        json!(101),
        json!(1.5),
        json!(""),
        json!("1.5"),
        json!("55;1.5"),
        json!("40;101"),
        json!([]),
        json!([40, "bad"]),
        json!([40, 1.5]),
        json!([40, 101]),
        json!({"value": 40}),
        json!(true),
        Value::Null,
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "internal_bridge_fan_speed": value })).unwrap();

        let err = options.internal_bridge_fan_speed().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("internal_bridge_fan_speed"));
    }
}
