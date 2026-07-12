use crate::{SliceError, SliceOptions};
use serde_json::json;

#[test]
fn default_gcode_flavor_option_is_valid() {
    let options: SliceOptions = serde_json::from_value(json!({})).unwrap();

    let errors = options.validate_gcode_flavor_option().unwrap();

    assert!(errors.is_empty());
}

#[test]
fn active_gcode_flavor_values_are_valid() {
    for flavor in ["marlin", "klipper", "reprapfirmware", "repetier", "marlin2"] {
        let options: SliceOptions = serde_json::from_value(json!({
            "gcode_flavor": flavor
        }))
        .unwrap();

        let errors = options.validate_gcode_flavor_option().unwrap();

        assert!(errors.is_empty(), "{flavor}");
    }
}

#[test]
fn unknown_gcode_flavor_value_is_reported() {
    let options: SliceOptions = serde_json::from_value(json!({
        "gcode_flavor": "unknown-firmware"
    }))
    .unwrap();

    let errors = options.validate_gcode_flavor_option().unwrap();

    assert_eq!(errors["gcode_flavor"], "invalid value unknown-firmware");
}

#[test]
fn mapped_but_inactive_gcode_flavor_values_are_reported() {
    for flavor in [
        "reprap",
        "teacup",
        "makerware",
        "sailfish",
        "smoothie",
        "mach3",
        "machinekit",
        "no-extrusion",
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "gcode_flavor": flavor
        }))
        .unwrap();

        let errors = options.validate_gcode_flavor_option().unwrap();

        assert_eq!(errors["gcode_flavor"], format!("invalid value {flavor}"));
    }
}

#[test]
fn invalid_gcode_flavor_type_returns_invalid_input() {
    let options: SliceOptions = serde_json::from_value(json!({
        "gcode_flavor": true
    }))
    .unwrap();

    let error = options.validate_gcode_flavor_option().unwrap_err();

    assert!(matches!(error, SliceError::InvalidInput(_)));
}

#[test]
fn basic_and_firmware_retraction_validation_apis_remain_intact() {
    let basic_options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0
    }))
    .unwrap();

    let basic_errors = basic_options.validate_basic_fdm_options().unwrap();

    assert!(basic_errors["layer_height"].contains("invalid value 0"));

    let firmware_options: SliceOptions = serde_json::from_value(json!({
        "use_firmware_retraction": true,
        "gcode_flavor": "unknown-firmware",
        "wipe": false
    }))
    .unwrap();

    let firmware_errors = firmware_options
        .validate_firmware_retraction_options()
        .unwrap();

    assert!(firmware_errors.is_empty());
}
