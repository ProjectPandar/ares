use super::super::temperature_vitrification::TemperatureVitrification;
use super::super::*;
use serde_json::{Value, json};

#[test]
fn temperature_vitrification_defaults_to_orca_softening_temperature() {
    assert_eq!(
        SliceOptions::default().temperature_vitrification().unwrap(),
        TemperatureVitrification::new(100)
    );
}

#[test]
fn temperature_vitrification_accepts_integer_vector_forms_and_uses_minimum() {
    for (value, expected) in [
        (json!(105), 105),
        (json!("106"), 106),
        (json!("107;98"), 98),
        (json!("109,97"), 97),
        (json!([110, 95, 120]), 95),
        (json!(0), 0),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "temperature_vitrification": value
        }))
        .unwrap();

        assert_eq!(
            options.temperature_vitrification().unwrap(),
            TemperatureVitrification::new(expected),
            "{expected}"
        );
    }
}

#[test]
fn temperature_vitrification_rejects_invalid_values() {
    for value in [
        json!(-1),
        json!(100.5),
        json!("100.5"),
        json!([100.5]),
        json!(""),
        json!("100;"),
        json!([]),
        json!(["100", "bad"]),
        json!([["100"]]),
        json!({"value": 100}),
        json!(true),
        Value::Null,
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "temperature_vitrification": value
        }))
        .unwrap();

        let err = options.temperature_vitrification().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("temperature_vitrification"));
    }
}
