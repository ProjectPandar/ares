use super::super::*;
use crate::SliceError;
use serde_json::{Value, json};

#[test]
fn filament_cooling_before_tower_uses_orca_default_when_missing() {
    let values = SliceOptions::default()
        .filament_cooling_before_tower()
        .unwrap();

    assert_eq!(values, vec![10.0]);
}

#[test]
fn filament_cooling_before_tower_accepts_supported_numeric_vector_forms() {
    for (value, expected) in [
        (json!(12), vec![12.0]),
        (json!("13.5"), vec![13.5]),
        (json!("14;7.5"), vec![14.0, 7.5]),
        (json!("15,8.25"), vec![15.0, 8.25]),
        (json!([16, 9.75]), vec![16.0, 9.75]),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "filament_cooling_before_tower": value
        }))
        .unwrap();

        assert_eq!(options.filament_cooling_before_tower().unwrap(), expected);
    }
}

#[test]
fn filament_cooling_before_tower_accepts_nullable_values() {
    for (value, expected) in [
        (Value::Null, "nil"),
        (json!("nil"), "nil"),
        (json!([12, null, "nil", "7.5"]), "12,nil,nil,7.5"),
        (json!("12,nil;7.5"), "12,nil,7.5"),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "filament_cooling_before_tower": value
        }))
        .unwrap();

        assert_eq!(
            options.filament_cooling_before_tower_placeholder().unwrap(),
            expected
        );
    }
}

#[test]
fn filament_cooling_before_tower_all_nil_is_detected_for_exports() {
    for value in [Value::Null, json!("nil"), json!([null, "nil"])] {
        let options: SliceOptions = serde_json::from_value(json!({
            "filament_cooling_before_tower": value
        }))
        .unwrap();

        assert_eq!(
            options
                .filament_cooling_before_tower_config_export()
                .unwrap(),
            None
        );
    }
}

#[test]
fn filament_cooling_before_tower_rejects_invalid_values_with_key_name() {
    for value in [
        json!(-0.1),
        json!("NaN"),
        json!("inf"),
        json!("1;"),
        json!("1,,2"),
        json!(""),
        json!([]),
        json!([1, "bad"]),
        json!([1, -0.1]),
        json!({"value": 1.0}),
        json!(true),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "filament_cooling_before_tower": value
        }))
        .unwrap();

        let err = options.filament_cooling_before_tower().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(
            err.to_string().contains("filament_cooling_before_tower"),
            "{err}"
        );
    }
}
