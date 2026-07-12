use crate::{MultiToMultiUpdate, SliceError, SliceOptions};

mod bools;
mod float_or_percent;
mod floats;
use serde_json::{Value, json};

pub(super) fn options(value: Value) -> SliceOptions {
    serde_json::from_value(value).unwrap()
}

pub(super) fn update(
    target: &mut SliceOptions,
    source: &SliceOptions,
    keys: &[&str],
) -> Result<isize, SliceError> {
    target.update_values_from_multi_to_multi_string_int_float_percent_bool_keys(
        MultiToMultiUpdate {
            new_config: source,
            key_set: keys,
            id_name: "printer_extruder_id",
            variant_name: "printer_extruder_variant",
            new_extruder_variants: &["Direct Drive Standard", "Bowden Standard"],
        },
    )
}

#[test]
fn missing_required_variant_or_id_returns_minus_one_and_leaves_self_unchanged() {
    let cases = [
        (
            json!({"printer_extruder_id": [7]}),
            json!({
                "printer_extruder_variant": ["Direct Drive Standard"],
                "printer_extruder_id": [1],
                "print_extruder_id": ["not an int"]
            }),
        ),
        (
            json!({"printer_extruder_variant": ["old"]}),
            json!({
                "printer_extruder_id": [1],
                "print_extruder_id": ["not an int"]
            }),
        ),
        (
            json!({"printer_extruder_variant": ["old"]}),
            json!({
                "printer_extruder_variant": ["Direct Drive Standard"],
                "print_extruder_id": ["not an int"]
            }),
        ),
    ];

    for (target_value, source_value) in cases {
        let mut target = options(target_value);
        let before = target.clone();
        let source = options(source_value);

        let result = update(&mut target, &source, &["print_extruder_id"]).unwrap();

        assert_eq!(result, -1);
        assert_eq!(target, before);
    }
}

#[test]
fn invalid_required_variant_or_id_values_return_invalid_input_without_mutation() {
    let cases = [
        (
            json!({"printer_extruder_variant": "old"}),
            json!({
                "printer_extruder_variant": ["Direct Drive Standard"],
                "printer_extruder_id": [1]
            }),
        ),
        (
            json!({"printer_extruder_variant": ["old"]}),
            json!({
                "printer_extruder_variant": "Direct Drive Standard",
                "printer_extruder_id": [1]
            }),
        ),
        (
            json!({"printer_extruder_variant": ["old"]}),
            json!({
                "printer_extruder_variant": ["Direct Drive Standard"],
                "printer_extruder_id": [1.5]
            }),
        ),
    ];

    for (target_value, source_value) in cases {
        let mut target = options(target_value);
        let before = target.clone();
        let source = options(source_value);

        let result = update(&mut target, &source, &["print_extruder_id"]);

        assert!(matches!(result, Err(SliceError::InvalidInput(_))));
        assert_eq!(target, before);
    }
}

#[test]
fn string_vector_keys_copy_and_overwrite_existing_values() {
    let mut target = options(json!({
        "printer_extruder_variant": ["old"],
        "print_extruder_variant": ["old print"]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "printer_extruder_id": [1, 2],
        "print_extruder_variant": ["quality", "draft"]
    }));

    update(&mut target, &source, &["print_extruder_variant"]).unwrap();

    assert_eq!(
        target.values()["print_extruder_variant"],
        json!(["quality", "draft"])
    );
}

#[test]
fn int_vector_keys_copy_and_overwrite_existing_values() {
    let mut target = options(json!({
        "printer_extruder_variant": ["old"],
        "print_extruder_id": [7]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard"],
        "printer_extruder_id": [1],
        "print_extruder_id": [1, 2, 3]
    }));

    update(&mut target, &source, &["print_extruder_id"]).unwrap();

    assert_eq!(target.values()["print_extruder_id"], json!([1, 2, 3]));
}

#[test]
fn missing_source_values_leave_existing_values_unchanged() {
    let mut target = options(json!({
        "printer_extruder_variant": ["old"],
        "print_extruder_id": [7]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard"],
        "printer_extruder_id": [1]
    }));

    update(&mut target, &source, &["print_extruder_id"]).unwrap();

    assert_eq!(target.values()["print_extruder_id"], json!([7]));
}

#[test]
fn keys_are_processed_sorted_unique_and_unknown_keys_are_skipped() {
    let mut target = options(json!({
        "printer_extruder_variant": ["old"],
        "print_extruder_variant": ["old print"]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard"],
        "printer_extruder_id": [1],
        "print_extruder_variant": ["new print"],
        "print_extruder_id": [2147483648i64]
    }));

    let result = update(
        &mut target,
        &source,
        &[
            "print_extruder_variant",
            "unknown_ares_option",
            "print_extruder_variant",
            "print_extruder_id",
        ],
    );

    assert!(matches!(result, Err(SliceError::InvalidInput(_))));
    assert_eq!(
        target.values()["print_extruder_variant"],
        json!(["old print"])
    );
}

#[test]
fn unsupported_kinds_are_skipped() {
    let mut target = options(json!({
        "printer_extruder_variant": ["old"],
        "enable_prime_tower": true
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard"],
        "printer_extruder_id": [1],
        "enable_prime_tower": "not a bool"
    }));

    update(&mut target, &source, &["enable_prime_tower"]).unwrap();

    assert_eq!(target.values()["enable_prime_tower"], json!(true));
}

#[test]
fn invalid_supported_source_values_return_invalid_input_without_mutation() {
    let source_cases = [
        json!({
            "printer_extruder_variant": ["Direct Drive Standard"],
            "printer_extruder_id": [1],
            "print_extruder_variant": ["valid", 1]
        }),
        json!({
            "printer_extruder_variant": ["Direct Drive Standard"],
            "printer_extruder_id": [1],
            "print_extruder_id": [1.25]
        }),
    ];
    let key_cases = ["print_extruder_variant", "print_extruder_id"];

    for (source_value, key) in source_cases.into_iter().zip(key_cases) {
        let mut target = options(json!({"printer_extruder_variant": ["old"]}));
        let before = target.clone();
        let source = options(source_value);

        let result = update(&mut target, &source, &[key]);

        assert!(matches!(result, Err(SliceError::InvalidInput(_))));
        assert_eq!(target, before);
    }
}

#[test]
fn representative_source_option_names_copy() {
    let mut target = options(json!({
        "printer_extruder_variant": ["old printer"],
        "print_extruder_variant": ["old print"],
        "filament_extruder_variant": ["old filament"],
        "printer_extruder_id": [9],
        "print_extruder_id": [8],
        "filament_self_index": [7]
    }));
    let source = options(json!({
        "printer_extruder_variant": ["Direct Drive Standard", "Bowden Standard"],
        "printer_extruder_id": [1, 2],
        "print_extruder_variant": ["quality", "draft"],
        "filament_extruder_variant": ["support", "model"],
        "print_extruder_id": [3, 4],
        "filament_self_index": [5, 6]
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
        json!(["Direct Drive Standard", "Bowden Standard"])
    );
    assert_eq!(
        target.values()["print_extruder_variant"],
        json!(["quality", "draft"])
    );
    assert_eq!(
        target.values()["filament_extruder_variant"],
        json!(["support", "model"])
    );
    assert_eq!(target.values()["printer_extruder_id"], json!([1, 2]));
    assert_eq!(target.values()["print_extruder_id"], json!([3, 4]));
    assert_eq!(target.values()["filament_self_index"], json!([5, 6]));
}
