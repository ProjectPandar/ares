use super::super::flush_placeholders::FlushPlaceholders;
use super::super::*;
use crate::SliceError;
use serde_json::{Value, json};

#[test]
fn flush_placeholders_use_orca_defaults_when_missing() {
    let placeholders = SliceOptions::default().flush_placeholders().unwrap();

    assert_eq!(placeholders, FlushPlaceholders::new(vec![2.0], vec![240]));
}

#[test]
fn flush_placeholders_replace_zero_values_and_preserve_non_zero_values() {
    let options: SliceOptions = serde_json::from_value(json!({
        "filament_flush_volumetric_speed": [0, 4.5],
        "filament_max_volumetric_speed": [2, 8],
        "filament_flush_temp": [0, 245],
        "nozzle_temperature_range_high": [260, 270]
    }))
    .unwrap();

    assert_eq!(
        options.flush_placeholders().unwrap(),
        FlushPlaceholders::new(vec![2.0, 4.5], vec![260, 245])
    );
}

#[test]
fn flush_placeholders_accept_supported_vector_forms() {
    for (value, expected) in [
        (json!(6.25), vec![6.25]),
        (json!("6.5"), vec![6.5]),
        (json!("0;7.5"), vec![2.0, 7.5]),
        (json!("0,8.5"), vec![2.0, 8.5]),
        (json!([0, 9.5]), vec![2.0, 9.5]),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "filament_flush_volumetric_speed": value
        }))
        .unwrap();

        assert_eq!(
            options.flush_placeholders().unwrap().flush_volumetric_speeds(),
            expected.as_slice()
        );
    }

    for (value, expected) in [
        (json!(235), vec![235]),
        (json!("236"), vec![236]),
        (json!("0;237"), vec![240, 237]),
        (json!("0,238"), vec![240, 238]),
        (json!([0, 239]), vec![240, 239]),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "filament_flush_temp": value
        }))
        .unwrap();

        assert_eq!(
            options.flush_placeholders().unwrap().flush_temperatures(),
            expected.as_slice()
        );
    }
}

#[test]
fn flush_placeholders_replace_zero_with_fallback_by_index_or_first() {
    let options: SliceOptions = serde_json::from_value(json!({
        "filament_flush_volumetric_speed": [0, 18.5, 0, 0],
        "filament_max_volumetric_speed": [3.5, 4.5],
        "filament_flush_temp": [0, 255, 0, 0],
        "nozzle_temperature_range_high": [245, 260],
    }))
    .unwrap();

    let placeholders = options.flush_placeholders().unwrap();

    assert_eq!(
        placeholders,
        FlushPlaceholders::new(vec![3.5, 18.5, 3.5, 3.5], vec![245, 255, 245, 245])
    );
}

#[test]
fn flush_placeholders_reject_invalid_values_with_key_name() {
    for (key, value) in [
        ("filament_flush_volumetric_speed", json!(-0.1)),
        ("filament_flush_volumetric_speed", json!(200.1)),
        ("filament_flush_volumetric_speed", json!("NaN")),
        ("filament_flush_volumetric_speed", json!("inf")),
        ("filament_flush_volumetric_speed", json!("1;")),
        ("filament_flush_volumetric_speed", json!("")),
        ("filament_flush_volumetric_speed", json!([])),
        ("filament_flush_volumetric_speed", json!([1, "bad"])),
        ("filament_flush_volumetric_speed", json!({"value": 1.0})),
        ("filament_flush_volumetric_speed", json!(true)),
        ("filament_flush_volumetric_speed", Value::Null),
        ("filament_flush_temp", json!(-1)),
        ("filament_flush_temp", json!(1501)),
        ("filament_flush_temp", json!(240.5)),
        ("filament_flush_temp", json!("240.5")),
        ("filament_flush_temp", json!("")),
        ("filament_flush_temp", json!([])),
        ("filament_flush_temp", json!([100, "bad"])),
        ("filament_flush_temp", json!(["240"])),
        ("filament_max_volumetric_speed", json!(-0.1)),
        ("filament_max_volumetric_speed", json!("NaN")),
        ("nozzle_temperature_range_high", json!(1501)),
        ("nozzle_temperature_range_high", json!("bad")),
        ("nozzle_temperature_range_high", json!({"value": 240})),
        ("nozzle_temperature_range_high", Value::Null),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({ key: value })).unwrap();

        let err = options.flush_placeholders().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key), "{key}: {err}");
    }
}
