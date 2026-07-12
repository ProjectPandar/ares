use super::super::*;
use serde_json::{Value, json};

#[test]
fn auxiliary_fan_placeholders_default_to_orca_placeholder_values() {
    let placeholders = SliceOptions::default()
        .auxiliary_fan_placeholders()
        .unwrap();

    assert_eq!(placeholders.max_additional_fan(), 0.0);
    assert_eq!(placeholders.first_x_layer_fan_speed(), 0.0);
    assert_eq!(placeholders.close_additional_fan_first_x_layers(), 1);
    assert_eq!(placeholders.additional_fan_full_speed_layer(), 0);
}

#[test]
fn auxiliary_fan_placeholders_parse_first_vector_values() {
    let options: SliceOptions = serde_json::from_value(json!({
        "additional_cooling_fan_speed": [70, 40],
        "first_x_layer_fan_speed": [12.5, 9.0],
        "close_additional_fan_first_x_layers": [3, 1],
        "additional_fan_full_speed_layer": [8, 4]
    }))
    .unwrap();
    let placeholders = options.auxiliary_fan_placeholders().unwrap();

    assert_eq!(placeholders.max_additional_fan(), 70.0);
    assert_eq!(placeholders.first_x_layer_fan_speed(), 12.5);
    assert_eq!(placeholders.close_additional_fan_first_x_layers(), 3);
    assert_eq!(placeholders.additional_fan_full_speed_layer(), 8);
}

#[test]
fn change_extrusion_role_gcode_accepts_only_strings() {
    let options: SliceOptions = serde_json::from_value(json!({
        "change_extrusion_role_gcode": "; role [extrusion_role]"
    }))
    .unwrap();
    assert_eq!(
        options.change_extrusion_role_gcode().unwrap(),
        "; role [extrusion_role]"
    );

    let absent = SliceOptions::default();
    assert_eq!(absent.change_extrusion_role_gcode().unwrap(), "");

    for value in [
        json!(7),
        json!(["; invalid"]),
        json!({ "value": "; role" }),
        Value::Null,
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "change_extrusion_role_gcode": value })).unwrap();
        let err = options.change_extrusion_role_gcode().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("change_extrusion_role_gcode"));
    }
}

#[test]
fn machine_start_gcode_accepts_only_strings() {
    let options: SliceOptions = serde_json::from_value(json!({
        "machine_start_gcode": "M100 [max_additional_fan]"
    }))
    .unwrap();

    assert_eq!(
        options.machine_start_gcode().unwrap(),
        "M100 [max_additional_fan]"
    );

    let options: SliceOptions = serde_json::from_value(json!({
        "machine_start_gcode": ["M100"]
    }))
    .unwrap();
    let err = options.machine_start_gcode().unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("machine_start_gcode"));
}

#[test]
fn machine_end_gcode_accepts_only_strings() {
    let options: SliceOptions = serde_json::from_value(json!({
        "machine_end_gcode": "M104 S0"
    }))
    .unwrap();

    assert_eq!(options.machine_end_gcode().unwrap(), "M104 S0");

    let absent = SliceOptions::default();
    assert_eq!(absent.machine_end_gcode().unwrap(), "");

    let options: SliceOptions = serde_json::from_value(json!({
        "machine_end_gcode": ["M104 S0"]
    }))
    .unwrap();
    let err = options.machine_end_gcode().unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("machine_end_gcode"));
}

#[test]
fn filament_end_gcode_accepts_string_or_string_array() {
    let options: SliceOptions = serde_json::from_value(json!({
        "filament_end_gcode": "; filament end"
    }))
    .unwrap();
    assert_eq!(options.filament_end_gcode().unwrap(), "; filament end");

    let options: SliceOptions = serde_json::from_value(json!({
        "filament_end_gcode": ["; first", "; second"]
    }))
    .unwrap();
    assert_eq!(options.filament_end_gcode().unwrap(), "; first");

    let options: SliceOptions = serde_json::from_value(json!({
        "filament_end_gcode": []
    }))
    .unwrap();
    assert_eq!(options.filament_end_gcode().unwrap(), "");

    let absent = SliceOptions::default();
    assert_eq!(absent.filament_end_gcode().unwrap(), "");

    for value in [
        json!(7),
        json!(["; ok", 7]),
        json!({ "value": "; end" }),
        Value::Null,
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "filament_end_gcode": value })).unwrap();
        let err = options.filament_end_gcode().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("filament_end_gcode"));
    }
}

