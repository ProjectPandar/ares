use serde_json::json;

use super::collect_full_print_config_diff_updates;
use crate::SliceError;

fn json_map(value: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
    value
        .as_object()
        .expect("test value must be object")
        .clone()
}

fn collect(
    current: &serde_json::Map<String, serde_json::Value>,
    new_full: &serde_json::Map<String, serde_json::Value>,
    keys: &[&str],
    plate_index: usize,
) -> Vec<String> {
    let mut diff_keys = Vec::new();
    let owned_keys = keys.iter().map(|key| (*key).to_owned()).collect::<Vec<_>>();
    collect_full_print_config_diff_updates(
        current,
        new_full,
        &owned_keys,
        plate_index,
        &mut diff_keys,
    )
    .expect("full print diff collection should succeed");
    diff_keys
}

#[test]
fn full_print_diff_collects_missing_old_and_changed_non_wipe_keys() {
    let current = json_map(json!({ "travel_speed": 120 }));
    let new_full = json_map(json!({
        "travel_speed": 150,
        "inner_wall_speed": 60
    }));

    let diff_keys = collect(
        &current,
        &new_full,
        &["travel_speed", "inner_wall_speed"],
        0,
    );

    assert_eq!(diff_keys, ["travel_speed", "inner_wall_speed"]);
}

#[test]
fn full_print_diff_suppresses_equal_value() {
    let current = json_map(json!({ "travel_speed": 120 }));
    let new_full = json_map(json!({ "travel_speed": 120 }));

    let diff_keys = collect(&current, &new_full, &["travel_speed"], 0);

    assert!(diff_keys.is_empty());
}

#[test]
fn full_print_diff_errors_when_declared_new_key_is_missing() {
    let current = serde_json::Map::new();
    let new_full = serde_json::Map::new();
    let mut diff_keys = Vec::new();

    let error = collect_full_print_config_diff_updates(
        &current,
        &new_full,
        &["travel_speed".to_owned()],
        0,
        &mut diff_keys,
    )
    .expect_err("declared new key must exist");

    assert!(
        matches!(error, SliceError::InvalidInput(message) if message == "travel_speed new full value is missing")
    );
    assert!(diff_keys.is_empty());
}

#[test]
fn full_print_diff_uses_wipe_tower_plate_index_when_old_exists() {
    let current = json_map(json!({ "wipe_tower_y": [10, 20] }));
    let new_same_selected = json_map(json!({ "wipe_tower_y": [99, 20] }));
    let new_changed_selected = json_map(json!({ "wipe_tower_y": [10, 25] }));

    let same_keys = collect(&current, &new_same_selected, &["wipe_tower_y"], 1);
    let changed_keys = collect(&current, &new_changed_selected, &["wipe_tower_y"], 1);

    assert!(same_keys.is_empty());
    assert_eq!(changed_keys, ["wipe_tower_y"]);
}

#[test]
fn full_print_diff_wipe_tower_missing_old_always_emits() {
    let current = serde_json::Map::new();
    let new_full = json_map(json!({ "wipe_tower_x": [10] }));

    let diff_keys = collect(&current, &new_full, &["wipe_tower_x"], 1);

    assert_eq!(diff_keys, ["wipe_tower_x"]);
}

#[test]
fn full_print_diff_wipe_tower_emits_when_only_one_side_has_plate_index() {
    let current = json_map(json!({
        "wipe_tower_x": [10],
        "wipe_tower_y": [5, 6]
    }));
    let new_full = json_map(json!({
        "wipe_tower_x": [10, 20],
        "wipe_tower_y": [5]
    }));

    let diff_keys = collect(&current, &new_full, &["wipe_tower_x", "wipe_tower_y"], 1);

    assert_eq!(diff_keys, ["wipe_tower_x", "wipe_tower_y"]);
}
