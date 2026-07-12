use super::super::apply_non_diff_direct_inheritance;
use crate::SliceOptions;
use serde_json::json;

fn options(value: serde_json::Value) -> SliceOptions {
    serde_json::from_value(value).unwrap()
}

#[test]
fn direct_inheritance_copies_changed_non_different_keys() {
    let mut current = options(json!({
        "wall_sequence": "inner wall/outer wall",
        "machine_start_gcode": "old start",
        "unrelated": true
    }));
    let target = options(json!({
        "wall_sequence": "outer wall/inner wall",
        "machine_start_gcode": "new start"
    }));

    apply_non_diff_direct_inheritance(
        &mut current,
        &target,
        &["wall_sequence", "machine_start_gcode"],
        &[],
    );

    assert_eq!(
        current.values()["wall_sequence"],
        json!("outer wall/inner wall")
    );
    assert_eq!(current.values()["machine_start_gcode"], json!("new start"));
    assert_eq!(current.values()["unrelated"], json!(true));
}

#[test]
fn direct_inheritance_skips_missing_and_equal_values() {
    let mut current = options(json!({
        "wall_sequence": "inner wall/outer wall",
        "curr_bed_type": "Textured PEI Plate",
        "current_only": 1
    }));
    let before = current.clone();
    let target = options(json!({
        "wall_sequence": "inner wall/outer wall",
        "target_only": 2
    }));

    apply_non_diff_direct_inheritance(
        &mut current,
        &target,
        &[
            "wall_sequence",
            "current_only",
            "target_only",
            "missing_both",
        ],
        &[],
    );

    assert_eq!(current, before);
}

#[test]
fn direct_inheritance_skips_different_keys_and_copies_other_keys() {
    let mut current = options(json!({
        "wall_sequence": "inner wall/outer wall",
        "machine_start_gcode": "old start"
    }));
    let target = options(json!({
        "wall_sequence": "outer wall/inner wall",
        "machine_start_gcode": "new start"
    }));

    apply_non_diff_direct_inheritance(
        &mut current,
        &target,
        &["wall_sequence", "machine_start_gcode"],
        &["machine_start_gcode"],
    );

    assert_eq!(
        current.values()["wall_sequence"],
        json!("outer wall/inner wall")
    );
    assert_eq!(current.values()["machine_start_gcode"], json!("old start"));
}

#[test]
fn direct_inheritance_repeated_keys_are_idempotent() {
    let mut current = options(json!({
        "wall_sequence": "inner wall/outer wall"
    }));
    let target = options(json!({
        "wall_sequence": "outer wall/inner wall"
    }));

    apply_non_diff_direct_inheritance(
        &mut current,
        &target,
        &["wall_sequence", "wall_sequence"],
        &[],
    );

    assert_eq!(
        current.values()["wall_sequence"],
        json!("outer wall/inner wall")
    );
}

#[test]
fn direct_inheritance_copies_unknown_present_json_without_registry_lookup() {
    let mut current = options(json!({
        "future_orca_key": {"mode": "old"}
    }));
    let target = options(json!({
        "future_orca_key": {"mode": "new", "nested": [1, 2, 3]}
    }));

    apply_non_diff_direct_inheritance(&mut current, &target, &["future_orca_key"], &[]);

    assert_eq!(
        current.values()["future_orca_key"],
        json!({"mode": "new", "nested": [1, 2, 3]})
    );
}
