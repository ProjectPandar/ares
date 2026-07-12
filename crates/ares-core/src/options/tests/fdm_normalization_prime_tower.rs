use super::super::*;
use crate::SliceError;
use serde_json::json;

#[test]
fn missing_prime_tower_option_skips_prime_tower_side_effects() {
    let mut options: SliceOptions = serde_json::from_value(json!({})).unwrap();

    options.normalize_fdm(2).unwrap();

    let has_independent_support_layer_height = options
        .values()
        .contains_key("independent_support_layer_height");
    assert!(!options.values().contains_key("enable_prime_tower"));
    assert!(!has_independent_support_layer_height);
}

#[test]
fn zero_used_filaments_skips_prime_tower_branch() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "enable_prime_tower": true,
        "independent_support_layer_height": true,
        "print_sequence": "by object",
    }))
    .unwrap();

    options.normalize_fdm(0).unwrap();

    assert_eq!(options.values()["enable_prime_tower"], json!(true));
    assert_eq!(
        options.values()["independent_support_layer_height"],
        json!(true)
    );
}

#[test]
fn non_smooth_single_filament_disables_prime_tower() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "enable_prime_tower": true,
        "independent_support_layer_height": true,
        "print_sequence": "by layer",
        "timelapse_type": "0",
    }))
    .unwrap();

    options.normalize_fdm(1).unwrap();

    assert_eq!(options.values()["enable_prime_tower"], json!(false));
    assert_eq!(
        options.values()["independent_support_layer_height"],
        json!(true)
    );
}

#[test]
fn non_smooth_by_object_disables_prime_tower() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "enable_prime_tower": true,
        "independent_support_layer_height": true,
        "print_sequence": "by object",
    }))
    .unwrap();

    options.normalize_fdm(2).unwrap();

    assert_eq!(options.values()["enable_prime_tower"], json!(false));
    assert_eq!(
        options.values()["independent_support_layer_height"],
        json!(true)
    );
}

#[test]
fn smooth_timelapse_preserves_prime_tower_and_disables_independent_support_height() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "enable_prime_tower": true,
        "independent_support_layer_height": true,
        "print_sequence": "by object",
        "timelapse_type": "1",
    }))
    .unwrap();

    options.normalize_fdm(1).unwrap();

    assert_eq!(options.values()["enable_prime_tower"], json!(true));
    assert_eq!(
        options.values()["independent_support_layer_height"],
        json!(false)
    );
}

#[test]
fn false_prime_tower_creates_default_independent_support_height() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "enable_prime_tower": false,
        "print_sequence": "by layer",
    }))
    .unwrap();

    options.normalize_fdm(2).unwrap();

    assert_eq!(options.values()["enable_prime_tower"], json!(false));
    assert_eq!(
        options.values()["independent_support_layer_height"],
        json!(true)
    );
}

#[test]
fn disabled_prime_tower_creates_default_independent_support_height() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "enable_prime_tower": true,
        "print_sequence": "by object",
    }))
    .unwrap();

    options.normalize_fdm(2).unwrap();

    assert_eq!(options.values()["enable_prime_tower"], json!(false));
    assert_eq!(
        options.values()["independent_support_layer_height"],
        json!(true)
    );
}

#[test]
fn enabled_prime_tower_creates_disabled_independent_support_height() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "enable_prime_tower": true,
        "print_sequence": "by layer",
    }))
    .unwrap();

    options.normalize_fdm(2).unwrap();

    assert_eq!(options.values()["enable_prime_tower"], json!(true));
    assert_eq!(
        options.values()["independent_support_layer_height"],
        json!(false)
    );
}

#[test]
fn enabled_prime_tower_disables_existing_independent_support_height() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "enable_prime_tower": true,
        "independent_support_layer_height": true,
        "print_sequence": "by layer",
    }))
    .unwrap();

    options.normalize_fdm(2).unwrap();

    assert_eq!(options.values()["enable_prime_tower"], json!(true));
    assert_eq!(
        options.values()["independent_support_layer_height"],
        json!(false)
    );
}

#[test]
fn invalid_prime_tower_branch_values_return_invalid_input() {
    for values in [
        json!({"enable_prime_tower": "true"}),
        json!({"enable_prime_tower": true, "print_sequence": 1}),
        json!({"enable_prime_tower": true, "print_sequence": "sequential"}),
        json!({"enable_prime_tower": true, "timelapse_type": 1}),
        json!({"enable_prime_tower": true, "timelapse_type": "smooth"}),
    ] {
        let mut options: SliceOptions = serde_json::from_value(values.clone()).unwrap();

        let err = options
            .normalize_fdm(2)
            .expect_err(&format!("expected invalid input for {values:?}"));

        assert!(matches!(err, SliceError::InvalidInput(_)));
    }
}

#[test]
fn prime_tower_branch_keeps_earlier_fdm_normalization() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "extruder": 3,
        "spiral_mode": true,
        "resolution": 0,
        "enable_prime_tower": true,
        "print_sequence": "by layer",
        "timelapse_type": "1",
    }))
    .unwrap();

    options.normalize_fdm(2).unwrap();

    assert_eq!(options.values()["resolution"], json!(0.001));
    assert!(!options.values().contains_key("extruder"));
    assert_eq!(options.values()["sparse_infill_filament"], json!(3));
    assert_eq!(options.values()["wall_filament"], json!(3));
    assert_eq!(options.values()["solid_infill_filament"], json!(3));
    assert_eq!(options.values()["wall_loops"], json!(1));
    assert_eq!(options.values()["enable_prime_tower"], json!(true));
    assert_eq!(
        options.values()["independent_support_layer_height"],
        json!(false)
    );
}
