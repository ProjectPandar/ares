use super::super::*;
use serde_json::{Value, json};

#[test]
fn auxiliary_fan_control_defaults_to_disabled() {
    let control = SliceOptions::default().auxiliary_fan_control().unwrap();

    assert_eq!(control.speed_for_layer(0), None);
    assert_eq!(control.completion_shutdown_speed(), None);
}

#[test]
fn auxiliary_fan_control_requires_machine_capability_gate() {
    let options: SliceOptions = serde_json::from_value(json!({
        "auxiliary_fan": false,
        "additional_cooling_fan_speed": 70,
        "close_additional_fan_first_x_layers": 0
    }))
    .unwrap();
    let control = options.auxiliary_fan_control().unwrap();

    assert_eq!(control.speed_for_layer(0), None);
    assert_eq!(control.completion_shutdown_speed(), None);
}

#[test]
fn auxiliary_fan_control_zero_speed_emits_no_commands() {
    let options: SliceOptions = serde_json::from_value(json!({
        "auxiliary_fan": true,
        "additional_cooling_fan_speed": 0,
        "close_additional_fan_first_x_layers": 0
    }))
    .unwrap();
    let control = options.auxiliary_fan_control().unwrap();

    assert_eq!(control.speed_for_layer(0), None);
    assert_eq!(control.completion_shutdown_speed(), None);
}

#[test]
fn auxiliary_fan_control_accepts_first_integer_speed_forms() {
    for value in [
        json!(70),
        json!("70"),
        json!([70, 20]),
        json!("70;20"),
        json!("70,20"),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "auxiliary_fan": true,
            "additional_cooling_fan_speed": value,
            "close_additional_fan_first_x_layers": 0
        }))
        .unwrap();
        let control = options.auxiliary_fan_control().unwrap();

        assert_eq!(control.speed_for_layer(0), Some(70));
        assert_eq!(control.completion_shutdown_speed(), Some(0));
    }
}

#[test]
fn auxiliary_fan_control_suppresses_layers_before_close_threshold() {
    let options: SliceOptions = serde_json::from_value(json!({
        "auxiliary_fan": true,
        "additional_cooling_fan_speed": 70,
        "close_additional_fan_first_x_layers": 2
    }))
    .unwrap();
    let control = options.auxiliary_fan_control().unwrap();

    assert_eq!(control.speed_for_layer(0), None);
    assert_eq!(control.speed_for_layer(1), None);
    assert_eq!(control.speed_for_layer(2), Some(70));
    assert_eq!(control.completion_shutdown_speed(), Some(0));
}

#[test]
fn auxiliary_fan_control_uses_auxiliary_close_threshold() {
    let options: SliceOptions = serde_json::from_value(json!({
        "auxiliary_fan": true,
        "additional_cooling_fan_speed": 70,
        "close_additional_fan_first_x_layers": 2
    }))
    .unwrap();
    let control = options.auxiliary_fan_control().unwrap();

    assert_eq!(control.speed_for_layer(0), None);
    assert_eq!(control.speed_for_layer(1), None);
    assert_eq!(control.speed_for_layer(2), Some(70));
    assert_eq!(control.completion_shutdown_speed(), Some(0));
}

#[test]
fn auxiliary_fan_control_ignores_part_cooling_close_threshold() {
    let options: SliceOptions = serde_json::from_value(json!({
        "auxiliary_fan": true,
        "additional_cooling_fan_speed": 70,
        "close_fan_the_first_x_layers": 3
    }))
    .unwrap();
    let control = options.auxiliary_fan_control().unwrap();

    assert_eq!(control.speed_for_layer(0), None);
    assert_eq!(control.speed_for_layer(1), Some(70));
}

#[test]
fn auxiliary_fan_control_linearly_ramps_until_full_speed_layer() {
    let options: SliceOptions = serde_json::from_value(json!({
        "auxiliary_fan": true,
        "additional_cooling_fan_speed": 80,
        "close_additional_fan_first_x_layers": 2,
        "additional_fan_full_speed_layer": 5
    }))
    .unwrap();
    let control = options.auxiliary_fan_control().unwrap();

    assert_eq!(control.speed_for_layer(0), None);
    assert_eq!(control.speed_for_layer(1), None);
    assert_eq!(control.speed_for_layer(2), Some(27));
    assert_eq!(control.speed_for_layer(3), Some(53));
    assert_eq!(control.speed_for_layer(4), Some(80));
    assert_eq!(control.speed_for_layer(5), Some(80));
}

#[test]
fn auxiliary_fan_control_uses_first_x_layer_speed_before_close_threshold() {
    let options: SliceOptions = serde_json::from_value(json!({
        "auxiliary_fan": true,
        "first_x_layer_fan_speed": 12.5,
        "additional_cooling_fan_speed": 70,
        "close_additional_fan_first_x_layers": 2
    }))
    .unwrap();
    let control = options.auxiliary_fan_control().unwrap();

    assert_eq!(control.speed_for_layer(0), Some(13));
    assert_eq!(control.speed_for_layer(1), Some(13));
    assert_eq!(control.speed_for_layer(2), Some(70));
    assert_eq!(control.completion_shutdown_speed(), Some(0));
}

