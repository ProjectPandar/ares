use serde_json::json;

use super::{
    apply_vector_override, compute_filament_override_value, prepared_filament_override_value,
};
use crate::SliceError;

#[test]
fn long_retraction_disabled_replaces_bool_and_float_vectors_with_nil_arrays() {
    let enable = json!(0);

    let bool_result = prepared_filament_override_value(
        "long_retractions_when_cut",
        &json!([true, false, true]),
        Some(&enable),
    )
    .expect("disabled long retraction should replace bool vector");
    assert_eq!(bool_result, json!(["nil", "nil", "nil"]));

    let float_result = prepared_filament_override_value(
        "retraction_distances_when_cut",
        &json!([1.2, 0.8]),
        Some(&enable),
    )
    .expect("disabled long retraction should replace float vector");
    assert_eq!(float_result, json!(["nil", "nil"]));
}

#[test]
fn long_retraction_machine_mode_replaces_special_keys_with_nil_arrays() {
    let enable = json!(1);

    let bool_result = prepared_filament_override_value(
        "long_retractions_when_cut",
        &json!([false, true]),
        Some(&enable),
    )
    .expect("machine mode long retraction should replace bool vector");
    assert_eq!(bool_result, json!(["nil", "nil"]));

    let float_result = prepared_filament_override_value(
        "retraction_distances_when_cut",
        &json!([0.4, 0.6, 0.8]),
        Some(&enable),
    )
    .expect("machine mode long retraction should replace float vector");
    assert_eq!(float_result, json!(["nil", "nil", "nil"]));
}

#[test]
fn long_retraction_filament_mode_preserves_special_key_values() {
    let enable = json!(2);
    let bool_values = json!([true, false]);
    let float_values = json!([0.5, 1.25]);

    let bool_result =
        prepared_filament_override_value("long_retractions_when_cut", &bool_values, Some(&enable))
            .expect("filament mode should preserve bool vector");
    assert_eq!(bool_result, bool_values);

    let float_result = prepared_filament_override_value(
        "retraction_distances_when_cut",
        &float_values,
        Some(&enable),
    )
    .expect("filament mode should preserve float vector");
    assert_eq!(float_result, float_values);
}

#[test]
fn non_special_keys_pass_through_without_array_requirement() {
    let scalar_value = json!(3.5);

    let result = prepared_filament_override_value("retraction_length", &scalar_value, None)
        .expect("non-special keys should pass through scalar values");

    assert_eq!(result, scalar_value);
}

#[test]
fn invalid_enable_or_special_array_input_errors() {
    let missing_enable =
        prepared_filament_override_value("long_retractions_when_cut", &json!([true]), None)
            .expect_err("special keys require enable_long_retraction_when_cut");
    assert!(matches!(missing_enable, SliceError::InvalidInput(_)));

    let non_integer_enable = prepared_filament_override_value(
        "long_retractions_when_cut",
        &json!([true]),
        Some(&json!("2")),
    )
    .expect_err("special keys require an integer enable value");
    assert!(matches!(non_integer_enable, SliceError::InvalidInput(_)));

    let non_array_value = prepared_filament_override_value(
        "retraction_distances_when_cut",
        &json!(1.25),
        Some(&json!(0)),
    )
    .expect_err("special key values must be arrays when not in filament mode");
    assert!(matches!(non_array_value, SliceError::InvalidInput(_)));
}

#[test]
fn vector_override_non_nullable_replaces_and_detects_unchanged_values() {
    let mut machine_values = vec![json!(1), json!(2)];
    let override_values = vec![json!(3), json!(4), json!(5)];

    let modified = apply_vector_override(&mut machine_values, &override_values, &[], false)
        .expect("non-nullable override should replace different vectors");
    assert!(modified);
    assert_eq!(machine_values, override_values);

    let modified = apply_vector_override(&mut machine_values, &override_values, &[], false)
        .expect("non-nullable override should detect unchanged vectors");
    assert!(!modified);
    assert_eq!(machine_values, override_values);
}

#[test]
fn vector_override_nullable_copies_non_nil_entries_and_reports_modification() {
    let mut machine_values = vec![json!(10), json!(20), json!(30)];
    let override_values = vec![json!("nil"), json!(25), json!(35)];

    let modified = apply_vector_override(&mut machine_values, &override_values, &[], true)
        .expect("nullable override should copy non-nil entries");

    assert!(modified);
    assert_eq!(machine_values, vec![json!(10), json!(25), json!(35)]);
}

#[test]
fn vector_override_nullable_nil_restores_from_one_based_default_index() {
    let mut machine_values = vec![json!("current-1"), json!("current-2"), json!("current-3")];
    let override_values = vec![json!("nil"), json!("nil"), json!("nil")];
    let default_index = [3, 1, 2];

    let modified =
        apply_vector_override(&mut machine_values, &override_values, &default_index, true)
            .expect("nullable nil should restore indexed defaults");

    assert!(!modified);
    assert_eq!(
        machine_values,
        vec![json!("current-3"), json!("current-1"), json!("current-2")]
    );
}

