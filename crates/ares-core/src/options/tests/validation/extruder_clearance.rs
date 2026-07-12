use crate::{SliceError, SliceOptions};
use serde_json::json;

#[test]
fn default_extruder_clearance_options_are_valid() {
    let options: SliceOptions = serde_json::from_value(json!({})).unwrap();

    let errors = options.validate_extruder_clearance_options().unwrap();

    assert!(errors.is_empty());
}

#[test]
fn invalid_extruder_clearance_values_are_reported() {
    let options: SliceOptions = serde_json::from_value(json!({
        "extruder_clearance_radius": 0,
        "extruder_clearance_height_to_rod": -1.25,
        "extruder_clearance_height_to_lid": 0,
        "nozzle_height": -0.5
    }))
    .unwrap();

    let errors = options.validate_extruder_clearance_options().unwrap();

    assert_eq!(
        errors["extruder_clearance_radius"],
        "invalid value 0.000000"
    );
    assert_eq!(
        errors["extruder_clearance_height_to_rod"],
        "invalid value -1.250000"
    );
    assert_eq!(
        errors["extruder_clearance_height_to_lid"],
        "invalid value 0.000000"
    );
    assert_eq!(errors["nozzle_height"], "invalid value -0.500000");
}

#[test]
fn numeric_string_extruder_clearance_values_use_same_predicate() {
    let options: SliceOptions = serde_json::from_value(json!({
        "extruder_clearance_radius": "-2.5",
        "extruder_clearance_height_to_rod": "40",
        "extruder_clearance_height_to_lid": "120",
        "nozzle_height": "0"
    }))
    .unwrap();

    let errors = options.validate_extruder_clearance_options().unwrap();

    assert_eq!(
        errors["extruder_clearance_radius"],
        "invalid value -2.500000"
    );
    assert_eq!(errors["nozzle_height"], "invalid value 0.000000");
    assert!(!errors.contains_key("extruder_clearance_height_to_rod"));
    assert!(!errors.contains_key("extruder_clearance_height_to_lid"));
}

#[test]
fn invalid_extruder_clearance_types_return_invalid_input() {
    for (key, value) in [
        ("extruder_clearance_radius", json!(true)),
        ("extruder_clearance_height_to_rod", json!("high")),
        ("extruder_clearance_height_to_lid", json!([120])),
        ("nozzle_height", json!({"height": 2.5})),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            key: value
        }))
        .unwrap();

        let error = options.validate_extruder_clearance_options().unwrap_err();

        assert!(matches!(error, SliceError::InvalidInput(_)));
    }
}

#[test]
fn existing_validation_apis_remain_intact_after_extruder_clearance_validation() {
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
    let skirt_bridge_options: SliceOptions = serde_json::from_value(json!({
        "bridge_flow": 0
    }))
    .unwrap();

    let basic_errors = basic_options.validate_basic_fdm_options().unwrap();
    let firmware_errors = firmware_options
        .validate_firmware_retraction_options()
        .unwrap();
    let flavor_errors = flavor_options.validate_gcode_flavor_option().unwrap();
    let pattern_errors = pattern_options.validate_infill_pattern_options().unwrap();
    let skirt_bridge_errors = skirt_bridge_options
        .validate_skirt_and_bridge_flow_options()
        .unwrap();

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
    assert_eq!(skirt_bridge_errors["bridge_flow"], "invalid value 0.000000");
}
