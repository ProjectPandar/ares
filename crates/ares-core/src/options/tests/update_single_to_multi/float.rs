use super::{options, update};
use crate::SliceError;
use serde_json::json;

#[test]
fn missing_variant_returns_minus_one_before_float_validation() {
    let mut target = options(json!({"fan_max_speed": [80.0]}));
    let before = target.clone();
    let source = options(json!({"fan_max_speed": ["not a float"]}));

    let result = update(&mut target, &source, &["fan_max_speed"]).unwrap();

    assert_eq!(result, -1);
    assert_eq!(target, before);
}

#[test]
fn float_keys_clamp_target_values_down_to_source_values() {
    let mut target = options(json!({"fan_max_speed": [90.0, 70.0]}));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "fan_max_speed": [80.0, 60.0]
    }));

    update(&mut target, &source, &["fan_max_speed"]).unwrap();

    assert_eq!(target.values()["fan_max_speed"], json!([80.0, 60.0]));
}

#[test]
fn float_keys_preserve_target_values_less_than_or_equal_to_source() {
    let mut target = options(json!({"fan_max_speed": [70.0, 60.0]}));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "fan_max_speed": [80.0, 60.0]
    }));

    update(&mut target, &source, &["fan_max_speed"]).unwrap();

    assert_eq!(target.values()["fan_max_speed"], json!([70.0, 60.0]));
}

#[test]
fn float_targets_truncate_to_variant_count_before_limiting() {
    let mut target = options(json!({"fan_max_speed": [90.0, 70.0, 50.0]}));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "fan_max_speed": [80.0, 80.0]
    }));

    update(&mut target, &source, &["fan_max_speed"]).unwrap();

    assert_eq!(target.values()["fan_max_speed"], json!([80.0, 70.0]));
}

#[test]
fn float_targets_extend_with_first_target_value_before_limiting() {
    let mut target = options(json!({"fan_max_speed": [90.0]}));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard", "Direct Drive High Flow"],
        "fan_max_speed": [100.0, 80.0, 95.0]
    }));

    update(&mut target, &source, &["fan_max_speed"]).unwrap();

    assert_eq!(target.values()["fan_max_speed"], json!([90.0, 80.0, 90.0]));
}

#[test]
fn absent_float_targets_use_registry_default_before_limiting() {
    let mut target = options(json!({}));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "fan_max_speed": [80.0, 120.0]
    }));

    update(&mut target, &source, &["fan_max_speed"]).unwrap();

    assert_eq!(target.values()["fan_max_speed"], json!([80.0, 100.0]));
}

#[test]
fn float_source_length_must_match_variant_count() {
    let mut target = options(json!({"fan_max_speed": [90.0]}));
    let before = target.clone();
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "fan_max_speed": [80.0]
    }));

    let result = update(&mut target, &source, &["fan_max_speed"]);

    assert!(matches!(result, Err(SliceError::InvalidInput(_))));
    assert_eq!(target, before);

    let mut empty_target = options(json!({"fan_max_speed": [90.0]}));
    let empty_source = options(json!({
        "printer_extruder_variant": [],
        "fan_max_speed": []
    }));

    update(&mut empty_target, &empty_source, &["fan_max_speed"]).unwrap();

    assert_eq!(empty_target.values()["fan_max_speed"], json!([]));
}

#[test]
fn invalid_float_source_or_target_values_return_invalid_input_without_mutation() {
    let cases = [
        json!({
            "printer_extruder_variant": ["Direct Drive Standard"],
            "fan_max_speed": ["fast"]
        }),
        json!({
            "printer_extruder_variant": ["Direct Drive Standard"],
            "fan_max_speed": [80.0]
        }),
    ];
    let mut invalid_source_target = options(json!({"fan_max_speed": [90.0]}));
    let before_source = invalid_source_target.clone();
    let result = update(
        &mut invalid_source_target,
        &options(cases[0].clone()),
        &["fan_max_speed"],
    );
    assert!(matches!(result, Err(SliceError::InvalidInput(_))));
    assert_eq!(invalid_source_target, before_source);

    let mut invalid_target = options(json!({"fan_max_speed": ["fast"]}));
    let before_target = invalid_target.clone();
    let result = update(
        &mut invalid_target,
        &options(cases[1].clone()),
        &["fan_max_speed"],
    );
    assert!(matches!(result, Err(SliceError::InvalidInput(_))));
    assert_eq!(invalid_target, before_target);
}

#[test]
fn representative_float_option_names_limit() {
    let mut target = options(json!({
        "extruder_printable_height": [10.0],
        "fan_cooling_layer_time": [90.0],
        "fan_max_speed": [110.0]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard"],
        "extruder_printable_height": [5.0],
        "fan_cooling_layer_time": [60.0],
        "fan_max_speed": [100.0]
    }));

    update(
        &mut target,
        &source,
        &[
            "extruder_printable_height",
            "fan_cooling_layer_time",
            "fan_max_speed",
        ],
    )
    .unwrap();

    assert_eq!(target.values()["extruder_printable_height"], json!([5.0]));
    assert_eq!(target.values()["fan_cooling_layer_time"], json!([60.0]));
    assert_eq!(target.values()["fan_max_speed"], json!([100.0]));
}
