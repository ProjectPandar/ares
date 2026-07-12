use super::super::*;
use serde_json::{Value, json};

#[test]
fn exhaust_fan_control_defaults_to_supported_but_inactive() {
    let control = SliceOptions::default().exhaust_fan_control().unwrap();

    assert_eq!(control.during_print_speed(), None);
    assert_eq!(control.completion_speed(), None);
}

#[test]
fn exhaust_fan_control_accepts_scalar_string_array_and_separated_forms() {
    for (activate, during_active, complete_active, during_speed, complete_speed) in [
        (json!(true), json!(true), json!(true), json!(60), json!(80)),
        (
            json!("true"),
            json!("true"),
            json!("true"),
            json!("61"),
            json!("81"),
        ),
        (
            json!([false, true]),
            json!([true, true]),
            json!([false, true]),
            json!([50, 62]),
            json!([70, 82]),
        ),
        (
            json!("false;true"),
            json!("true;true"),
            json!("false;true"),
            json!("50;63"),
            json!("70;83"),
        ),
        (
            json!("false,true"),
            json!("true,true"),
            json!("false,true"),
            json!("50,64"),
            json!("70,84"),
        ),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "activate_air_filtration": activate,
            "activate_air_filtration_during_print": during_active,
            "activate_air_filtration_on_completion": complete_active,
            "during_print_exhaust_fan_speed": during_speed,
            "complete_print_exhaust_fan_speed": complete_speed
        }))
        .unwrap();
        let control = options.exhaust_fan_control().unwrap();

        assert!(control.during_print_speed().is_some());
        assert!(control.completion_speed().is_some());
    }
}

#[test]
fn exhaust_fan_control_uses_active_indexes_and_max_phase_speed() {
    let options: SliceOptions = serde_json::from_value(json!({
        "activate_air_filtration": [true, false, true],
        "activate_air_filtration_during_print": [true, true, true],
        "activate_air_filtration_on_completion": [false, true, true],
        "during_print_exhaust_fan_speed": [30, 99, 60],
        "complete_print_exhaust_fan_speed": [80, 90, 70]
    }))
    .unwrap();
    let control = options.exhaust_fan_control().unwrap();

    assert_eq!(control.during_print_speed(), Some(60));
    assert_eq!(control.completion_speed(), Some(70));
}

#[test]
fn exhaust_fan_control_uses_last_value_fallback_over_max_vector_domain() {
    let options: SliceOptions = serde_json::from_value(json!({
        "activate_air_filtration": [false, true],
        "activate_air_filtration_during_print": [true],
        "activate_air_filtration_on_completion": [false, true, true],
        "during_print_exhaust_fan_speed": [20, 65, 90],
        "complete_print_exhaust_fan_speed": [75]
    }))
    .unwrap();
    let control = options.exhaust_fan_control().unwrap();

    assert_eq!(control.during_print_speed(), Some(90));
    assert_eq!(control.completion_speed(), Some(75));
}

#[test]
fn exhaust_fan_control_support_false_disables_both_phases() {
    let options: SliceOptions = serde_json::from_value(json!({
        "support_air_filtration": false,
        "activate_air_filtration": true,
        "activate_air_filtration_during_print": true,
        "activate_air_filtration_on_completion": true,
        "during_print_exhaust_fan_speed": 60,
        "complete_print_exhaust_fan_speed": 80
    }))
    .unwrap();
    let control = options.exhaust_fan_control().unwrap();

    assert_eq!(control.during_print_speed(), None);
    assert_eq!(control.completion_speed(), None);
}

#[test]
fn exhaust_fan_control_accepts_support_string_forms() {
    let enabled: SliceOptions = serde_json::from_value(json!({
        "support_air_filtration": "true",
        "activate_air_filtration": true,
        "activate_air_filtration_during_print": true,
        "activate_air_filtration_on_completion": true,
        "during_print_exhaust_fan_speed": 60,
        "complete_print_exhaust_fan_speed": 80
    }))
    .unwrap();
    let disabled: SliceOptions = serde_json::from_value(json!({
        "support_air_filtration": "false",
        "activate_air_filtration": true,
        "activate_air_filtration_during_print": true,
        "activate_air_filtration_on_completion": true,
        "during_print_exhaust_fan_speed": 60,
        "complete_print_exhaust_fan_speed": 80
    }))
    .unwrap();

    assert_eq!(
        enabled.exhaust_fan_control().unwrap().during_print_speed(),
        Some(60)
    );
    assert_eq!(
        enabled.exhaust_fan_control().unwrap().completion_speed(),
        Some(80)
    );
    assert_eq!(
        disabled.exhaust_fan_control().unwrap().during_print_speed(),
        None
    );
    assert_eq!(
        disabled.exhaust_fan_control().unwrap().completion_speed(),
        None
    );
}

#[test]
fn exhaust_fan_control_active_zero_speed_is_emitted() {
    let options: SliceOptions = serde_json::from_value(json!({
        "activate_air_filtration": true,
        "activate_air_filtration_during_print": true,
        "activate_air_filtration_on_completion": true,
        "during_print_exhaust_fan_speed": 0,
        "complete_print_exhaust_fan_speed": 0
    }))
    .unwrap();
    let control = options.exhaust_fan_control().unwrap();

    assert_eq!(control.during_print_speed(), Some(0));
    assert_eq!(control.completion_speed(), Some(0));
}

#[test]
fn exhaust_fan_control_rejects_invalid_bool_values() {
    for (key, value) in [
        ("support_air_filtration", json!([true])),
        ("support_air_filtration", json!("True")),
        ("activate_air_filtration", json!(1)),
        ("activate_air_filtration", json!("true;")),
        ("activate_air_filtration_during_print", json!([])),
        (
            "activate_air_filtration_during_print",
            json!([true, "false"]),
        ),
        (
            "activate_air_filtration_on_completion",
            json!({"value": true}),
        ),
        ("activate_air_filtration_on_completion", Value::Null),
    ] {
        let mut values = serde_json::Map::new();
        values.insert("activate_air_filtration".to_owned(), json!(true));
        values.insert(key.to_owned(), value);
        let options: SliceOptions = serde_json::from_value(Value::Object(values)).unwrap();

        let err = options.exhaust_fan_control().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key));
    }
}

#[test]
fn exhaust_fan_control_rejects_invalid_speed_values() {
    for (key, value) in [
        ("during_print_exhaust_fan_speed", json!(-1)),
        ("during_print_exhaust_fan_speed", json!(101)),
        ("during_print_exhaust_fan_speed", json!("60;")),
        ("during_print_exhaust_fan_speed", json!([])),
        ("during_print_exhaust_fan_speed", json!(["60"])),
        ("complete_print_exhaust_fan_speed", json!(80.5)),
        ("complete_print_exhaust_fan_speed", json!("80.5")),
        ("complete_print_exhaust_fan_speed", json!({"value": 80})),
        ("complete_print_exhaust_fan_speed", Value::Null),
    ] {
        let mut values = serde_json::Map::new();
        values.insert("activate_air_filtration".to_owned(), json!(true));
        values.insert(key.to_owned(), value);
        let options: SliceOptions = serde_json::from_value(Value::Object(values)).unwrap();

        let err = options.exhaust_fan_control().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key));
    }
}