#[test]
fn vector_override_nullable_nil_falls_back_to_first_original_machine_value() {
    let mut machine_values = vec![json!("fallback"), json!("second"), json!("third")];
    let override_values = vec![json!("nil"), json!("nil"), json!("nil")];
    let default_index = [0, 4];

    let modified =
        apply_vector_override(&mut machine_values, &override_values, &default_index, true)
            .expect("missing or invalid default index should use first default value");

    assert!(!modified);
    assert_eq!(
        machine_values,
        vec![json!("fallback"), json!("fallback"), json!("fallback")]
    );
}

#[test]
fn vector_override_nullable_zero_overlap_is_no_op_when_override_is_empty() {
    let mut machine_values = vec![json!(1), json!(2)];
    let modified = apply_vector_override(&mut machine_values, &[], &[], true)
        .expect("empty override vector should be a nullable no-op");
    assert!(!modified);
    assert_eq!(machine_values, vec![json!(1), json!(2)]);
}

#[test]
fn vector_override_nullable_zero_overlap_is_no_op_when_machine_is_empty() {
    let mut machine_values = Vec::new();

    let modified = apply_vector_override(&mut machine_values, &[json!("nil")], &[1], true)
        .expect("empty machine vector should be a nullable no-op");

    assert!(!modified);
    assert!(machine_values.is_empty());
}

#[test]
fn filament_override_update_emits_changed_non_nullable_override() {
    let mut diff_keys = Vec::new();
    let mut filament_overrides = serde_json::Map::new();

    let changed = compute_filament_override_value(
        "retraction_length",
        &json!([1, 2]),
        &json!([1, 2]),
        &json!([3, 4]),
        None,
        &[],
        false,
        &mut diff_keys,
        &mut filament_overrides,
    )
    .expect("changed non-nullable override should emit output");

    assert!(changed);
    assert_eq!(diff_keys, ["retraction_length"]);
    assert_eq!(
        filament_overrides.get("retraction_length"),
        Some(&json!([3, 4]))
    );
}

#[test]
fn filament_override_update_suppresses_unchanged_output() {
    let mut diff_keys = vec!["existing".to_owned()];
    let mut filament_overrides = serde_json::Map::from_iter([("existing".to_owned(), json!([9]))]);

    let changed = compute_filament_override_value(
        "retraction_length",
        &json!([3, 4]),
        &json!([1, 2]),
        &json!([3, 4]),
        None,
        &[],
        false,
        &mut diff_keys,
        &mut filament_overrides,
    )
    .expect("unchanged computed value should not emit output");

    assert!(!changed);
    assert_eq!(diff_keys, ["existing"]);
    assert_eq!(
        filament_overrides,
        serde_json::Map::from_iter([("existing".to_owned(), json!([9]))])
    );
}

#[test]
fn filament_override_update_uses_long_retraction_nil_preparation() {
    let mut diff_keys = Vec::new();
    let mut filament_overrides = serde_json::Map::new();

    let changed = compute_filament_override_value(
        "long_retractions_when_cut",
        &json!([true, false]),
        &json!([true, false]),
        &json!([false, true]),
        Some(&json!(0)),
        &[1, 2],
        true,
        &mut diff_keys,
        &mut filament_overrides,
    )
    .expect("long retraction disabled should prepare nil filament override");

    assert!(!changed);
    assert!(diff_keys.is_empty());
    assert!(filament_overrides.is_empty());
}

#[test]
fn filament_override_update_non_nullable_replacement_uses_final_changed_predicate() {
    let mut diff_keys = Vec::new();
    let mut filament_overrides = serde_json::Map::new();

    let changed = compute_filament_override_value(
        "retraction_length",
        &json!([1, 2]),
        &json!([7, 8]),
        &json!([7, 8]),
        None,
        &[],
        false,
        &mut diff_keys,
        &mut filament_overrides,
    )
    .expect("equal override application can still emit when old differs from computed clone");

    assert!(changed);
    assert_eq!(diff_keys, ["retraction_length"]);
    assert_eq!(
        filament_overrides.get("retraction_length"),
        Some(&json!([7, 8]))
    );
}

#[test]
fn filament_override_update_nullable_nil_restore_can_emit_without_non_nil_copy() {
    let mut diff_keys = Vec::new();
    let mut filament_overrides = serde_json::Map::new();

    let changed = compute_filament_override_value(
        "retraction_length",
        &json!(["old-1", "old-2"]),
        &json!(["new-1", "new-2"]),
        &json!(["nil", "nil"]),
        None,
        &[2, 1],
        true,
        &mut diff_keys,
        &mut filament_overrides,
    )
    .expect("nil restore can change clone even when no non-nil override was copied");

    assert!(changed);
    assert_eq!(diff_keys, ["retraction_length"]);
    assert_eq!(
        filament_overrides.get("retraction_length"),
        Some(&json!(["new-2", "new-1"]))
    );
}

#[test]
fn filament_override_update_nullable_zero_overlap_uses_final_comparison() {
    let mut diff_keys = Vec::new();
    let mut filament_overrides = serde_json::Map::new();

    let changed = compute_filament_override_value(
        "retraction_length",
        &json!(["old"]),
        &json!(["new"]),
        &json!([]),
        None,
        &[],
        true,
        &mut diff_keys,
        &mut filament_overrides,
    )
    .expect("zero-overlap override should leave clone for final comparison");

    assert!(changed);
    assert_eq!(diff_keys, ["retraction_length"]);
    assert_eq!(
        filament_overrides.get("retraction_length"),
        Some(&json!(["new"]))
    );
}
