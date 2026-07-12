use crate::{SliceError, SliceOptions};
use serde_json::{Value, json};

fn options(value: Value) -> SliceOptions {
    serde_json::from_value(value).unwrap()
}

fn update(target: &mut SliceOptions, keys: &[&str]) -> Result<isize, SliceError> {
    target.update_values_from_multi_to_single_2_float_keys(keys)
}

#[test]
fn float_keys_collapse_to_minimum_non_nil_value() {
    let mut target = options(json!({
        "filament_flow_ratio": [0.9, 0.6, 0.7]
    }));

    update(&mut target, &["filament_flow_ratio"]).unwrap();

    assert_eq!(target.values()["filament_flow_ratio"], json!([0.6]));
}

#[test]
fn float_keys_skip_nil_and_all_nil_preserves_first_entry() {
    let mut target = options(json!({
        "filament_flow_ratio": ["nil", 0.8, "nil"],
        "filament_retraction_length": ["nil", "nil", "nil"]
    }));

    update(
        &mut target,
        &["filament_flow_ratio", "filament_retraction_length"],
    )
    .unwrap();

    assert_eq!(target.values()["filament_flow_ratio"], json!([0.8]));
    assert_eq!(
        target.values()["filament_retraction_length"],
        json!(["nil"])
    );
}

#[test]
fn float_values_at_or_above_sentinel_preserve_original_first_entry() {
    let mut target = options(json!({
        "filament_flow_ratio": [1.2, 9999.0, 10000.0]
    }));

    update(&mut target, &["filament_flow_ratio"]).unwrap();

    assert_eq!(target.values()["filament_flow_ratio"], json!([1.2]));
}

#[test]
fn single_entry_float_uses_same_sentinel_selection_rule() {
    let mut selected = options(json!({
        "filament_flow_ratio": [0.8]
    }));
    let mut not_selected = options(json!({
        "fan_max_speed": [9999.0]
    }));

    update(&mut selected, &["filament_flow_ratio"]).unwrap();
    update(&mut not_selected, &["fan_max_speed"]).unwrap();

    assert_eq!(selected.values()["filament_flow_ratio"], json!([0.8]));
    assert_eq!(not_selected.values()["fan_max_speed"], json!([9999.0]));
}

#[test]
fn unsupported_kinds_and_absent_key_set_entries_are_skipped() {
    let mut target = options(json!({
        "filament_flow_ratio": [0.9, 0.6, 0.7],
        "filament_colour": ["#111111", "#222222", "#333333"]
    }));

    update(&mut target, &["filament_colour"]).unwrap();

    assert_eq!(
        target.values()["filament_flow_ratio"],
        json!([0.9, 0.6, 0.7])
    );
    assert_eq!(
        target.values()["filament_colour"],
        json!(["#111111", "#222222", "#333333"])
    );
}

#[test]
fn invalid_float_values_and_empty_arrays_do_not_mutate() {
    let cases = [
        json!({"filament_flow_ratio": [0.9, "fast", 0.7]}),
        json!({"filament_flow_ratio": []}),
    ];

    for value in cases {
        let mut target = options(value);
        let before = target.clone();

        let result = update(&mut target, &["filament_flow_ratio"]);

        assert!(matches!(result, Err(SliceError::InvalidInput(_))));
        assert_eq!(target, before);
    }
}

#[test]
fn later_invalid_float_key_does_not_partially_mutate_earlier_valid_key() {
    let mut target = options(json!({
        "filament_flow_ratio": [0.9, 0.6, 0.7],
        "filament_retraction_length": [1.0, "bad", 2.0]
    }));
    let before = target.clone();

    let result = update(
        &mut target,
        &["filament_flow_ratio", "filament_retraction_length"],
    );

    assert!(matches!(result, Err(SliceError::InvalidInput(_))));
    assert_eq!(target, before);
}

