use crate::{DifferentExtrudersSupport, SliceError, SliceOptions};
use serde_json::json;

fn options(value: serde_json::Value) -> SliceOptions {
    serde_json::from_value(value).unwrap()
}

fn support(supported: bool, extruder_count: usize) -> DifferentExtrudersSupport {
    DifferentExtrudersSupport {
        supported,
        extruder_count,
    }
}

#[test]
fn missing_nozzle_uses_default_count_and_returns_false() {
    assert_eq!(
        options(json!({})).support_different_extruders().unwrap(),
        support(false, 1)
    );
    assert_eq!(
        options(json!({ "extruder_variant_list": ["A,B"] }))
            .support_different_extruders()
            .unwrap(),
        support(false, 1)
    );
}

#[test]
fn missing_variant_list_returns_false_with_resolved_nozzle_count() {
    assert_eq!(
        options(json!({ "nozzle_diameter": [0.4, 0.6] }))
            .support_different_extruders()
            .unwrap(),
        support(false, 2)
    );
}

#[test]
fn identical_variants_return_false() {
    assert_eq!(
        options(json!({
            "nozzle_diameter": [0.4, 0.6],
            "extruder_variant_list": ["A", "A"]
        }))
        .support_different_extruders()
        .unwrap(),
        support(false, 2)
    );
}

#[test]
fn distinct_variants_across_nozzles_return_true() {
    assert_eq!(
        options(json!({
            "nozzle_diameter": [0.4, 0.6],
            "extruder_variant_list": ["A", "B"]
        }))
        .support_different_extruders()
        .unwrap(),
        support(true, 2)
    );
}

#[test]
fn single_nozzle_with_multiple_variant_tokens_returns_true() {
    assert_eq!(
        options(json!({
            "nozzle_diameter": [0.4],
            "extruder_variant_list": ["A,B"]
        }))
        .support_different_extruders()
        .unwrap(),
        support(true, 1)
    );
}

#[test]
fn variant_get_at_uses_first_value_for_out_of_range_indices() {
    assert_eq!(
        options(json!({
            "nozzle_diameter": [0.4, 0.6, 0.8],
            "extruder_variant_list": ["A"]
        }))
        .support_different_extruders()
        .unwrap(),
        support(false, 3)
    );
}

#[test]
fn boost_split_edge_cases_affect_unique_variant_set() {
    for (variant, expected_supported) in [
        ("", false),
        (",", false),
        (",A", true),
        ("A,", true),
        ("A,,B", true),
    ] {
        assert_eq!(
            options(json!({
                "nozzle_diameter": [0.4],
                "extruder_variant_list": [variant]
            }))
            .support_different_extruders()
            .unwrap(),
            support(expected_supported, 1),
            "{variant:?}"
        );
    }
}

#[test]
fn invalid_boundary_values_return_invalid_input() {
    for value in [
        json!({ "nozzle_diameter": [0.0], "extruder_variant_list": ["A"] }),
        json!({ "nozzle_diameter": [0.4], "extruder_variant_list": "A" }),
        json!({ "nozzle_diameter": [0.4], "extruder_variant_list": [] }),
        json!({ "nozzle_diameter": [0.4], "extruder_variant_list": [7] }),
    ] {
        assert!(matches!(
            options(value).support_different_extruders(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}
