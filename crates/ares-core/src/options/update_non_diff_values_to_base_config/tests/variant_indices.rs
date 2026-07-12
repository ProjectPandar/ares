use super::super::non_diff_variant_indices;
use crate::{SliceError, SliceOptions};
use serde_json::json;

fn options(value: serde_json::Value) -> SliceOptions {
    serde_json::from_value(value).unwrap()
}

#[test]
fn non_diff_variant_indices_match_target_variants_and_ids_in_target_order() {
    let current = options(json!({
        "printer_extruder_id": [2, 1, 3],
        "printer_extruder_variant": ["Bowden Standard", "Direct Drive Standard", "High Flow"]
    }));
    let target = options(json!({
        "printer_extruder_id": [1, 3, 2],
        "printer_extruder_variant": ["Direct Drive Standard", "High Flow", "Bowden Standard"]
    }));

    assert_eq!(
        non_diff_variant_indices(
            &current,
            &target,
            "printer_extruder_id",
            "printer_extruder_variant",
        )
        .unwrap(),
        vec![1, 2, 0]
    );
}

#[test]
fn non_diff_variant_indices_empty_id_name_matches_by_variant_only() {
    let current = options(json!({
        "printer_extruder_id": [9, 8],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"]
    }));
    let target = options(json!({
        "printer_extruder_id": [1, 2],
        "printer_extruder_variant": ["Bowden Standard", "Direct Drive Standard"]
    }));

    assert_eq!(
        non_diff_variant_indices(&current, &target, "", "printer_extruder_variant").unwrap(),
        vec![1, 0]
    );
}

#[test]
fn non_diff_variant_indices_missing_current_variants_maps_first_target_to_zero() {
    let current = options(json!({}));
    let target = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard", "High Flow"]
    }));

    assert_eq!(
        non_diff_variant_indices(
            &current,
            &target,
            "printer_extruder_id",
            "printer_extruder_variant",
        )
        .unwrap(),
        vec![0, -1, -1]
    );
}

#[test]
fn non_diff_variant_indices_current_id_length_mismatch_returns_all_negative_one() {
    let current = options(json!({
        "printer_extruder_id": [1],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"]
    }));
    let target = options(json!({
        "printer_extruder_id": [1, 2],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"]
    }));

    assert_eq!(
        non_diff_variant_indices(
            &current,
            &target,
            "printer_extruder_id",
            "printer_extruder_variant",
        )
        .unwrap(),
        vec![-1, -1]
    );
}

#[test]
fn non_diff_variant_indices_target_id_length_mismatch_returns_all_negative_one() {
    let current = options(json!({
        "printer_extruder_id": [1, 2],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"]
    }));
    let target = options(json!({
        "printer_extruder_id": [1],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"]
    }));

    assert_eq!(
        non_diff_variant_indices(
            &current,
            &target,
            "printer_extruder_id",
            "printer_extruder_variant",
        )
        .unwrap(),
        vec![-1, -1]
    );
}

#[test]
fn non_diff_variant_indices_unmatched_targets_remain_negative_one() {
    let current = options(json!({
        "printer_extruder_id": [1],
        "printer_extruder_variant": ["Direct Drive Standard"]
    }));
    let target = options(json!({
        "printer_extruder_id": [1, 2],
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"]
    }));

    assert_eq!(
        non_diff_variant_indices(
            &current,
            &target,
            "printer_extruder_id",
            "printer_extruder_variant",
        )
        .unwrap(),
        vec![0, -1]
    );
}

#[test]
fn non_diff_variant_indices_missing_target_variants_returns_empty() {
    let current = options(json!({
        "printer_extruder_id": [1],
        "printer_extruder_variant": ["Direct Drive Standard"]
    }));
    let target = options(json!({}));

    assert_eq!(
        non_diff_variant_indices(
            &current,
            &target,
            "printer_extruder_id",
            "printer_extruder_variant",
        )
        .unwrap(),
        Vec::<isize>::new()
    );
}

#[test]
fn non_diff_variant_indices_reject_malformed_present_vectors() {
    let valid = options(json!({
        "printer_extruder_id": [1],
        "printer_extruder_variant": ["Direct Drive Standard"]
    }));
    for (current, target) in [
        (
            options(json!({
                "printer_extruder_id": ["1"],
                "printer_extruder_variant": ["Direct Drive Standard"]
            })),
            valid.clone(),
        ),
        (
            options(json!({
                "printer_extruder_id": [1],
                "printer_extruder_variant": [1]
            })),
            valid.clone(),
        ),
        (
            valid.clone(),
            options(json!({
                "printer_extruder_id": [1],
                "printer_extruder_variant": [1]
            })),
        ),
    ] {
        let result = non_diff_variant_indices(
            &current,
            &target,
            "printer_extruder_id",
            "printer_extruder_variant",
        );
        assert!(matches!(result, Err(SliceError::InvalidInput(_))));
    }
}