#[test]
fn representative_float_option_names_collapse() {
    let mut target = options(json!({
        "filament_flow_ratio": [0.9, 0.6, 0.7],
        "filament_retraction_length": ["nil", 2.0, 1.0],
        "fan_max_speed": [90.0, 70.0, 60.0]
    }));

    update(
        &mut target,
        &[
            "filament_flow_ratio",
            "filament_retraction_length",
            "fan_max_speed",
        ],
    )
    .unwrap();

    assert_eq!(target.values()["filament_flow_ratio"], json!([0.6]));
    assert_eq!(target.values()["filament_retraction_length"], json!([1.0]));
    assert_eq!(target.values()["fan_max_speed"], json!([60.0]));
}

#[test]
fn float_or_percent_collapse_selects_value_below_sentinel_and_preserves_percent_flag() {
    let mut target = options(json!({
        "bridge_acceleration": ["90%", 80.0, 70.0]
    }));

    update(&mut target, &["bridge_acceleration"]).unwrap();

    assert_eq!(target.values()["bridge_acceleration"], json!([70.0]));
}

#[test]
fn float_or_percent_skip_nil_and_all_nil_preserves_first_entry() {
    let mut target = options(json!({
        "bridge_acceleration": ["nil", "60%", "nil"],
        "line_width": ["nil", "nil", "nil"]
    }));

    update(&mut target, &["bridge_acceleration", "line_width"]).unwrap();

    assert_eq!(target.values()["bridge_acceleration"], json!(["60%"]));
    assert_eq!(target.values()["line_width"], json!(["nil"]));
}

#[test]
fn float_or_percent_equal_numeric_values_keep_first_candidate_not_operator_ordering() {
    let mut target = options(json!({
        "bridge_acceleration": ["50%", 50.0, "nil"]
    }));

    update(&mut target, &["bridge_acceleration"]).unwrap();

    assert_eq!(target.values()["bridge_acceleration"], json!(["50%"]));
}

#[test]
fn float_or_percent_values_at_or_above_sentinel_preserve_original_first_entry() {
    let mut numeric_first = options(json!({
        "bridge_acceleration": [9999.0, "10000%", "nil"]
    }));
    let mut percent_first = options(json!({
        "line_width": ["10000%", 9999.0, "nil"]
    }));

    update(&mut numeric_first, &["bridge_acceleration"]).unwrap();
    update(&mut percent_first, &["line_width"]).unwrap();

    assert_eq!(
        numeric_first.values()["bridge_acceleration"],
        json!([9999.0])
    );
    assert_eq!(percent_first.values()["line_width"], json!(["10000%"]));
}

#[test]
fn invalid_float_or_percent_values_and_empty_arrays_do_not_mutate() {
    let cases = [
        json!({"bridge_acceleration": ["90%", "fast", 70.0]}),
        json!({"bridge_acceleration": []}),
    ];

    for value in cases {
        let mut target = options(value);
        let before = target.clone();

        let result = update(&mut target, &["bridge_acceleration"]);

        assert!(matches!(result, Err(SliceError::InvalidInput(_))));
        assert_eq!(target, before);
    }
}

#[test]
fn later_invalid_float_or_percent_key_does_not_partially_mutate_earlier_valid_key() {
    let mut target = options(json!({
        "bridge_acceleration": ["90%", 80.0, 70.0],
        "line_width": [0.45, "bad", 0.4]
    }));
    let before = target.clone();

    let result = update(&mut target, &["bridge_acceleration", "line_width"]);

    assert!(matches!(result, Err(SliceError::InvalidInput(_))));
    assert_eq!(target, before);
}

#[test]
fn representative_float_or_percent_option_names_collapse() {
    let mut target = options(json!({
        "bridge_acceleration": ["90%", 80.0, 70.0],
        "line_width": [45.0, "40%", "nil"],
        "outer_wall_line_width": [0.3, 0.25, 0.2]
    }));

    update(
        &mut target,
        &["bridge_acceleration", "line_width", "outer_wall_line_width"],
    )
    .unwrap();

    assert_eq!(target.values()["bridge_acceleration"], json!([70.0]));
    assert_eq!(target.values()["line_width"], json!(["40%"]));
    assert_eq!(target.values()["outer_wall_line_width"], json!([0.2]));
}

