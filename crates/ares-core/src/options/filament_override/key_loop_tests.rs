use serde_json::json;

use super::collect_filament_override_updates;
use crate::SliceError;

fn json_map(value: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
    value
        .as_object()
        .expect("test value must be object")
        .clone()
}

#[test]
fn filament_override_key_loop_collects_changed_unprefixed_override() {
    let old_machine = json_map(json!({
        "retraction_length": [1, 2]
    }));
    let new_machine = json_map(json!({
        "retraction_length": [1, 2]
    }));
    let new_full = json_map(json!({
        "filament_retraction_length": [3, 4]
    }));
    let mut diff_keys = Vec::new();
    let mut filament_overrides = serde_json::Map::new();

    collect_filament_override_updates(
        &old_machine,
        &new_machine,
        &new_full,
        None,
        &[],
        false,
        &mut diff_keys,
        &mut filament_overrides,
    )
    .expect("present filament override should be collected");

    assert_eq!(diff_keys, ["retraction_length"]);
    assert_eq!(
        filament_overrides.get("retraction_length"),
        Some(&json!([3, 4]))
    );
}

#[test]
fn filament_override_key_loop_skips_missing_prefixed_key_before_machine_lookup() {
    let old_machine = serde_json::Map::new();
    let new_machine = serde_json::Map::new();
    let new_full = serde_json::Map::new();
    let mut diff_keys = Vec::new();
    let mut filament_overrides = serde_json::Map::new();

    collect_filament_override_updates(
        &old_machine,
        &new_machine,
        &new_full,
        None,
        &[],
        false,
        &mut diff_keys,
        &mut filament_overrides,
    )
    .expect("missing filament override keys should skip machine lookup");

    assert!(diff_keys.is_empty());
    assert!(filament_overrides.is_empty());
}

#[test]
fn filament_override_key_loop_errors_when_machine_values_missing_for_present_filament() {
    let old_machine = serde_json::Map::new();
    let new_machine = serde_json::Map::new();
    let new_full = json_map(json!({
        "filament_retraction_length": [3, 4]
    }));
    let mut diff_keys = Vec::new();
    let mut filament_overrides = serde_json::Map::new();

    let error = collect_filament_override_updates(
        &old_machine,
        &new_machine,
        &new_full,
        None,
        &[],
        false,
        &mut diff_keys,
        &mut filament_overrides,
    )
    .expect_err("present filament override requires machine values");

    assert!(
        matches!(error, SliceError::InvalidInput(message) if message == "retraction_length old machine value is missing")
    );
    assert!(diff_keys.is_empty());
    assert!(filament_overrides.is_empty());
}

#[test]
fn filament_override_key_loop_preserves_extruder_retract_key_source_order() {
    let old_machine = json_map(json!({
        "deretraction_speed": [1, 2],
        "retraction_length": [1, 2],
        "wipe": [false, false]
    }));
    let new_machine = json_map(json!({
        "deretraction_speed": [1, 2],
        "retraction_length": [1, 2],
        "wipe": [false, false]
    }));
    let new_full = json_map(json!({
        "filament_deretraction_speed": [10, 20],
        "filament_retraction_length": [3, 4],
        "filament_wipe": [true, false]
    }));
    let mut diff_keys = Vec::new();
    let mut filament_overrides = serde_json::Map::new();

    collect_filament_override_updates(
        &old_machine,
        &new_machine,
        &new_full,
        None,
        &[],
        false,
        &mut diff_keys,
        &mut filament_overrides,
    )
    .expect("multiple changed overrides should preserve registry source order");

    assert_eq!(
        diff_keys,
        ["deretraction_speed", "retraction_length", "wipe"]
    );
    assert_eq!(
        filament_overrides.get("deretraction_speed"),
        Some(&json!([10, 20]))
    );
    assert_eq!(
        filament_overrides.get("retraction_length"),
        Some(&json!([3, 4]))
    );
    assert_eq!(filament_overrides.get("wipe"), Some(&json!([true, false])));
}

#[test]
fn filament_override_key_loop_suppresses_unchanged_computed_override() {
    let old_machine = json_map(json!({
        "retraction_length": [3, 4]
    }));
    let new_machine = json_map(json!({
        "retraction_length": [1, 2]
    }));
    let new_full = json_map(json!({
        "filament_retraction_length": [3, 4]
    }));
    let mut diff_keys = vec!["existing".to_owned()];
    let mut filament_overrides = serde_json::Map::from_iter([("existing".to_owned(), json!([9]))]);

    collect_filament_override_updates(
        &old_machine,
        &new_machine,
        &new_full,
        None,
        &[],
        false,
        &mut diff_keys,
        &mut filament_overrides,
    )
    .expect("unchanged computed override should leave outputs untouched");

    assert_eq!(diff_keys, ["existing"]);
    assert_eq!(
        filament_overrides,
        serde_json::Map::from_iter([("existing".to_owned(), json!([9]))])
    );
}
