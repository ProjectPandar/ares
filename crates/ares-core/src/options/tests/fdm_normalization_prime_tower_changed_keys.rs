use super::super::*;
use crate::SliceError;
use serde_json::json;

#[test]
fn missing_prime_tower_option_returns_no_changed_keys() {
    let mut options: SliceOptions = serde_json::from_value(json!({})).unwrap();

    let changed_keys = options.normalize_fdm_2(1, 2).unwrap();

    assert!(changed_keys.is_empty());
    assert!(!options.values().contains_key("enable_prime_tower"));
    assert!(
        !options
            .values()
            .contains_key("independent_support_layer_height")
    );
}

#[test]
fn zero_used_filaments_returns_no_changed_keys() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "enable_prime_tower": true,
        "independent_support_layer_height": true,
        "print_sequence": "by object",
    }))
    .unwrap();

    let changed_keys = options.normalize_fdm_2(2, 0).unwrap();

    assert!(changed_keys.is_empty());
    assert_eq!(options.values()["enable_prime_tower"], json!(true));
    assert_eq!(
        options.values()["independent_support_layer_height"],
        json!(true)
    );
}

#[test]
fn single_filament_disables_prime_tower_and_reports_key() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "enable_prime_tower": true,
        "independent_support_layer_height": true,
        "print_sequence": "by layer",
        "timelapse_type": "0",
    }))
    .unwrap();

    let changed_keys = options.normalize_fdm_2(1, 1).unwrap();

    assert_eq!(changed_keys, ["enable_prime_tower"]);
    assert_eq!(options.values()["enable_prime_tower"], json!(false));
    assert_eq!(
        options.values()["independent_support_layer_height"],
        json!(true)
    );
}

#[test]
fn by_object_with_multiple_objects_disables_prime_tower_and_reports_key() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "enable_prime_tower": true,
        "independent_support_layer_height": true,
        "print_sequence": "by object",
    }))
    .unwrap();

    let changed_keys = options.normalize_fdm_2(2, 2).unwrap();

    assert_eq!(changed_keys, ["enable_prime_tower"]);
    assert_eq!(options.values()["enable_prime_tower"], json!(false));
    assert_eq!(
        options.values()["independent_support_layer_height"],
        json!(true)
    );
}

#[test]
fn by_object_with_one_object_keeps_prime_tower_and_reports_support_height_key() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "enable_prime_tower": true,
        "independent_support_layer_height": true,
        "print_sequence": "by object",
    }))
    .unwrap();

    let changed_keys = options.normalize_fdm_2(1, 2).unwrap();

    assert_eq!(changed_keys, ["independent_support_layer_height"]);
    assert_eq!(options.values()["enable_prime_tower"], json!(true));
    assert_eq!(
        options.values()["independent_support_layer_height"],
        json!(false)
    );
}

#[test]
fn wrapping_detection_keeps_prime_tower_and_reports_support_height_key() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "enable_prime_tower": true,
        "independent_support_layer_height": true,
        "enable_wrapping_detection": true,
        "print_sequence": "by layer",
    }))
    .unwrap();

    let changed_keys = options.normalize_fdm_2(1, 1).unwrap();

    assert_eq!(changed_keys, ["independent_support_layer_height"]);
    assert_eq!(options.values()["enable_prime_tower"], json!(true));
    assert_eq!(
        options.values()["independent_support_layer_height"],
        json!(false)
    );
}

#[test]
fn smooth_timelapse_keeps_prime_tower_and_reports_support_height_key() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "enable_prime_tower": true,
        "independent_support_layer_height": true,
        "print_sequence": "by object",
        "timelapse_type": "1",
    }))
    .unwrap();

    let changed_keys = options.normalize_fdm_2(2, 1).unwrap();

    assert_eq!(changed_keys, ["independent_support_layer_height"]);
    assert_eq!(options.values()["enable_prime_tower"], json!(true));
    assert_eq!(
        options.values()["independent_support_layer_height"],
        json!(false)
    );
}

#[test]
fn false_prime_tower_creates_default_support_height_without_reporting_key() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "enable_prime_tower": false,
        "print_sequence": "by layer",
    }))
    .unwrap();

    let changed_keys = options.normalize_fdm_2(1, 2).unwrap();

    assert!(changed_keys.is_empty());
    assert_eq!(options.values()["enable_prime_tower"], json!(false));
    assert_eq!(
        options.values()["independent_support_layer_height"],
        json!(true)
    );
}

#[test]
fn already_disabled_support_height_is_not_reported_again() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "enable_prime_tower": true,
        "independent_support_layer_height": false,
        "print_sequence": "by layer",
    }))
    .unwrap();

    let changed_keys = options.normalize_fdm_2(1, 2).unwrap();

    assert!(changed_keys.is_empty());
    assert_eq!(options.values()["enable_prime_tower"], json!(true));
    assert_eq!(
        options.values()["independent_support_layer_height"],
        json!(false)
    );
}

#[test]
fn invalid_changed_key_branch_values_return_invalid_input() {
    for values in [
        json!({"enable_prime_tower": "true"}),
        json!({"enable_prime_tower": true, "independent_support_layer_height": "true"}),
        json!({"enable_prime_tower": true, "print_sequence": 1}),
        json!({"enable_prime_tower": true, "print_sequence": "sequential"}),
        json!({"enable_prime_tower": true, "timelapse_type": 1}),
        json!({"enable_prime_tower": true, "timelapse_type": "smooth"}),
        json!({"enable_prime_tower": true, "enable_wrapping_detection": "false"}),
    ] {
        let mut options: SliceOptions = serde_json::from_value(values.clone()).unwrap();

        let err = options
            .normalize_fdm_2(1, 2)
            .expect_err(&format!("expected invalid input for {values:?}"));

        assert!(matches!(err, SliceError::InvalidInput(_)));
    }
}

#[test]
fn normalize_fdm_existing_api_keeps_m189_behavior() {
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