#[test]
fn bool_keys_collapse_to_first_non_nil_value() {
    let mut target = options(json!({
        "filament_long_retractions_when_cut": ["nil", true, false]
    }));

    update(&mut target, &["filament_long_retractions_when_cut"]).unwrap();

    assert_eq!(
        target.values()["filament_long_retractions_when_cut"],
        json!([true])
    );
}

#[test]
fn bool_keys_skip_nil_and_select_first_false_before_later_true() {
    let mut target = options(json!({
        "filament_retract_when_changing_layer": ["nil", false, true]
    }));

    update(&mut target, &["filament_retract_when_changing_layer"]).unwrap();

    assert_eq!(
        target.values()["filament_retract_when_changing_layer"],
        json!([false])
    );
}

#[test]
fn bool_keys_all_nil_preserve_original_first_entry() {
    let mut target = options(json!({
        "filament_wipe": ["nil", "nil", "nil"]
    }));

    update(&mut target, &["filament_wipe"]).unwrap();

    assert_eq!(target.values()["filament_wipe"], json!(["nil"]));
}

#[test]
fn bool_keys_accept_numeric_tokens_and_reject_true_false_strings() {
    let mut numeric = options(json!({
        "filament_long_retractions_when_cut": ["nil", "0", 1]
    }));
    let mut text = options(json!({
        "filament_retract_when_changing_layer": ["false", true]
    }));
    let before = text.clone();

    update(&mut numeric, &["filament_long_retractions_when_cut"]).unwrap();
    let result = update(&mut text, &["filament_retract_when_changing_layer"]);

    assert_eq!(
        numeric.values()["filament_long_retractions_when_cut"],
        json!([false])
    );
    assert!(matches!(result, Err(SliceError::InvalidInput(_))));
    assert_eq!(text, before);
}

#[test]
fn invalid_bool_values_and_empty_arrays_do_not_mutate() {
    let cases = [
        json!({"filament_long_retractions_when_cut": [true, "yes", false]}),
        json!({"filament_long_retractions_when_cut": []}),
    ];

    for value in cases {
        let mut target = options(value);
        let before = target.clone();

        let result = update(&mut target, &["filament_long_retractions_when_cut"]);

        assert!(matches!(result, Err(SliceError::InvalidInput(_))));
        assert_eq!(target, before);
    }
}

#[test]
fn later_invalid_bool_key_does_not_partially_mutate_earlier_valid_key() {
    let mut target = options(json!({
        "filament_long_retractions_when_cut": ["nil", true, false],
        "filament_retract_when_changing_layer": [true, "bad", false]
    }));
    let before = target.clone();

    let result = update(
        &mut target,
        &[
            "filament_long_retractions_when_cut",
            "filament_retract_when_changing_layer",
        ],
    );

    assert!(matches!(result, Err(SliceError::InvalidInput(_))));
    assert_eq!(target, before);
}

#[test]
fn representative_bool_option_names_collapse() {
    let mut target = options(json!({
        "filament_long_retractions_when_cut": ["nil", true, false],
        "filament_retract_when_changing_layer": [false, "nil", true],
        "filament_wipe": [true, false, "nil"]
    }));

    update(
        &mut target,
        &[
            "filament_long_retractions_when_cut",
            "filament_retract_when_changing_layer",
            "filament_wipe",
        ],
    )
    .unwrap();

    assert_eq!(
        target.values()["filament_long_retractions_when_cut"],
        json!([true])
    );
    assert_eq!(
        target.values()["filament_retract_when_changing_layer"],
        json!([false])
    );
    assert_eq!(target.values()["filament_wipe"], json!([true]));
}
