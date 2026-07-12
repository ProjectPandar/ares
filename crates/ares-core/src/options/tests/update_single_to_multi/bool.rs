use super::{options, update};
use crate::SliceError;
use serde_json::json;

#[test]
fn missing_variant_returns_minus_one_before_bool_validation() {
    let mut target = options(json!({"wipe": [true]}));
    let before = target.clone();
    let source = options(json!({"wipe": ["not a bool"]}));

    let result = update(&mut target, &source, &["wipe"]).unwrap();

    assert_eq!(result, -1);
    assert_eq!(target, before);
}

#[test]
fn bool_keys_resize_without_copying_source_values() {
    let mut target = options(json!({"wipe": [true, false]}));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "wipe": [false, true]
    }));

    update(&mut target, &source, &["wipe"]).unwrap();

    assert_eq!(target.values()["wipe"], json!([true, false]));
}

#[test]
fn bool_targets_truncate_to_variant_count_without_copying_source_values() {
    let mut target = options(json!({"wipe": [true, false, true]}));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "wipe": [false, true]
    }));

    update(&mut target, &source, &["wipe"]).unwrap();

    assert_eq!(target.values()["wipe"], json!([true, false]));
}

#[test]
fn bool_targets_extend_with_first_target_value_without_copying_source_values() {
    let mut target = options(json!({"wipe": [true]}));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard", "Direct Drive High Flow"],
        "wipe": [false, false, false]
    }));

    update(&mut target, &source, &["wipe"]).unwrap();

    assert_eq!(target.values()["wipe"], json!([true, true, true]));
}

#[test]
fn absent_bool_targets_use_registry_default_without_copying_source_values() {
    let mut target = options(json!({}));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "wipe": [true, true],
        "enable_overhang_bridge_fan": [false, false]
    }));

    update(
        &mut target,
        &source,
        &["wipe", "enable_overhang_bridge_fan"],
    )
    .unwrap();

    assert_eq!(target.values()["wipe"], json!([false, false]));
    assert_eq!(
        target.values()["enable_overhang_bridge_fan"],
        json!([true, true])
    );
}

#[test]
fn bool_source_length_must_match_variant_count() {
    let mut target = options(json!({"wipe": [true]}));
    let before = target.clone();
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "wipe": [false]
    }));

    let result = update(&mut target, &source, &["wipe"]);

    assert!(matches!(result, Err(SliceError::InvalidInput(_))));
    assert_eq!(target, before);

    let mut empty_target = options(json!({"wipe": [true]}));
    let empty_source = options(json!({
        "printer_extruder_variant": [],
        "wipe": []
    }));

    update(&mut empty_target, &empty_source, &["wipe"]).unwrap();

    assert_eq!(empty_target.values()["wipe"], json!([]));
}

#[test]
fn invalid_bool_source_or_target_values_return_invalid_input_without_mutation() {
    let invalid_source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard"],
        "wipe": ["not a bool"]
    }));
    let mut target = options(json!({"wipe": [true]}));
    let before = target.clone();

    let result = update(&mut target, &invalid_source, &["wipe"]);

    assert!(matches!(result, Err(SliceError::InvalidInput(_))));
    assert_eq!(target, before);

    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard"],
        "wipe": [false]
    }));
    let mut invalid_target = options(json!({"wipe": ["not a bool"]}));
    let before = invalid_target.clone();

    let result = update(&mut invalid_target, &source, &["wipe"]);

    assert!(matches!(result, Err(SliceError::InvalidInput(_))));
    assert_eq!(invalid_target, before);
}

#[test]
fn representative_bool_option_names_resize() {
    let mut target = options(json!({
        "activate_air_filtration": [true],
        "enable_pressure_advance": [true],
        "filament_is_support": [true],
        "wipe": [true]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "activate_air_filtration": [false, false],
        "enable_pressure_advance": [false, false],
        "filament_is_support": [false, false],
        "wipe": [false, false]
    }));

    update(
        &mut target,
        &source,
        &[
            "activate_air_filtration",
            "enable_pressure_advance",
            "filament_is_support",
            "wipe",
        ],
    )
    .unwrap();

    assert_eq!(
        target.values()["activate_air_filtration"],
        json!([true, true])
    );
    assert_eq!(
        target.values()["enable_pressure_advance"],
        json!([true, true])
    );
    assert_eq!(target.values()["filament_is_support"], json!([true, true]));
    assert_eq!(target.values()["wipe"], json!([true, true]));
}
