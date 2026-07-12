use crate::{SliceError, SliceOptions};
use serde_json::json;

fn options(value: serde_json::Value) -> SliceOptions {
    serde_json::from_value(value).unwrap()
}

#[test]
fn absent_and_single_nozzle_return_false() {
    assert!(!options(json!({})).is_using_different_extruders().unwrap());
    assert!(
        !options(json!({ "nozzle_diameter": [0.4] }))
            .is_using_different_extruders()
            .unwrap()
    );
}

#[test]
fn multiple_nozzles_with_missing_enum_vectors_return_false() {
    assert!(
        !options(json!({ "nozzle_diameter": [0.4, 0.6] }))
            .is_using_different_extruders()
            .unwrap()
    );
    assert!(
        !options(json!({
            "nozzle_diameter": [0.4, 0.6],
            "extruder_type": ["Direct Drive", "Bowden"]
        }))
        .is_using_different_extruders()
        .unwrap()
    );
    assert!(
        !options(json!({
            "nozzle_diameter": [0.4, 0.6],
            "nozzle_volume_type": ["Standard", "High Flow"]
        }))
        .is_using_different_extruders()
        .unwrap()
    );
}

#[test]
fn matching_extruder_and_nozzle_volume_types_return_false() {
    assert!(
        !options(json!({
            "nozzle_diameter": [0.4, 0.6],
            "extruder_type": ["Direct Drive", "Direct Drive"],
            "nozzle_volume_type": ["Standard", "Standard"]
        }))
        .is_using_different_extruders()
        .unwrap()
    );
}

#[test]
fn different_extruder_type_returns_true() {
    assert!(
        options(json!({
            "nozzle_diameter": [0.4, 0.6],
            "extruder_type": ["Direct Drive", "Bowden"],
            "nozzle_volume_type": ["Standard", "Standard"]
        }))
        .is_using_different_extruders()
        .unwrap()
    );
}

#[test]
fn different_nozzle_volume_type_returns_true() {
    assert!(
        options(json!({
            "nozzle_diameter": [0.4, 0.6],
            "extruder_type": ["Direct Drive", "Direct Drive"],
            "nozzle_volume_type": ["Standard", "High Flow"]
        }))
        .is_using_different_extruders()
        .unwrap()
    );
}

#[test]
fn enum_get_at_uses_first_value_for_out_of_range_indices() {
    assert!(
        !options(json!({
            "nozzle_diameter": [0.4, 0.6, 0.8],
            "extruder_type": ["Direct Drive"],
            "nozzle_volume_type": ["Standard"]
        }))
        .is_using_different_extruders()
        .unwrap()
    );
}

#[test]
fn invalid_boundary_values_return_invalid_input() {
    for value in [
        json!({ "nozzle_diameter": [0.4, 0.6], "extruder_type": "Direct Drive", "nozzle_volume_type": ["Standard"] }),
        json!({ "nozzle_diameter": [0.4, 0.6], "extruder_type": [], "nozzle_volume_type": ["Standard"] }),
        json!({ "nozzle_diameter": [0.4, 0.6], "extruder_type": [7], "nozzle_volume_type": ["Standard"] }),
        json!({ "nozzle_diameter": [0.4, 0.6], "extruder_type": ["Cartesian"], "nozzle_volume_type": ["Standard"] }),
        json!({ "nozzle_diameter": [0.4, 0.6], "extruder_type": ["Direct Drive"], "nozzle_volume_type": "Standard" }),
        json!({ "nozzle_diameter": [0.4, 0.6], "extruder_type": ["Direct Drive"], "nozzle_volume_type": [] }),
        json!({ "nozzle_diameter": [0.4, 0.6], "extruder_type": ["Direct Drive"], "nozzle_volume_type": [7] }),
        json!({ "nozzle_diameter": [0.4, 0.6], "extruder_type": ["Direct Drive"], "nozzle_volume_type": ["Ultra Flow"] }),
        json!({ "nozzle_diameter": [0.0, 0.6] }),
    ] {
        assert!(matches!(
            options(value).is_using_different_extruders(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}
