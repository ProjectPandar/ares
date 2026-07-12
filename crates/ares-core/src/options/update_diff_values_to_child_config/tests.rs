use super::*;
use serde_json::json;
use std::collections::BTreeMap;

mod full_update;

fn options(values: &[(&str, Value)]) -> SliceOptions {
    SliceOptions {
        values: values
            .iter()
            .map(|(key, value)| ((*key).to_owned(), value.clone()))
            .collect::<BTreeMap<_, _>>(),
    }
}

#[test]
fn diff_child_variant_indices_match_current_variants_and_ids_in_current_order() {
    let current = options(&[
        ("printer_extruder_id", json!([10, 20, 30])),
        ("printer_extruder_variant", json!(["a", "b", "c"])),
    ]);
    let target = options(&[
        ("printer_extruder_id", json!([30, 10, 20])),
        ("printer_extruder_variant", json!(["c", "a", "b"])),
    ]);

    assert_eq!(
        diff_child_variant_indices(
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
fn diff_child_variant_indices_empty_id_name_matches_by_variant_only() {
    let current = options(&[("printer_extruder_variant", json!(["b", "a"]))]);
    let target = options(&[("printer_extruder_variant", json!(["a", "b"]))]);

    assert_eq!(
        diff_child_variant_indices(&current, &target, "", "printer_extruder_variant").unwrap(),
        vec![1, 0]
    );
}

#[test]
fn diff_child_variant_indices_missing_current_variants_returns_zero() {
    let current = options(&[]);
    let target = options(&[("printer_extruder_variant", json!(["a"]))]);

    assert_eq!(
        diff_child_variant_indices(
            &current,
            &target,
            "printer_extruder_id",
            "printer_extruder_variant",
        )
        .unwrap(),
        vec![0]
    );
}

#[test]
fn diff_child_variant_indices_missing_target_variants_sets_first_current_to_zero() {
    let current = options(&[("printer_extruder_variant", json!(["a", "b", "c"]))]);
    let target = options(&[]);

    assert_eq!(
        diff_child_variant_indices(
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
fn diff_child_variant_indices_current_id_length_mismatch_returns_initialized_vector() {
    let current = options(&[
        ("printer_extruder_id", json!([10])),
        ("printer_extruder_variant", json!(["a", "b"])),
    ]);
    let target = options(&[("printer_extruder_variant", json!(["a", "b"]))]);

    assert_eq!(
        diff_child_variant_indices(
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
fn diff_child_variant_indices_target_id_length_mismatch_returns_initialized_vector() {
    let current = options(&[
        ("printer_extruder_id", json!([10, 20])),
        ("printer_extruder_variant", json!(["a", "b"])),
    ]);
    let target = options(&[
        ("printer_extruder_id", json!([10])),
        ("printer_extruder_variant", json!(["a", "b"])),
    ]);

    assert_eq!(
        diff_child_variant_indices(
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
fn diff_child_variant_indices_unmatched_current_variants_remain_negative_one() {
    let current = options(&[("printer_extruder_variant", json!(["a", "missing"]))]);
    let target = options(&[("printer_extruder_variant", json!(["a", "b"]))]);

    assert_eq!(
        diff_child_variant_indices(&current, &target, "", "printer_extruder_variant").unwrap(),
        vec![0, -1]
    );
}

#[test]
fn diff_child_variant_indices_reject_malformed_present_vectors() {
    let current = options(&[
        ("printer_extruder_id", json!(["not-int"])),
        ("printer_extruder_variant", json!(["a"])),
    ]);
    let target = options(&[("printer_extruder_variant", json!([1]))]);

    let result = diff_child_variant_indices(
        &current,
        &target,
        "printer_extruder_id",
        "printer_extruder_variant",
    );
    assert!(matches!(result, Err(SliceError::InvalidInput(_))));
}

#[test]
fn diff_direct_child_values_copies_changed_scalar_values() {
    let mut current = options(&[("speed", json!(40)), ("unchanged", json!(1))]);
    let target = options(&[("speed", json!(60)), ("unchanged", json!(1))]);

    apply_diff_direct_child_values(
        &mut current,
        &target,
        DiffDirectChildValueKeys {
            keys: &["speed", "unchanged"],
            extruder_id_name: "id",
            extruder_variant_name: "variant",
            key_set1: &[],
            key_set2: &[],
        },
    );

    assert_eq!(current.values().get("speed"), Some(&json!(60)));
    assert_eq!(current.values().get("unchanged"), Some(&json!(1)));
}

#[test]
fn diff_direct_child_values_copies_vector_values_absent_from_restore_sets() {
    let mut current = options(&[("colors", json!(["red"]))]);
    let target = options(&[("colors", json!(["blue", "green"]))]);

    apply_diff_direct_child_values(
        &mut current,
        &target,
        DiffDirectChildValueKeys {
            keys: &["colors"],
            extruder_id_name: "id",
            extruder_variant_name: "variant",
            key_set1: &[],
            key_set2: &[],
        },
    );

    assert_eq!(
        current.values().get("colors"),
        Some(&json!(["blue", "green"]))
    );
}

#[test]
fn diff_direct_child_values_skips_extruder_metadata_keys() {
    let mut current = options(&[
        ("printer_extruder_id", json!([1])),
        ("printer_extruder_variant", json!(["a"])),
    ]);
    let target = options(&[
        ("printer_extruder_id", json!([2])),
        ("printer_extruder_variant", json!(["b"])),
    ]);

    apply_diff_direct_child_values(
        &mut current,
        &target,
        DiffDirectChildValueKeys {
            keys: &["printer_extruder_id", "printer_extruder_variant"],
            extruder_id_name: "printer_extruder_id",
            extruder_variant_name: "printer_extruder_variant",
            key_set1: &[],
            key_set2: &[],
        },
    );

    assert_eq!(
        current.values().get("printer_extruder_id"),
        Some(&json!([1]))
    );
    assert_eq!(
        current.values().get("printer_extruder_variant"),
        Some(&json!(["a"]))
    );
}

#[test]
fn diff_direct_child_values_skips_missing_and_equal_values() {
    let mut current = options(&[("present_equal", json!(1)), ("missing_target", json!(2))]);
    let target = options(&[("present_equal", json!(1)), ("missing_source", json!(3))]);

    apply_diff_direct_child_values(
        &mut current,
        &target,
        DiffDirectChildValueKeys {
            keys: &["present_equal", "missing_target", "missing_source"],
            extruder_id_name: "id",
            extruder_variant_name: "variant",
            key_set1: &[],
            key_set2: &[],
        },
    );

    assert_eq!(current.values().get("present_equal"), Some(&json!(1)));
    assert_eq!(current.values().get("missing_target"), Some(&json!(2)));
    assert!(!current.values().contains_key("missing_source"));
}

#[test]
fn diff_direct_child_values_leaves_restore_vectors_for_later_set_only_diff() {
    let mut current = options(&[
        ("temperatures", json!([200, 210])),
        ("offsets", json!([0.1, 0.2])),
    ]);
    let target = options(&[
        ("temperatures", json!([220, 230])),
        ("offsets", json!([0.3, 0.4])),
    ]);

    apply_diff_direct_child_values(
        &mut current,
        &target,
        DiffDirectChildValueKeys {
            keys: &["temperatures", "offsets"],
            extruder_id_name: "id",
            extruder_variant_name: "variant",
            key_set1: &["temperatures"],
            key_set2: &["offsets"],
        },
    );

    assert_eq!(
        current.values().get("temperatures"),
        Some(&json!([200, 210]))
    );
    assert_eq!(current.values().get("offsets"), Some(&json!([0.1, 0.2])));
}

#[test]
fn diff_vector_stride_uses_two_for_key_set2_members_only() {
    assert_eq!(
        diff_vector_stride("filament_colour", &["filament_colour"]),
        2
    );
    assert_eq!(diff_vector_stride("temperature", &["filament_colour"]), 1);
    assert_eq!(diff_vector_stride("temperature", &[]), 1);
}

#[test]
fn diff_set_only_diff_stride1_copies_selected_target_values() {
    let mut source = vec![10, 20, 30];
    let target = vec![Some(1), Some(2), Some(3)];

    apply_diff_set_only_diff(&mut source, &target, &[2, -1, 0], 1).unwrap();

    assert_eq!(source, vec![3, 20, 1]);
}

#[test]
fn diff_set_only_diff_all_negative_indexes_leave_source_unchanged() {
    let mut source = vec![10, 20];
    let target = vec![Some(1), Some(2)];

    apply_diff_set_only_diff(&mut source, &target, &[-1, -1], 1).unwrap();

    assert_eq!(source, vec![10, 20]);
}

#[test]
fn diff_set_only_diff_invalid_source_size_errors_before_mutating() {
    let mut source = vec![10];
    let target = vec![Some(1), Some(2)];

    let error = apply_diff_set_only_diff(&mut source, &target, &[0, 1], 1)
        .expect_err("source length must match diff index length times stride");

    assert_eq!(source, vec![10]);
    assert!(
        matches!(error, SliceError::InvalidInput(message) if message == "ConfigOptionVector::set_only_diff(): Assigning from an vector with invalid diff_index size")
    );
}

#[test]
fn diff_set_only_diff_stride2_copies_selected_target_pairs() {
    let mut source = vec![10, 11, 20, 21, 30, 31];
    let target = vec![Some(1), Some(2), Some(3), Some(4), Some(5), Some(6)];

    apply_diff_set_only_diff(&mut source, &target, &[2, -1, 0], 2).unwrap();

    assert_eq!(source, vec![5, 6, 20, 21, 1, 2]);
}

#[test]
fn diff_set_only_diff_nil_target_slot_skips_whole_stride_segment() {
    let mut source = vec![10, 11, 20, 21];
    let target = vec![None, Some(2), Some(3), Some(4)];

    apply_diff_set_only_diff(&mut source, &target, &[0, 1], 2).unwrap();

    assert_eq!(source, vec![10, 11, 3, 4]);
}
