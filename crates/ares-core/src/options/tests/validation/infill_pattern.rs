use crate::{SliceError, SliceOptions};
use serde_json::json;

const SPARSE_PATTERNS: &[&str] = &[
    "rectilinear",
    "alignedrectilinear",
    "zigzag",
    "crosszag",
    "lockedzag",
    "line",
    "grid",
    "triangles",
    "tri-hexagon",
    "cubic",
    "adaptivecubic",
    "quartercubic",
    "supportcubic",
    "lightning",
    "honeycomb",
    "3dhoneycomb",
    "lateral-honeycomb",
    "lateral-lattice",
    "crosshatch",
    "tpmsd",
    "tpmsfk",
    "gyroid",
    "concentric",
    "hilbertcurve",
    "archimedeanchords",
    "octagramspiral",
];

const SURFACE_PATTERNS: &[&str] = &[
    "monotonic",
    "monotonicline",
    "rectilinear",
    "alignedrectilinear",
    "concentric",
    "hilbertcurve",
    "archimedeanchords",
    "octagramspiral",
];

#[test]
fn default_infill_pattern_options_are_valid() {
    let options: SliceOptions = serde_json::from_value(json!({})).unwrap();

    let errors = options.validate_infill_pattern_options().unwrap();

    assert!(errors.is_empty());
}

#[test]
fn active_sparse_infill_pattern_values_are_valid() {
    for pattern in SPARSE_PATTERNS {
        let options: SliceOptions = serde_json::from_value(json!({
            "sparse_infill_pattern": pattern
        }))
        .unwrap();

        let errors = options.validate_infill_pattern_options().unwrap();

        assert!(errors.is_empty(), "{pattern}");
    }
}

#[test]
fn active_surface_pattern_values_are_valid() {
    for key in [
        "top_surface_pattern",
        "bottom_surface_pattern",
        "internal_solid_infill_pattern",
    ] {
        for pattern in SURFACE_PATTERNS {
            let options: SliceOptions = serde_json::from_value(json!({
                key: pattern
            }))
            .unwrap();

            let errors = options.validate_infill_pattern_options().unwrap();

            assert!(errors.is_empty(), "{key}={pattern}");
        }
    }
}

#[test]
fn sparse_only_pattern_values_are_rejected_for_surface_patterns() {
    for key in [
        "top_surface_pattern",
        "bottom_surface_pattern",
        "internal_solid_infill_pattern",
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            key: "gyroid"
        }))
        .unwrap();

        let errors = options.validate_infill_pattern_options().unwrap();

        assert_eq!(errors[key], "invalid value gyroid");
    }
}

#[test]
fn unknown_infill_pattern_values_are_reported_by_key() {
    let options: SliceOptions = serde_json::from_value(json!({
        "sparse_infill_pattern": "unknown-sparse",
        "top_surface_pattern": "unknown-top",
        "bottom_surface_pattern": "unknown-bottom",
        "internal_solid_infill_pattern": "unknown-internal"
    }))
    .unwrap();

    let errors = options.validate_infill_pattern_options().unwrap();

    assert_eq!(
        errors["sparse_infill_pattern"],
        "invalid value unknown-sparse"
    );
    assert_eq!(errors["top_surface_pattern"], "invalid value unknown-top");
    assert_eq!(
        errors["bottom_surface_pattern"],
        "invalid value unknown-bottom"
    );
    assert_eq!(
        errors["internal_solid_infill_pattern"],
        "invalid value unknown-internal"
    );
}

#[test]
fn invalid_infill_pattern_types_return_invalid_input() {
    for key in [
        "sparse_infill_pattern",
        "top_surface_pattern",
        "bottom_surface_pattern",
        "internal_solid_infill_pattern",
    ] {
        let options: SliceOptions = serde_json::from_value(json!({
            key: true
        }))
        .unwrap();

        let error = options.validate_infill_pattern_options().unwrap_err();

        assert!(matches!(error, SliceError::InvalidInput(_)));
    }
}

#[test]
fn existing_validation_apis_remain_intact_after_infill_pattern_validation() {
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

    let basic_errors = basic_options.validate_basic_fdm_options().unwrap();
    let firmware_errors = firmware_options
        .validate_firmware_retraction_options()
        .unwrap();
    let flavor_errors = flavor_options.validate_gcode_flavor_option().unwrap();

    assert!(basic_errors["layer_height"].contains("invalid value 0"));
    assert!(firmware_errors.is_empty());
    assert_eq!(
        flavor_errors["gcode_flavor"],
        "invalid value unknown-firmware"
    );
}
