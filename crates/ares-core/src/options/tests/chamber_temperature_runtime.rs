use super::super::*;
use serde_json::{Value, json};

#[test]
fn chamber_temperature_control_defaults_to_disabled_zero() {
    assert_eq!(
        SliceOptions::default()
            .chamber_temperature_control()
            .unwrap(),
        ChamberTemperatureControl::disabled()
    );
}

#[test]
fn chamber_temperature_control_accepts_bool_and_integer_vector_forms() {
    for (activation, temperature, expected) in [
        (
            json!(true),
            json!(45),
            ChamberTemperatureControl::enabled(45),
        ),
        (
            json!("true"),
            json!("46"),
            ChamberTemperatureControl::enabled(46),
        ),
        (
            json!("false;true"),
            json!("44;47"),
            ChamberTemperatureControl::enabled(47),
        ),
        (
            json!("false,true"),
            json!("48,49"),
            ChamberTemperatureControl::enabled(49),
        ),
        (
            json!([false, true]),
            json!([40, 50]),
            ChamberTemperatureControl::enabled(50),
        ),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "activate_chamber_temp_control": activation,
            "chamber_temperature": temperature
        }))
        .unwrap();

        assert_eq!(options.chamber_temperature_control().unwrap(), expected);
    }
}

#[test]
fn chamber_temperature_control_uses_any_activation_and_max_temperature() {
    let options: SliceOptions = serde_json::from_value(json!({
        "activate_chamber_temp_control": [false, true, false],
        "chamber_temperature": [35, 55, 45]
    }))
    .unwrap();

    assert_eq!(
        options.chamber_temperature_control().unwrap(),
        ChamberTemperatureControl::enabled(55)
    );
}

#[test]
fn chamber_temperature_control_defaults_support_chamber_control_to_enabled() {
    let options: SliceOptions = serde_json::from_value(json!({
        "activate_chamber_temp_control": true,
        "chamber_temperature": 45
    }))
    .unwrap();

    assert_eq!(
        options.chamber_temperature_control().unwrap(),
        ChamberTemperatureControl::enabled(45)
    );
}

#[test]
fn chamber_temperature_control_respects_unsupported_printer_chamber_control() {
    let options: SliceOptions = serde_json::from_value(json!({
        "support_chamber_temp_control": false,
        "activate_chamber_temp_control": true,
        "chamber_temperature": [40, 55]
    }))
    .unwrap();

    assert_eq!(
        options.chamber_temperature_control().unwrap(),
        ChamberTemperatureControl::disabled()
    );
    assert_eq!(options.chamber_temperature_values().unwrap(), vec![40, 55]);
    assert_eq!(options.overall_chamber_temperature().unwrap(), 55);
}

#[test]
fn chamber_temperature_control_accepts_explicit_supported_printer_chamber_control() {
    let options: SliceOptions = serde_json::from_value(json!({
        "support_chamber_temp_control": true,
        "activate_chamber_temp_control": [false, true],
        "chamber_temperature": [35, 50]
    }))
    .unwrap();

    assert_eq!(
        options.chamber_temperature_control().unwrap(),
        ChamberTemperatureControl::enabled(50)
    );
}

#[test]
fn chamber_temperature_control_stays_disabled_when_activation_is_false_or_temperature_is_zero() {
    for (activation, temperature) in [
        (json!(false), json!(45)),
        (json!([false, false]), json!([45, 50])),
        (json!(true), json!(0)),
        (json!([true, true]), json!([0, 0])),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "activate_chamber_temp_control": activation,
            "chamber_temperature": temperature
        }))
        .unwrap();

        assert_eq!(
            options.chamber_temperature_control().unwrap(),
            ChamberTemperatureControl::disabled()
        );
    }
}

#[test]
fn chamber_temperature_control_accepts_legacy_chamber_temperatures_alias() {
    let options: SliceOptions = serde_json::from_value(json!({
        "activate_chamber_temp_control": true,
        "chamber_temperatures": [42, 52]
    }))
    .unwrap();

    assert_eq!(
        options.chamber_temperature_control().unwrap(),
        ChamberTemperatureControl::enabled(52)
    );
}

#[test]
fn chamber_temperature_control_rejects_invalid_activation_values() {
    for value in [
        json!(1),
        json!("True"),
        json!(""),
        json!("true;"),
        json!([]),
        json!([true, "false"]),
        json!({"value": true}),
        Value::Null,
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "activate_chamber_temp_control": value,
            "chamber_temperature": 45
        }))
        .unwrap();

        let err = options.chamber_temperature_control().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("activate_chamber_temp_control"));
    }
}

#[test]
fn chamber_temperature_control_rejects_invalid_support_chamber_control_values() {
    for value in [
        json!(1),
        json!("true"),
        json!([]),
        json!([true]),
        json!({"value": true}),
        Value::Null,
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "support_chamber_temp_control": value,
            "activate_chamber_temp_control": true,
            "chamber_temperature": 45
        }))
        .unwrap();

        let err = options.chamber_temperature_control().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("support_chamber_temp_control"));
    }
}

#[test]
fn chamber_temperature_control_rejects_invalid_temperature_values() {
    for value in [
        json!(-1),
        json!(45.5),
        json!("45.5"),
        json!(""),
        json!("45;"),
        json!([]),
        json!(["45", "bad"]),
        json!([45.5]),
        json!({"value": 45}),
        json!(true),
        Value::Null,
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "activate_chamber_temp_control": true,
            "chamber_temperature": value
        }))
        .unwrap();

        let err = options.chamber_temperature_control().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("chamber_temperature"));
    }
}
