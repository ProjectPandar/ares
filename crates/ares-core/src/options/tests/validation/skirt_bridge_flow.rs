use crate::{SliceError, SliceOptions};
use serde_json::json;

#[test]
fn default_skirt_and_bridge_flow_options_are_valid() {
    let options: SliceOptions = serde_json::from_value(json!({})).unwrap();

    let errors = options.validate_skirt_and_bridge_flow_options().unwrap();

    assert!(errors.is_empty());
}

#[test]
fn negative_skirt_height_is_reported() {
    let options: SliceOptions = serde_json::from_value(json!({
        "skirt_height": -1
    }))
    .unwrap();

    let errors = options.validate_skirt_and_bridge_flow_options().unwrap();

    assert_eq!(errors["skirt_height"], "invalid value -1");
}

#[test]
fn bridge_flow_zero_reports_bridge_and_internal_bridge_flow() {
    let options: SliceOptions = serde_json::from_value(json!({
        "bridge_flow": 0
    }))
    .unwrap();

    let errors = options.validate_skirt_and_bridge_flow_options().unwrap();

    assert_eq!(errors["bridge_flow"], "invalid value 0.000000");
    assert_eq!(errors["internal_bridge_flow"], "invalid value 1.000000");
}

#[test]
fn internal_bridge_flow_zero_alone_is_deferred_by_upstream_guard() {
    let options: SliceOptions = serde_json::from_value(json!({
        "bridge_flow": 1,
        "internal_bridge_flow": 0
    }))
    .unwrap();

    let errors = options.validate_skirt_and_bridge_flow_options().unwrap();

    assert!(errors.is_empty());
}

#[test]
fn numeric_string_bridge_flow_values_use_same_predicate() {
    let options: SliceOptions = serde_json::from_value(json!({
        "bridge_flow": "-0.5",
        "internal_bridge_flow": "0.75"
    }))
    .unwrap();

    let errors = options.validate_skirt_and_bridge_flow_options().unwrap();

    assert_eq!(errors["bridge_flow"], "invalid value -0.500000");
    assert_eq!(errors["internal_bridge_flow"], "invalid value 0.750000");
}

#[test]
fn invalid_skirt_and_bridge_flow_types_return_invalid_input() {
    for (key, value) in [
        ("skirt_height", json!(1.5)),
        ("bridge_flow", json!(true)),
        ("internal_bridge_flow", json!("fast")),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            key: value
        }))
        .unwrap();

        let error = options
            .validate_skirt_and_bridge_flow_options()
            .unwrap_err();

        assert!(matches!(error, SliceError::InvalidInput(_)));
    }
}

#[test]
fn existing_validation_apis_remain_intact_after_skirt_bridge_flow_validation() {
    let basic_options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0
    }))
    .unwrap();
    let firmware_options: SliceOptions = serde_json::from_value(json!({
        "use_firmware_retraction": true,
        "gcode_flavor": "unknown-firmware",
        "wipe": false
    }))
    .unwrap();
    let flavor_options: SliceOptions = serde_json::from_value(json!({
        "gcode_flavor": "unknown-firmware"
    }))
    .unwrap();
    let pattern_options: SliceOptions = serde_json::from_value(json!({
        "top_surface_pattern": "gyroid"
    }))
    .unwrap();

    let basic_errors = basic_options.validate_basic_fdm_options().unwrap();
    let firmware_errors = firmware_options
        .validate_firmware_retraction_options()
        .unwrap();
    let flavor_errors = flavor_options.validate_gcode_flavor_option().unwrap();
    let pattern_errors = pattern_options.validate_infill_pattern_options().unwrap();

    assert!(basic_errors["layer_height"].contains("invalid value 0"));
    assert!(firmware_errors.is_empty());
    assert_eq!(
        flavor_errors["gcode_flavor"],
        "invalid value unknown-firmware"
    );
    assert_eq!(
        pattern_errors["top_surface_pattern"],
        "invalid value gyroid"
    );
}
