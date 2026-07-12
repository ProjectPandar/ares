use super::{options, update};
use crate::SliceError;
use serde_json::json;

#[test]
fn missing_variant_returns_minus_one_and_leaves_self_unchanged() {
    let mut target = options(json!({
        "printer_extruder_id": [7],
        "printer_extruder_variant": ["Direct Drive Standard"]
    }));
    let before = target.clone();
    let source = options(json!({"printer_extruder_id": ["not an int"]}));

    let result = update(&mut target, &source, &["printer_extruder_id"]).unwrap();

    assert_eq!(result, -1);
    assert_eq!(target, before);
}

#[test]
fn empty_variant_array_is_accepted_for_string_int_slice() {
    let mut target = options(json!({}));
    let source = options(json!({
        "printer_extruder_variant": [],
        "printer_extruder_id": [1, 2]
    }));

    let result = update(&mut target, &source, &["printer_extruder_id"]).unwrap();

    assert_eq!(result, 0);
    assert_eq!(target.values()["printer_extruder_id"], json!([1, 2]));
}

#[test]
fn invalid_present_variant_returns_invalid_input_and_leaves_self_unchanged() {
    let mut target = options(json!({"printer_extruder_id": [7]}));
    let before = target.clone();
    let source = options(json!({
        "printer_extruder_variant": "Direct Drive Standard",
        "printer_extruder_id": [1]
    }));

    let result = update(&mut target, &source, &["printer_extruder_id"]);

    assert!(matches!(result, Err(SliceError::InvalidInput(_))));
    assert_eq!(target, before);
}

#[test]
fn string_vector_keys_copy_and_overwrite_existing_values() {
    let mut target = options(json!({"printer_extruder_variant": ["old"]}));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden High Flow"]
    }));

    update(&mut target, &source, &["printer_extruder_variant"]).unwrap();

    assert_eq!(
        target.values()["printer_extruder_variant"],
        json!(["Direct Drive Standard", "Bowden High Flow"])
    );
}

#[test]
fn int_vector_keys_copy_and_overwrite_existing_values() {
    let mut target = options(json!({"printer_extruder_id": [7]}));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard"],
        "printer_extruder_id": [1, 2, 3]
    }));

    update(&mut target, &source, &["printer_extruder_id"]).unwrap();

    assert_eq!(target.values()["printer_extruder_id"], json!([1, 2, 3]));
}

#[test]
fn missing_source_values_leave_existing_values_unchanged() {
    let mut target = options(json!({"printer_extruder_id": [7]}));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard"]
    }));

    update(&mut target, &source, &["printer_extruder_id"]).unwrap();

    assert_eq!(target.values()["printer_extruder_id"], json!([7]));
}

#[test]
fn keys_are_processed_sorted_unique_and_unknown_keys_are_skipped() {
    let mut target = options(json!({"printer_extruder_variant": ["old"]}));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard"],
        "printer_extruder_id": [2147483648i64]
    }));

    let result = update(
        &mut target,
        &source,
        &[
            "printer_extruder_variant",
            "unknown_ares_option",
            "printer_extruder_variant",
            "printer_extruder_id",
        ],
    );

    assert!(matches!(result, Err(SliceError::InvalidInput(_))));
    assert_eq!(target.values()["printer_extruder_variant"], json!(["old"]));
}

#[test]
fn invalid_later_key_leaves_all_existing_values_unchanged() {
    let mut target = options(json!({
        "filament_extruder_variant": ["old filament"],
        "printer_extruder_id": [7],
        "printer_extruder_variant": ["old printer"]
    }));
    let before = target.clone();
    let source = options(json!({
        "filament_extruder_variant": ["Direct Drive High Flow"],
        "printer_extruder_id": [2147483648i64],
        "printer_extruder_variant": ["Direct Drive Standard"]
    }));

    let result = update(
        &mut target,
        &source,
        &[
            "filament_extruder_variant",
            "printer_extruder_id",
            "printer_extruder_variant",
        ],
    );

    assert!(matches!(result, Err(SliceError::InvalidInput(_))));
    assert_eq!(target, before);
}

#[test]
fn unsupported_kinds_are_skipped() {
    let mut target = options(json!({"enable_prime_tower": true}));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard"],
        "enable_prime_tower": "not a bool"
    }));

    update(&mut target, &source, &["enable_prime_tower"]).unwrap();

    assert_eq!(target.values()["enable_prime_tower"], json!(true));
}

#[test]
fn invalid_supported_source_values_return_invalid_input() {
    let source_cases = [
        json!({
            "printer_extruder_variant": ["Direct Drive Standard"],
            "print_extruder_variant": ["valid", 1]
        }),
        json!({
            "printer_extruder_variant": ["Direct Drive Standard"],
            "print_extruder_id": [1.25]
        }),
    ];
    let key_cases = ["print_extruder_variant", "print_extruder_id"];

    for (source_value, key) in source_cases.into_iter().zip(key_cases) {
        let mut target = options(json!({}));
        let source = options(source_value);

        let result = update(&mut target, &source, &[key]);

        assert!(matches!(result, Err(SliceError::InvalidInput(_))));
    }
}

#[test]
fn representative_source_option_names_copy() {
    let mut target = options(json!({}));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard"],
        "print_extruder_variant": ["Bowden Standard"],
        "filament_extruder_variant": ["Direct Drive High Flow"],
        "printer_extruder_id": [1],
        "print_extruder_id": [2],
        "filament_self_index": [3]
    }));

    update(
        &mut target,
        &source,
        &[
            "printer_extruder_variant",
            "print_extruder_variant",
            "filament_extruder_variant",
            "printer_extruder_id",
            "print_extruder_id",
            "filament_self_index",
        ],
    )
    .unwrap();

    assert_eq!(
        target.values()["printer_extruder_variant"],
        json!(["Direct Drive Standard"])
    );
    assert_eq!(
        target.values()["print_extruder_variant"],
        json!(["Bowden Standard"])
    );
    assert_eq!(
        target.values()["filament_extruder_variant"],
        json!(["Direct Drive High Flow"])
    );
    assert_eq!(target.values()["printer_extruder_id"], json!([1]));
    assert_eq!(target.values()["print_extruder_id"], json!([2]));
    assert_eq!(target.values()["filament_self_index"], json!([3]));
}
