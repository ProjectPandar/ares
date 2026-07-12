use serde_json::json;

use super::collect_print_config_diff_updates;

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
) -> (Vec<String>, serde_json::Map<String, serde_json::Value>) {
    let mut diff_keys = Vec::new();
    let mut filament_overrides = serde_json::Map::new();
    let owned_keys = keys.iter().map(|key| (*key).to_owned()).collect::<Vec<_>>();
    collect_print_config_diff_updates(
        current,
        new_full,
        &owned_keys,
        plate_index,
        None,
        &[],
        false,
        &mut diff_keys,
        &mut filament_overrides,
    )
    .expect("print diff collection should succeed");
    (diff_keys, filament_overrides)
}

#[test]
fn print_diff_skips_missing_new_option() {
    let current = json_map(json!({ "travel_speed": 120 }));
    let new_full = serde_json::Map::new();

    let (diff_keys, filament_overrides) = collect(&current, &new_full, &["travel_speed"], 0);

    assert!(diff_keys.is_empty());
    assert!(filament_overrides.is_empty());
}

#[test]
fn print_diff_collects_changed_scalar_and_suppresses_equal_value() {
    let current = json_map(json!({
        "travel_speed": 120,
        "inner_wall_speed": 60
    }));
    let new_full = json_map(json!({
        "travel_speed": 150,
        "inner_wall_speed": 60
    }));

    let (diff_keys, filament_overrides) = collect(
        &current,
        &new_full,
        &["travel_speed", "inner_wall_speed"],
        0,
    );

    assert_eq!(diff_keys, ["travel_speed"]);
    assert!(filament_overrides.is_empty());
}

#[test]
fn print_diff_uses_wipe_tower_plate_index_comparison() {
    let current = json_map(json!({ "wipe_tower_x": [10, 20] }));
    let new_same_selected = json_map(json!({ "wipe_tower_x": [99, 20] }));
    let new_changed_selected = json_map(json!({ "wipe_tower_x": [10, 25] }));

    let (same_keys, _) = collect(&current, &new_same_selected, &["wipe_tower_x"], 1);
    let (changed_keys, _) = collect(&current, &new_changed_selected, &["wipe_tower_x"], 1);

    assert!(same_keys.is_empty());
    assert_eq!(changed_keys, ["wipe_tower_x"]);
}

#[test]
fn print_diff_wipe_tower_emits_when_only_one_side_has_plate_index() {
    let current = json_map(json!({
        "wipe_tower_x": [10],
        "wipe_tower_y": [5, 6]
    }));
    let new_full = json_map(json!({
        "wipe_tower_x": [10, 20],
        "wipe_tower_y": [5]
    }));

    let (diff_keys, _) = collect(&current, &new_full, &["wipe_tower_x", "wipe_tower_y"], 1);

    assert_eq!(diff_keys, ["wipe_tower_x", "wipe_tower_y"]);
}

#[test]
fn print_diff_filament_override_branch_takes_precedence() {
    let current = json_map(json!({ "retraction_length": [1, 2] }));
    let new_full = json_map(json!({
        "retraction_length": [9, 9],
        "filament_retraction_length": [3, 4]
    }));

    let (diff_keys, filament_overrides) = collect(&current, &new_full, &["retraction_length"], 0);

    assert_eq!(diff_keys, ["retraction_length"]);
    assert_eq!(
        filament_overrides.get("retraction_length"),
        Some(&json!([3, 4]))
    );
}
