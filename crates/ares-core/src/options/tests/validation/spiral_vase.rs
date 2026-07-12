use crate::{SliceError, SliceOptions};
use serde_json::json;

const SPIRAL_VASE_MESSAGE_PREFIX: &str = "Invalid value when spiral vase mode is enabled";

#[test]
fn default_spiral_vase_cli_options_are_valid() {
    let options: SliceOptions = serde_json::from_value(json!({})).unwrap();

    let errors = options.validate_spiral_vase_cli_options().unwrap();

    assert!(errors.is_empty());
}

#[test]
fn spiral_vase_cli_validation_is_suppressed_when_spiral_mode_is_false() {
    let options: SliceOptions = serde_json::from_value(json!({
        "spiral_mode": false,
        "wall_loops": 2,
        "sparse_infill_density": 20,
        "top_shell_layers": 4,
        "enable_support": true,
        "enforce_support_layers": 1
    }))
    .unwrap();

    let errors = options.validate_spiral_vase_cli_options().unwrap();

    assert!(errors.is_empty());
}

#[test]
fn spiral_vase_cli_conflicts_are_reported() {
    let options: SliceOptions = serde_json::from_value(json!({
        "spiral_mode": true,
        "wall_loops": 2,
        "sparse_infill_density": 20,
        "top_shell_layers": 4,
        "enable_support": true,
        "enforce_support_layers": 1
    }))
    .unwrap();

    let errors = options.validate_spiral_vase_cli_options().unwrap();

    assert_eq!(
        errors["wall_loops"],
        format!("{SPIRAL_VASE_MESSAGE_PREFIX}: 2")
    );
    assert_eq!(
        errors["sparse_infill_density"],
        format!("{SPIRAL_VASE_MESSAGE_PREFIX}: 20.000000")
    );
    assert_eq!(
        errors["top_shell_layers"],
        format!("{SPIRAL_VASE_MESSAGE_PREFIX}: 4")
    );
    assert_eq!(
        errors["enable_support"],
        format!("{SPIRAL_VASE_MESSAGE_PREFIX}: 1")
    );
    assert_eq!(
        errors["enforce_support_layers"],
        format!("{SPIRAL_VASE_MESSAGE_PREFIX}: 1")
    );
    assert_eq!(errors.len(), 5);
}

#[test]
fn spiral_vase_true_with_missing_constrained_keys_uses_registry_defaults() {
    let options: SliceOptions = serde_json::from_value(json!({
        "spiral_mode": true
    }))
    .unwrap();

    let errors = options.validate_spiral_vase_cli_options().unwrap();

    assert_eq!(
        errors["wall_loops"],
        format!("{SPIRAL_VASE_MESSAGE_PREFIX}: 2")
    );
    assert_eq!(
        errors["sparse_infill_density"],
        format!("{SPIRAL_VASE_MESSAGE_PREFIX}: 20.000000")
    );
    assert_eq!(
        errors["top_shell_layers"],
        format!("{SPIRAL_VASE_MESSAGE_PREFIX}: 4")
    );
    assert_eq!(errors.len(), 3);
}

#[test]
fn numeric_string_spiral_vase_values_use_same_predicates() {
    let options: SliceOptions = serde_json::from_value(json!({
        "spiral_mode": true,
        "wall_loops": "3",
        "sparse_infill_density": "15.5",
        "top_shell_layers": "2",
        "enable_support": false,
        "enforce_support_layers": "0"
    }))
    .unwrap();

    let errors = options.validate_spiral_vase_cli_options().unwrap();

    assert_eq!(
        errors["wall_loops"],
        format!("{SPIRAL_VASE_MESSAGE_PREFIX}: 3")
    );
    assert_eq!(
        errors["sparse_infill_density"],
        format!("{SPIRAL_VASE_MESSAGE_PREFIX}: 15.500000")
    );
    assert_eq!(
        errors["top_shell_layers"],
        format!("{SPIRAL_VASE_MESSAGE_PREFIX}: 2")
    );
    assert_eq!(errors.len(), 3);
}

#[test]
fn invalid_spiral_vase_cli_types_return_invalid_input() {
    for (key, value, message) in [
        ("spiral_mode", json!(1), "spiral_mode must be a bool"),
        ("wall_loops", json!(1.5), "wall_loops must be an integer"),
        (
            "sparse_infill_density",
            json!(true),
            "sparse_infill_density must be a finite number",
        ),
        (
            "top_shell_layers",
            json!(false),
            "top_shell_layers must be an integer",
        ),
        (
            "enable_support",
            json!("true"),
            "enable_support must be a bool",
        ),
        (
            "enforce_support_layers",
            json!({"layers": 1}),
            "enforce_support_layers must be an integer",
        ),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            "spiral_mode": true,
            key: value
        }))
        .unwrap();

        let error = options.validate_spiral_vase_cli_options().unwrap_err();

        assert_eq!(error, SliceError::InvalidInput(message.to_owned()), "{key}");
    }
}

#[test]
fn existing_validation_apis_remain_intact_after_spiral_vase_validation() {
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
}