#[test]
fn filament_start_gcode_accepts_string_or_string_array() {
    let options: SliceOptions = serde_json::from_value(json!({
        "filament_start_gcode": "; filament start"
    }))
    .unwrap();
    assert_eq!(options.filament_start_gcode().unwrap(), "; filament start");

    let options: SliceOptions = serde_json::from_value(json!({
        "filament_start_gcode": ["; first", "; second"]
    }))
    .unwrap();
    assert_eq!(options.filament_start_gcode().unwrap(), "; first");

    let options: SliceOptions = serde_json::from_value(json!({
        "filament_start_gcode": []
    }))
    .unwrap();
    assert_eq!(options.filament_start_gcode().unwrap(), "");

    let absent = SliceOptions::default();
    assert_eq!(absent.filament_start_gcode().unwrap(), "");

    for value in [
        json!(7),
        json!(["; ok", 7]),
        json!({ "value": "; start" }),
        Value::Null,
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "filament_start_gcode": value })).unwrap();
        let err = options.filament_start_gcode().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("filament_start_gcode"));
    }
}

#[test]
fn file_start_gcode_accepts_only_strings() {
    let options: SliceOptions = serde_json::from_value(json!({
        "file_start_gcode": "; time {print_time_sec}"
    }))
    .unwrap();

    assert_eq!(
        options.file_start_gcode().unwrap(),
        "; time {print_time_sec}"
    );

    let absent = SliceOptions::default();
    assert_eq!(absent.file_start_gcode().unwrap(), "");

    let options: SliceOptions = serde_json::from_value(json!({
        "file_start_gcode": ["; invalid"]
    }))
    .unwrap();
    let err = options.file_start_gcode().unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("file_start_gcode"));
}

#[test]
fn before_layer_change_gcode_accepts_only_strings() {
    let options: SliceOptions = serde_json::from_value(json!({
        "before_layer_change_gcode": "; before [layer_z]"
    }))
    .unwrap();

    assert_eq!(
        options.before_layer_change_gcode().unwrap(),
        "; before [layer_z]"
    );

    let absent = SliceOptions::default();
    assert_eq!(absent.before_layer_change_gcode().unwrap(), "");

    let options: SliceOptions = serde_json::from_value(json!({
        "before_layer_change_gcode": ["; invalid"]
    }))
    .unwrap();
    let err = options.before_layer_change_gcode().unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("before_layer_change_gcode"));
}

#[test]
fn time_lapse_gcode_accepts_only_strings() {
    let options: SliceOptions = serde_json::from_value(json!({
        "time_lapse_gcode": "; timelapse {layer_z}"
    }))
    .unwrap();

    assert_eq!(options.time_lapse_gcode().unwrap(), "; timelapse {layer_z}");

    let absent = SliceOptions::default();
    assert_eq!(absent.time_lapse_gcode().unwrap(), "");

    let options: SliceOptions = serde_json::from_value(json!({
        "time_lapse_gcode": ["; invalid"]
    }))
    .unwrap();
    let err = options.time_lapse_gcode().unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("time_lapse_gcode"));
}

#[test]
fn auxiliary_fan_placeholders_reject_invalid_new_inputs() {
    for (key, value) in [
        ("first_x_layer_fan_speed", json!(101)),
        ("close_additional_fan_first_x_layers", json!(-1)),
        ("additional_fan_full_speed_layer", json!("")),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({ key: value })).unwrap();
        let err = options.auxiliary_fan_placeholders().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key));
    }
}
