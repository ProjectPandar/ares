use super::super::*;
use crate::SliceError;
use serde_json::{Value, json};

#[test]
fn change_filament_gcode_defaults_to_empty_string() {
    assert_eq!(SliceOptions::default().change_filament_gcode().unwrap(), "");
}

#[test]
fn change_filament_gcode_accepts_empty_string() {
    let options: SliceOptions = serde_json::from_value(json!({
        "change_filament_gcode": ""
    }))
    .unwrap();

    assert_eq!(options.change_filament_gcode().unwrap(), "");
}

#[test]
fn change_filament_gcode_accepts_non_empty_string() {
    let options: SliceOptions = serde_json::from_value(json!({
        "change_filament_gcode": "M600"
    }))
    .unwrap();

    assert_eq!(options.change_filament_gcode().unwrap(), "M600");
}

#[test]
fn legacy_tool_change_gcode_is_read_as_change_filament_gcode() {
    let options: SliceOptions = serde_json::from_value(json!({
        "tool_change_gcode": "M600"
    }))
    .unwrap();

    assert_eq!(options.values()["change_filament_gcode"], json!("M600"));
    assert_eq!(options.change_filament_gcode().unwrap(), "M600");
}

#[test]
fn change_filament_gcode_rejects_non_string_values() {
    for value in [
        json!(0),
        json!(true),
        json!([]),
        json!({ "value": "M600" }),
        Value::Null,
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "change_filament_gcode": value })).unwrap();
        let err = options.change_filament_gcode().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("change_filament_gcode"));
    }
}
