use super::super::*;
use serde_json::json;

#[test]
fn gcode_flavor_defaults_to_marlin_legacy() {
    assert_eq!(
        SliceOptions::default().gcode_flavor().unwrap(),
        GCodeFlavor::MarlinLegacy
    );
}

#[test]
fn gcode_flavor_parses_active_values() {
    for (value, expected) in [
        ("marlin", GCodeFlavor::MarlinLegacy),
        ("klipper", GCodeFlavor::Klipper),
        ("reprapfirmware", GCodeFlavor::RepRapFirmware),
        ("repetier", GCodeFlavor::Repetier),
        ("marlin2", GCodeFlavor::MarlinFirmware),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "gcode_flavor": value
        }))
        .unwrap();

        assert_eq!(options.gcode_flavor().unwrap(), expected, "{value}");
    }
}

#[test]
fn gcode_flavor_rejects_non_string_values() {
    let options: SliceOptions = serde_json::from_value(json!({
        "gcode_flavor": true
    }))
    .unwrap();

    let err = options.gcode_flavor().unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("gcode_flavor must be a string"));
}

#[test]
fn gcode_flavor_rejects_inactive_values() {
    let options: SliceOptions = serde_json::from_value(json!({
        "gcode_flavor": "makerware"
    }))
    .unwrap();

    let err = options.gcode_flavor().unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("invalid value makerware"));
}
