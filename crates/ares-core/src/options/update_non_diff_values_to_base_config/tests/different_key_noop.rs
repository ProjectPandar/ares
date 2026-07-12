use super::super::different_key_keeps_current_value;
use crate::SliceOptions;
use serde_json::json;

fn options(value: serde_json::Value) -> SliceOptions {
    serde_json::from_value(value).unwrap()
}

#[test]
fn different_key_noop_returns_true_for_missing_target_values() {
    let target = options(json!({
        "wall_sequence": "outer wall/inner wall"
    }));
    let before = target.clone();

    assert!(different_key_keeps_current_value(
        "missing_key",
        &target,
        &["missing_key"],
        &["missing_key"],
    ));
    assert_eq!(target, before);
}

#[test]
fn different_key_noop_returns_true_for_scalar_targets_even_in_restore_sets() {
    let target = options(json!({
        "wall_sequence": "outer wall/inner wall",
        "enable_prime_tower": true
    }));

    assert!(different_key_keeps_current_value(
        "wall_sequence",
        &target,
        &["wall_sequence"],
        &["wall_sequence"],
    ));
    assert!(different_key_keeps_current_value(
        "enable_prime_tower",
        &target,
        &["enable_prime_tower"],
        &[],
    ));
}

#[test]
fn different_key_noop_returns_true_for_vector_absent_from_restore_sets() {
    let target = options(json!({
        "filament_diameter": [1.75, 1.75]
    }));

    assert!(different_key_keeps_current_value(
        "filament_diameter",
        &target,
        &[],
        &[],
    ));
    assert!(different_key_keeps_current_value(
        "filament_diameter",
        &target,
        &[],
        &["other_key"],
    ));
}

#[test]
fn different_key_noop_returns_false_for_vector_key_set1_members() {
    let target = options(json!({
        "filament_diameter": [1.75, 1.75]
    }));

    assert!(!different_key_keeps_current_value(
        "filament_diameter",
        &target,
        &["filament_diameter"],
        &[],
    ));
}

#[test]
fn different_key_noop_returns_false_for_vector_key_set2_members() {
    let target = options(json!({
        "machine_max_speed_x": [500, 200]
    }));

    assert!(!different_key_keeps_current_value(
        "machine_max_speed_x",
        &target,
        &[],
        &["machine_max_speed_x"],
    ));
}

#[test]
fn different_key_noop_classifies_unknown_json_by_shape() {
    let target = options(json!({
        "future_scalar": {"mode": "old"},
        "future_vector": [{"mode": "old"}]
    }));
    let before = target.clone();

    assert!(different_key_keeps_current_value(
        "future_scalar",
        &target,
        &["future_scalar"],
        &["future_scalar"],
    ));
    assert!(!different_key_keeps_current_value(
        "future_vector",
        &target,
        &["future_vector"],
        &[],
    ));
    assert_eq!(target, before);
}
