use crate::{SliceError, SliceOptions};
use serde_json::json;

#[test]
fn default_fff_options_are_valid_through_aggregate() {
    let options: SliceOptions = serde_json::from_value(json!({})).unwrap();

    let errors = options.validate_fff_options(true).unwrap();

    assert!(errors.is_empty());
}

#[test]
fn aggregate_collects_representative_source_order_errors() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": -0.2,
        "use_firmware_retraction": true,
        "gcode_flavor": "teacup",
        "sparse_infill_pattern": "invalid-pattern",
        "skirt_height": -1,
        "bridge_flow": 0,
        "extruder_clearance_radius": 0,
        "filament_flow_ratio": [1.0, 0.0],
        "spiral_mode": true,
        "wall_loops": 2,
        "nozzle_diameter": 0.4,
        "outer_wall_line_width": 2.1,
        "line_width": -0.1
    }))
    .unwrap();

    let errors = options.validate_fff_options(true).unwrap();

    assert_eq!(errors["layer_height"], "invalid value -0.2");
    assert_eq!(
        errors["use_firmware_retraction"],
        "--use-firmware-retraction is only supported by Klipper, Marlin, Smoothie, RepRapFirmware, Repetier and Machinekit firmware"
    );
    assert_eq!(errors["gcode_flavor"], "invalid value teacup");
    assert_eq!(
        errors["sparse_infill_pattern"],
        "invalid value invalid-pattern"
    );
    assert_eq!(errors["skirt_height"], "invalid value -1");
    assert_eq!(errors["bridge_flow"], "invalid value 0.000000");
    assert_eq!(errors["internal_bridge_flow"], "invalid value 1.000000");
    assert_eq!(
        errors["extruder_clearance_radius"],
        "invalid value 0.000000"
    );
    assert_eq!(errors["filament_flow_ratio"], "invalid value 1,0");
    assert_eq!(
        errors["wall_loops"],
        "Invalid value when spiral vase mode is enabled: 2"
    );
    assert_eq!(
        errors["outer_wall_line_width"],
        "too large line width 2.100000"
    );
    assert_eq!(
        errors["line_width"],
        "-0.1 not in range [0.000000,1000.000000]"
    );
}

#[test]
fn aggregate_preserves_first_source_order_error_for_duplicate_keys() {
    let options: SliceOptions = serde_json::from_value(json!({
        "nozzle_diameter": 0.4,
        "outer_wall_line_width": 1001
    }))
    .unwrap();

    let errors = options.validate_fff_options(true).unwrap();

    assert_eq!(
        errors["outer_wall_line_width"],
        "too large line width 1001.000000"
    );
}

#[test]
fn aggregate_under_cli_false_suppresses_spiral_vase_cli_errors() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": -0.2,
        "spiral_mode": true,
        "wall_loops": 2,
        "sparse_infill_density": 20,
        "top_shell_layers": 4,
        "enable_support": true,
        "enforce_support_layers": 1
    }))
    .unwrap();

    let errors = options.validate_fff_options(false).unwrap();

    assert_eq!(errors["layer_height"], "invalid value -0.2");
    assert!(!errors.contains_key("wall_loops"));
}

#[test]
fn aggregate_returns_invalid_input_from_first_malformed_slice() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": true
    }))
    .unwrap();

    let error = options.validate_fff_options(true).unwrap_err();

    assert!(matches!(error, SliceError::InvalidInput(_)));
}

#[test]
fn standalone_validation_apis_remain_callable_after_aggregate_addition() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": -0.2,
        "outer_wall_line_width": 2.1,
        "line_width": -0.1,
        "nozzle_diameter": 0.4
    }))
    .unwrap();

    let basic_errors = options.validate_basic_fdm_options().unwrap();
    let width_errors = options.validate_extrusion_width_options().unwrap();
    let range_errors = options.validate_line_width_range_options().unwrap();

    assert_eq!(basic_errors["layer_height"], "invalid value -0.2");
    assert_eq!(
        width_errors["outer_wall_line_width"],
        "too large line width 2.100000"
    );
    assert_eq!(
        range_errors["line_width"],
        "-0.1 not in range [0.000000,1000.000000]"
    );
}
