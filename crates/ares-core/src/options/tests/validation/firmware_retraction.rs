use crate::{SliceError, SliceOptions};
use serde_json::json;

const FIRMWARE_RETRACTION_SUPPORT_MESSAGE: &str = "--use-firmware-retraction is only supported by Klipper, Marlin, Smoothie, RepRapFirmware, Repetier and Machinekit firmware";
const FIRMWARE_RETRACTION_WIPE_MESSAGE: &str =
    "--use-firmware-retraction is not compatible with --wipe";

#[test]
fn default_firmware_retraction_options_are_valid() {
    let options: SliceOptions = serde_json::from_value(json!({})).unwrap();

    let errors = options.validate_firmware_retraction_options().unwrap();

    assert!(errors.is_empty());
}

#[test]
fn firmware_retraction_disabled_ignores_flavor_and_wipe() {
    let options: SliceOptions = serde_json::from_value(json!({
        "use_firmware_retraction": false,
        "gcode_flavor": "teacup",
        "wipe": [true]
    }))
    .unwrap();

    let errors = options.validate_firmware_retraction_options().unwrap();

    assert!(errors.is_empty());
}

#[test]
fn supported_firmware_retraction_flavors_are_valid() {
    for flavor in [
        "klipper",
        "smoothie",
        "reprap",
        "reprapfirmware",
        "marlin",
        "marlin2",
        "machinekit",
        "repetier",
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "use_firmware_retraction": true,
            "gcode_flavor": flavor,
            "wipe": false
        }))
        .unwrap();

        let errors = options.validate_firmware_retraction_options().unwrap();

        assert!(errors.is_empty(), "{flavor}");
    }
}

#[test]
fn unsupported_firmware_retraction_flavors_are_reported() {
    for flavor in ["teacup", "makerware", "sailfish", "mach3", "no-extrusion"] {
        let options: SliceOptions = serde_json::from_value(json!({
            "use_firmware_retraction": true,
            "gcode_flavor": flavor
        }))
        .unwrap();

        let errors = options.validate_firmware_retraction_options().unwrap();

        assert_eq!(
            errors["use_firmware_retraction"], FIRMWARE_RETRACTION_SUPPORT_MESSAGE,
            "{flavor}"
        );
    }
}

#[test]
fn firmware_retraction_rejects_wipe() {
    let options: SliceOptions = serde_json::from_value(json!({
        "use_firmware_retraction": true,
        "gcode_flavor": "marlin",
        "wipe": [false, true]
    }))
    .unwrap();

    let errors = options.validate_firmware_retraction_options().unwrap();

    assert_eq!(
        errors["use_firmware_retraction"],
        FIRMWARE_RETRACTION_WIPE_MESSAGE
    );
}

#[test]
fn unsupported_flavor_message_remains_when_wipe_also_applies() {
    let options: SliceOptions = serde_json::from_value(json!({
        "use_firmware_retraction": true,
        "gcode_flavor": "teacup",
        "wipe": [true]
    }))
    .unwrap();

    let errors = options.validate_firmware_retraction_options().unwrap();

    assert_eq!(
        errors["use_firmware_retraction"],
        FIRMWARE_RETRACTION_SUPPORT_MESSAGE
    );
}

#[test]
fn unknown_gcode_flavor_is_deferred_to_enum_validation() {
    let options: SliceOptions = serde_json::from_value(json!({
        "use_firmware_retraction": true,
        "gcode_flavor": "unknown-firmware",
        "wipe": false
    }))
    .unwrap();

    let errors = options.validate_firmware_retraction_options().unwrap();

    assert!(errors.is_empty());
}

#[test]
fn invalid_firmware_retraction_types_return_invalid_input() {
    for (key, value) in [
        ("use_firmware_retraction", json!("true")),
        ("gcode_flavor", json!(true)),
        ("wipe", json!([false, "true"])),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            key: value
        }))
        .unwrap();

        let error = options.validate_firmware_retraction_options().unwrap_err();

        assert!(matches!(error, SliceError::InvalidInput(_)));
    }
}

#[test]
fn basic_validation_api_remains_intact() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0
    }))
    .unwrap();

    let errors = options.validate_basic_fdm_options().unwrap();

    assert!(errors["layer_height"].contains("invalid value 0"));
}
