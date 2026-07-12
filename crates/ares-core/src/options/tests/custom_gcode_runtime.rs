use super::super::*;
use serde_json::{Value, json};

#[test]
fn process_change_extrusion_role_gcode_accepts_only_strings() {
    let options: SliceOptions = serde_json::from_value(json!({
        "process_change_extrusion_role_gcode": "; process [extrusion_role]"
    }))
    .unwrap();
    assert_eq!(
        options.process_change_extrusion_role_gcode().unwrap(),
        "; process [extrusion_role]"
    );

    let absent = SliceOptions::default();
    assert_eq!(absent.process_change_extrusion_role_gcode().unwrap(), "");

    for value in [
        json!(7),
        json!(["; invalid"]),
        json!({ "value": "; process" }),
        Value::Null,
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "process_change_extrusion_role_gcode": value }))
                .unwrap();
        let err = options.process_change_extrusion_role_gcode().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(
            err.to_string()
                .contains("process_change_extrusion_role_gcode")
        );
    }
}

#[test]
fn filament_change_extrusion_role_gcode_accepts_strings_and_string_arrays() {
    let string_value: SliceOptions = serde_json::from_value(json!({
        "filament_change_extrusion_role_gcode": "; filament [extrusion_role]"
    }))
    .unwrap();
    assert_eq!(
        string_value.filament_change_extrusion_role_gcode().unwrap(),
        "; filament [extrusion_role]"
    );

    let array_value: SliceOptions = serde_json::from_value(json!({
        "filament_change_extrusion_role_gcode": ["; first filament", "; second filament"]
    }))
    .unwrap();
    assert_eq!(
        array_value.filament_change_extrusion_role_gcode().unwrap(),
        "; first filament"
    );

    let absent = SliceOptions::default();
    assert_eq!(absent.filament_change_extrusion_role_gcode().unwrap(), "");

    for value in [json!(""), json!([]), json!([""])] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "filament_change_extrusion_role_gcode": value }))
                .unwrap();
        assert_eq!(options.filament_change_extrusion_role_gcode().unwrap(), "");
    }

    for value in [
        json!(7),
        json!({ "value": "; filament" }),
        Value::Null,
        json!([7]),
        json!(["", 7]),
        json!(["; first", false]),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "filament_change_extrusion_role_gcode": value }))
                .unwrap();
        let err = options.filament_change_extrusion_role_gcode().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(
            err.to_string()
                .contains("filament_change_extrusion_role_gcode")
        );
    }
}
