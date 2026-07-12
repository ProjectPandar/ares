use super::{options, update};
use crate::SliceError;
use serde_json::json;

#[test]
fn missing_variant_returns_minus_one_before_float_or_percent_validation() {
    let mut target = options(json!({"line_width": [0.4]}));
    let before = target.clone();
    let source = options(json!({"line_width": ["bad%value"]}));

    let result = update(&mut target, &source, &["line_width"]).unwrap();

    assert_eq!(result, -1);
    assert_eq!(target, before);
}

#[test]
fn float_or_percent_keys_clamp_target_values_down_to_source_entries() {
    let mut target = options(json!({"line_width": [0.6, "90%"]}));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "line_width": [0.4, "80%"]
    }));

    update(&mut target, &source, &["line_width"]).unwrap();

    assert_eq!(target.values()["line_width"], json!([0.4, "80%"]));
}

#[test]
fn float_or_percent_keys_preserve_target_values_and_percent_flags_when_not_limited() {
    let mut target = options(json!({"line_width": [0.3, "80%"]}));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "line_width": [0.4, 80.0]
    }));

    update(&mut target, &source, &["line_width"]).unwrap();

    assert_eq!(target.values()["line_width"], json!([0.3, "80%"]));
}

#[test]
fn float_or_percent_source_percent_entries_preserve_percent_flag_when_limited() {
    let mut target = options(json!({"bridge_acceleration": [60.0]}));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard"],
        "bridge_acceleration": ["50%"]
    }));

    update(&mut target, &source, &["bridge_acceleration"]).unwrap();

    assert_eq!(target.values()["bridge_acceleration"], json!(["50%"]));
}

#[test]
fn float_or_percent_targets_truncate_to_variant_count_before_limiting() {
    let mut target = options(json!({"line_width": [0.6, 0.3, 0.9]}));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "line_width": [0.5, 0.4]
    }));

    update(&mut target, &source, &["line_width"]).unwrap();

    assert_eq!(target.values()["line_width"], json!([0.5, 0.3]));
}

#[test]
fn float_or_percent_targets_extend_with_first_target_entry_before_limiting() {
    let mut target = options(json!({"line_width": ["90%"]}));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard", "Direct Drive High Flow"],
        "line_width": [100.0, "80%", 95.0]
    }));

    update(&mut target, &source, &["line_width"]).unwrap();

    assert_eq!(target.values()["line_width"], json!(["90%", "80%", "90%"]));
}

#[test]
fn absent_float_or_percent_targets_use_registry_default_before_limiting() {
    let mut target = options(json!({}));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "bridge_acceleration": [40.0, "60%"]
    }));

    update(&mut target, &source, &["bridge_acceleration"]).unwrap();

    assert_eq!(target.values()["bridge_acceleration"], json!([40.0, "50%"]));
}

#[test]
fn float_or_percent_source_length_must_match_variant_count() {
    let mut target = options(json!({"line_width": [0.6]}));
    let before = target.clone();
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "line_width": [0.4]
    }));

    let result = update(&mut target, &source, &["line_width"]);

    assert!(matches!(result, Err(SliceError::InvalidInput(_))));
    assert_eq!(target, before);

    let mut empty_target = options(json!({"line_width": [0.6]}));
    let empty_source = options(json!({
        "printer_extruder_variant": [],
        "line_width": []
    }));

    update(&mut empty_target, &empty_source, &["line_width"]).unwrap();

    assert_eq!(empty_target.values()["line_width"], json!([]));
}

#[test]
fn invalid_float_or_percent_source_or_target_values_return_invalid_input_without_mutation() {
    let invalid_source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard"],
        "line_width": ["bad%value"]
    }));
    let mut target = options(json!({"line_width": [0.6]}));
    let before = target.clone();

    let result = update(&mut target, &invalid_source, &["line_width"]);

    assert!(matches!(result, Err(SliceError::InvalidInput(_))));
    assert_eq!(target, before);

    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard"],
        "line_width": [0.4]
    }));
    let mut invalid_target = options(json!({"line_width": ["bad%value"]}));
    let before = invalid_target.clone();

    let result = update(&mut invalid_target, &source, &["line_width"]);

    assert!(matches!(result, Err(SliceError::InvalidInput(_))));
    assert_eq!(invalid_target, before);
}

#[test]
fn representative_float_or_percent_option_names_limit() {
    let mut target = options(json!({
        "outer_wall_line_width": [0.5],
        "line_width": [0.7],
        "bridge_acceleration": ["60%"]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard"],
        "outer_wall_line_width": [0.4],
        "line_width": [0.6],
        "bridge_acceleration": ["50%"]
    }));

    update(
        &mut target,
        &source,
        &["outer_wall_line_width", "line_width", "bridge_acceleration"],
    )
    .unwrap();

    assert_eq!(target.values()["outer_wall_line_width"], json!([0.4]));
    assert_eq!(target.values()["line_width"], json!([0.6]));
    assert_eq!(target.values()["bridge_acceleration"], json!(["50%"]));
}
