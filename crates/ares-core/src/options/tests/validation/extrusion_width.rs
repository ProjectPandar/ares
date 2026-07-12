use crate::{SliceError, SliceOptions};
use serde_json::json;

const TOO_LARGE_PREFIX: &str = "too large line width";

#[test]
fn default_extrusion_width_options_are_valid() {
    let options: SliceOptions = serde_json::from_value(json!({})).unwrap();

    let errors = options.validate_extrusion_width_options().unwrap();

    assert!(errors.is_empty());
}

#[test]
fn max_nozzle_diameter_controls_extrusion_width_limit() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.25, 0.6],
        "outer_wall_line_width": 2.5,
        "inner_wall_line_width": 3.000001
    }))
    .unwrap();

    let errors = options.validate_extrusion_width_options().unwrap();

    assert!(!errors.contains_key("outer_wall_line_width"));
    assert_eq!(
        errors["inner_wall_line_width"],
        "too large line width 3.000001"
    );
    assert_eq!(errors.len(), 1);
}

#[test]
fn over_limit_absolute_extrusion_widths_are_reported() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": 0.4,
        "outer_wall_line_width": 2.25,
        "inner_wall_line_width": 2.000001,
        "sparse_infill_line_width": "2.1",
        "internal_solid_infill_line_width": 2.2,
        "top_surface_line_width": 2.3,
        "support_line_width": 2.4,
        "initial_layer_line_width": 2.5,
        "skin_infill_line_width": 2.6,
        "skeleton_infill_line_width": 2.7
    }))
    .unwrap();

    let errors = options.validate_extrusion_width_options().unwrap();

    assert_eq!(
        errors["outer_wall_line_width"],
        "too large line width 2.250000"
    );
    assert_eq!(
        errors["inner_wall_line_width"],
        "too large line width 2.000001"
    );
    assert_eq!(
        errors["sparse_infill_line_width"],
        "too large line width 2.100000"
    );
    assert_eq!(
        errors["internal_solid_infill_line_width"],
        "too large line width 2.200000"
    );
    assert_eq!(
        errors["top_surface_line_width"],
        "too large line width 2.300000"
    );
    assert_eq!(
        errors["support_line_width"],
        "too large line width 2.400000"
    );
    assert_eq!(
        errors["initial_layer_line_width"],
        "too large line width 2.500000"
    );
    assert_eq!(
        errors["skin_infill_line_width"],
        "too large line width 2.600000"
    );
    assert_eq!(
        errors["skeleton_infill_line_width"],
        "too large line width 2.700000"
    );
    assert_eq!(errors.len(), 9);
}

#[test]
fn over_limit_percent_extrusion_widths_use_documented_message_deviation() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": [0.25, 0.6],
        "outer_wall_line_width": "500.0001%",
        "inner_wall_line_width": "510%",
        "sparse_infill_line_width": "520%",
        "internal_solid_infill_line_width": "530%",
        "top_surface_line_width": "540%",
        "support_line_width": "550%",
        "initial_layer_line_width": "560%",
        "skin_infill_line_width": "570%",
        "skeleton_infill_line_width": "580%"
    }))
    .unwrap();

    let errors = options.validate_extrusion_width_options().unwrap();

    assert_eq!(
        errors["outer_wall_line_width"],
        "too large line width 3.000001"
    );
    assert_eq!(
        errors["inner_wall_line_width"],
        "too large line width 3.060000"
    );
    assert_eq!(
        errors["sparse_infill_line_width"],
        "too large line width 3.120000"
    );
    assert_eq!(
        errors["internal_solid_infill_line_width"],
        "too large line width 3.180000"
    );
    assert_eq!(
        errors["top_surface_line_width"],
        "too large line width 3.240000"
    );
    assert_eq!(
        errors["support_line_width"],
        "too large line width 3.300000"
    );
    assert_eq!(
        errors["initial_layer_line_width"],
        "too large line width 3.360000"
    );
    assert_eq!(
        errors["skin_infill_line_width"],
        "too large line width 3.420000"
    );
    assert_eq!(
        errors["skeleton_infill_line_width"],
        "too large line width 3.480000"
    );
    assert_eq!(errors.len(), 9);
}

#[test]
fn exactly_maximum_allowed_extrusion_width_is_valid() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": 0.4,
        "outer_wall_line_width": 2.0,
        "inner_wall_line_width": "500%",
        "sparse_infill_line_width": 2.0,
        "internal_solid_infill_line_width": "500%",
        "top_surface_line_width": 2.0,
        "support_line_width": "500%",
        "initial_layer_line_width": 2.0,
        "skin_infill_line_width": "500%",
        "skeleton_infill_line_width": 2.0
    }))
    .unwrap();

    let errors = options.validate_extrusion_width_options().unwrap();

    assert!(errors.is_empty());
}

#[test]
fn invalid_extrusion_width_boundary_values_return_invalid_input() {
    for (key, value) in [
        ("nozzle_diameter", json!(0)),
        ("nozzle_diameter", json!("nan")),
        ("outer_wall_line_width", json!(true)),
        ("inner_wall_line_width", json!("wide")),
        ("sparse_infill_line_width", json!("nan%")),
        ("top_surface_line_width", json!({"width": 2.1})),
        ("support_line_width", json!([2.1])),
        ("initial_layer_line_width", json!("inf")),
        ("skeleton_infill_line_width", json!("1%%")),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            key: value
        }))
        .unwrap();

        let error = options.validate_extrusion_width_options().unwrap_err();

        assert!(matches!(error, SliceError::InvalidInput(_)), "{key}");
    }
}

#[test]
fn finite_source_range_invalid_widths_are_deferred_to_generic_range_validation() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": 0.4,
        "outer_wall_line_width": -0.1,
        "inner_wall_line_width": "-25%"
    }))
    .unwrap();

    let errors = options.validate_extrusion_width_options().unwrap();

    assert!(errors.is_empty());
}

#[test]
fn existing_validation_apis_remain_intact_after_extrusion_width_validation() {
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
    let clearance_options: SliceOptions = serde_json::from_value(json!({
        "nozzle_height": 0
    }))
    .unwrap();
    let flow_options: SliceOptions = serde_json::from_value(json!({
        "filament_flow_ratio": 0
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
    let flow_errors = flow_options.validate_filament_flow_ratio_options().unwrap();

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
    assert_eq!(clearance_errors["nozzle_height"], "invalid value 0.000000");
    assert_eq!(flow_errors["filament_flow_ratio"], "invalid value 0");

    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": 0.4,
        "outer_wall_line_width": 2.1
    }))
    .unwrap();
    let errors = options.validate_extrusion_width_options().unwrap();

    assert_eq!(
        errors["outer_wall_line_width"],
        format!("{TOO_LARGE_PREFIX} 2.100000")
    );
}