#[test]
fn auxiliary_fan_control_ramps_from_first_x_layer_speed_to_additional_speed() {
    let options: SliceOptions = serde_json::from_value(json!({
        "auxiliary_fan": true,
        "first_x_layer_fan_speed": 20,
        "additional_cooling_fan_speed": 80,
        "close_additional_fan_first_x_layers": 2,
        "additional_fan_full_speed_layer": 5
    }))
    .unwrap();
    let control = options.auxiliary_fan_control().unwrap();

    assert_eq!(control.speed_for_layer(0), Some(20));
    assert_eq!(control.speed_for_layer(1), Some(20));
    assert_eq!(control.speed_for_layer(2), Some(40));
    assert_eq!(control.speed_for_layer(3), Some(60));
    assert_eq!(control.speed_for_layer(4), Some(80));
    assert_eq!(control.speed_for_layer(5), Some(80));
}

#[test]
fn auxiliary_fan_control_keeps_default_first_x_zero_silent_before_close_threshold() {
    let options: SliceOptions = serde_json::from_value(json!({
        "auxiliary_fan": true,
        "additional_cooling_fan_speed": 80,
        "close_additional_fan_first_x_layers": 2,
        "additional_fan_full_speed_layer": 5
    }))
    .unwrap();
    let control = options.auxiliary_fan_control().unwrap();

    assert_eq!(control.speed_for_layer(0), None);
    assert_eq!(control.speed_for_layer(1), None);
    assert_eq!(control.speed_for_layer(2), Some(27));
    assert_eq!(control.speed_for_layer(3), Some(53));
    assert_eq!(control.speed_for_layer(4), Some(80));
}

#[test]
fn auxiliary_fan_control_shuts_down_after_first_x_when_additional_speed_is_zero() {
    let options: SliceOptions = serde_json::from_value(json!({
        "auxiliary_fan": true,
        "first_x_layer_fan_speed": 20,
        "additional_cooling_fan_speed": 0,
        "close_additional_fan_first_x_layers": 2
    }))
    .unwrap();
    let control = options.auxiliary_fan_control().unwrap();

    assert_eq!(control.speed_for_layer(0), Some(20));
    assert_eq!(control.speed_for_layer(1), Some(20));
    assert_eq!(control.speed_for_layer(2), Some(0));
    assert_eq!(control.completion_shutdown_speed(), Some(0));
}

#[test]
fn auxiliary_fan_control_does_not_emit_initial_shutdown_when_close_threshold_is_zero() {
    let options: SliceOptions = serde_json::from_value(json!({
        "auxiliary_fan": true,
        "first_x_layer_fan_speed": 20,
        "additional_cooling_fan_speed": 0,
        "close_additional_fan_first_x_layers": 0
    }))
    .unwrap();
    let control = options.auxiliary_fan_control().unwrap();

    assert_eq!(control.speed_for_layer(0), None);
    assert_eq!(control.completion_shutdown_speed(), None);
}

#[test]
fn auxiliary_fan_control_accepts_first_x_layer_speed_first_value_forms() {
    for value in [
        json!(12.5),
        json!("12.5"),
        json!([12.5, 80]),
        json!([12.5, "20"]),
        json!("12.5;80"),
        json!("12.5,80"),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "auxiliary_fan": true,
            "first_x_layer_fan_speed": value,
            "additional_cooling_fan_speed": 70,
            "close_additional_fan_first_x_layers": 2
        }))
        .unwrap();
        let control = options.auxiliary_fan_control().unwrap();

        assert_eq!(control.speed_for_layer(0), Some(13));
    }
}

#[test]
fn auxiliary_fan_control_rejects_invalid_auxiliary_fan_values() {
    for value in [
        json!(1),
        json!("true"),
        json!([true]),
        json!({ "value": true }),
        Value::Null,
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "auxiliary_fan": value,
            "additional_cooling_fan_speed": 70
        }))
        .unwrap();

        let err = options.auxiliary_fan_control().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("auxiliary_fan"));
    }
}

#[test]
fn auxiliary_fan_control_rejects_invalid_additional_cooling_fan_speed_values() {
    for value in [
        json!(-1),
        json!(101),
        json!(70.5),
        json!("70.5"),
        json!(""),
        json!("70;"),
        json!([]),
        json!([70, "20"]),
        json!({ "value": 70 }),
        json!(true),
        Value::Null,
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "auxiliary_fan": true,
            "additional_cooling_fan_speed": value
        }))
        .unwrap();

        let err = options.auxiliary_fan_control().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("additional_cooling_fan_speed"));
    }
}

#[test]
fn auxiliary_fan_control_rejects_invalid_first_x_layer_fan_speed_values() {
    for value in [
        json!(-0.1),
        json!(100.1),
        json!(""),
        json!("12.5;"),
        json!([]),
        json!({ "value": 12.5 }),
        json!(true),
        Value::Null,
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "auxiliary_fan": true,
            "first_x_layer_fan_speed": value,
            "additional_cooling_fan_speed": 70
        }))
        .unwrap();

        let err = options.auxiliary_fan_control().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("first_x_layer_fan_speed"));
    }
}
