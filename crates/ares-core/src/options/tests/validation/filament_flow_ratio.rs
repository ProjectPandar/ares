use crate::{SliceError, SliceOptions};
use serde_json::json;

#[test]
fn default_filament_flow_ratio_is_valid() {
    let options: SliceOptions = serde_json::from_value(json!({})).unwrap();

    let errors = options.validate_filament_flow_ratio_options().unwrap();

    assert!(errors.is_empty());
}

#[test]
fn invalid_filament_flow_ratio_entries_are_reported() {
    for (value, message) in [
        (json!(0), "invalid value 0"),
        (json!([1.0, -0.25, 0.8]), "invalid value 1,-0.25,0.8"),
        (json!("1;0;0.5"), "invalid value 1,0,0.5"),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "filament_flow_ratio": value
        }))
        .unwrap();

        let errors = options.validate_filament_flow_ratio_options().unwrap();

        assert_eq!(errors["filament_flow_ratio"], message);
        assert_eq!(errors.len(), 1);
    }
}

#[test]
fn accepted_filament_flow_ratio_vector_forms_use_same_predicate() {
    for value in [
        json!(1.2),
        json!("1.2"),
        json!(["1.2", 0.8]),
        json!("1.2,0.8"),
        json!("1.2;0.8"),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "filament_flow_ratio": value
        }))
        .unwrap();

        let errors = options.validate_filament_flow_ratio_options().unwrap();

        assert!(errors.is_empty());
    }
}

#[test]
fn invalid_filament_flow_ratio_types_return_invalid_input() {
    for value in [
        json!(true),
        json!({"ratio": 1}),
        json!([]),
        json!([1, false]),
        json!("1,,0.5"),
        json!("fast"),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "filament_flow_ratio": value
        }))
        .unwrap();

        let error = options.validate_filament_flow_ratio_options().unwrap_err();

        assert!(matches!(error, SliceError::InvalidInput(_)));
    }
}

#[test]
fn existing_validation_apis_remain_intact_after_filament_flow_ratio_validation() {
    let basic_options: SliceOptions = serde_json::from_value(json!({
        "filament_diameter": 0.5
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
    let clearance_options: SliceOptions = serde_json::from_value(json!({
        "nozzle_height": 0
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
    let clearance_errors = clearance_options
        .validate_extruder_clearance_options()
        .unwrap();

    assert_eq!(basic_errors["filament_diameter"], "invalid value 0.5");
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
    assert_eq!(clearance_errors["nozzle_height"], "invalid value 0.000000");
}
