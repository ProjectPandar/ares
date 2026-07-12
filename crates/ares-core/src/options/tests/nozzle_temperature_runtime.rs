use super::super::*;
use crate::options::nozzle_temperature::FirstLayerNozzleTemperature;
use serde_json::{Value, json};

#[test]
fn first_layer_nozzle_temperature_defaults_to_orca_default() {
    assert_eq!(
        SliceOptions::default()
            .first_layer_nozzle_temperature()
            .unwrap(),
        FirstLayerNozzleTemperature::new(200)
    );
}

#[test]
fn first_layer_nozzle_temperature_accepts_integer_forms() {
    for (value, expected) in [
        (json!(215), 215),
        (json!("216"), 216),
        (json!("217;218"), 217),
        (json!("219,220"), 219),
        (json!([221, 222]), 221),
        (json!(0), 0),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "nozzle_temperature_initial_layer": value
        }))
        .unwrap();

        assert_eq!(
            options.first_layer_nozzle_temperature().unwrap(),
            FirstLayerNozzleTemperature::new(expected),
            "{expected}"
        );
    }
}

#[test]
fn first_layer_nozzle_temperature_rejects_invalid_values() {
    for value in [
        json!(-1),
        json!(200.5),
        json!("200.5"),
        json!([200.5]),
        json!(""),
        json!("200;"),
        json!([]),
        json!(["200", "bad"]),
        json!([["200"]]),
        json!({"value": 200}),
        json!(true),
        Value::Null,
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "nozzle_temperature_initial_layer": value
        }))
        .unwrap();

        let err = options.first_layer_nozzle_temperature().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("nozzle_temperature_initial_layer"));
    }
}

#[test]
fn nozzle_temperature_ranges_default_to_orca_values() {
    let options = SliceOptions::default();

    assert_eq!(
        options.nozzle_temperature_range_low_values().unwrap(),
        vec![190]
    );
    assert_eq!(
        options.nozzle_temperature_range_high_values().unwrap(),
        vec![240]
    );
    assert!(options.validate_nozzle_temperature_ranges().is_ok());
}

#[test]
fn nozzle_temperature_ranges_accept_existing_integer_vector_forms() {
    for (low, high) in [
        (json!(180), json!(240)),
        (json!("181"), json!("241")),
        (json!("182;183"), json!("242;243")),
        (json!("184,185"), json!("244,245")),
        (json!([186, 187]), json!([246, 247])),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "nozzle_temperature": [205, 215],
            "nozzle_temperature_range_low": low,
            "nozzle_temperature_range_high": high
        }))
        .unwrap();

        assert!(options.validate_nozzle_temperature_ranges().is_ok());
    }
}

#[test]
fn nozzle_temperature_ranges_reject_invalid_range_order() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_temperature": [200, 205],
        "nozzle_temperature_range_low": [190, 230],
        "nozzle_temperature_range_high": [240, 230]
    }))
    .unwrap();

    let err = options.validate_nozzle_temperature_ranges().unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("nozzle_temperature_range_low"));
    assert!(err.to_string().contains("nozzle_temperature_range_high"));
}

#[test]
fn nozzle_temperature_ranges_reject_mutually_incompatible_temperatures() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_temperature": [200, 260],
        "nozzle_temperature_range_low": [190, 250],
        "nozzle_temperature_range_high": [230, 280]
    }))
    .unwrap();

    let err = options.validate_nozzle_temperature_ranges().unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("nozzle_temperature"));
    assert!(err.to_string().contains("nozzle_temperature_range_low"));
    assert!(err.to_string().contains("nozzle_temperature_range_high"));
}

#[test]
fn nozzle_temperature_ranges_reject_invalid_integer_vector_values() {
    for key in [
        "nozzle_temperature_range_low",
        "nozzle_temperature_range_high",
    ] {
        for value in [
            json!(-1),
            json!(200.5),
            json!("200.5"),
            json!(""),
            json!("200;"),
            json!([]),
            json!(["200"]),
            json!([200.5]),
            json!({"value": 200}),
            json!(true),
            Value::Null,
        ] {
            let mut values = serde_json::Map::new();
            values.insert("nozzle_temperature".to_owned(), json!([200, 205]));
            values.insert("nozzle_temperature_range_low".to_owned(), json!([190, 190]));
            values.insert("nozzle_temperature_range_high".to_owned(), json!([240, 240]));
            values.insert(key.to_owned(), value);
            let options: SliceOptions =
                serde_json::from_value(serde_json::Value::Object(values)).unwrap();

            let err = options.validate_nozzle_temperature_ranges().unwrap_err();

            assert!(matches!(err, SliceError::InvalidInput(_)));
            assert!(err.to_string().contains(key));
        }
    }
}

#[test]
fn nozzle_temperature_ranges_use_first_value_fallback_for_missing_entries() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_temperature": [200, 260],
        "nozzle_temperature_range_low": [190],
        "nozzle_temperature_range_high": [240]
    }))
    .unwrap();

    let err = options.validate_nozzle_temperature_ranges().unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("nozzle_temperature"));
    assert!(err.to_string().contains("nozzle_temperature_range_low"));
    assert!(err.to_string().contains("nozzle_temperature_range_high"));
}

#[test]
fn nozzle_temperature_range_count_uses_non_temperature_vectors() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_temperature": [200],
        "nozzle_temperature_range_low": [190],
        "nozzle_temperature_range_high": [240],
        "filament_type": ["PLA", "PETG"],
        "filament_diameter": "1.75;1.75",
        "nozzle_diameter": ["0.4", "0.6", "0.8"]
    }))
    .unwrap();

    assert_eq!(
        options.nozzle_temperature_range_validation_count(&[200], &[190], &[240]),
        3
    );
}
