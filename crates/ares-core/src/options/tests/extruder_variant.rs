use super::super::*;
use crate::SliceError;
use serde_json::json;

#[test]
fn missing_variant_list_creates_default_entries_and_generated_arrays() {
    let mut options: SliceOptions = serde_json::from_value(json!({})).unwrap();

    options.extend_extruder_variant(3).unwrap();

    assert_eq!(
        options.values()["extruder_variant_list"],
        json!([
            "Direct Drive Standard",
            "Direct Drive Standard",
            "Direct Drive Standard"
        ])
    );
    assert_eq!(options.values()["printer_extruder_id"], json!([1, 2, 3]));
    assert_eq!(
        options.values()["printer_extruder_variant"],
        json!([
            "Direct Drive Standard",
            "Direct Drive Standard",
            "Direct Drive Standard"
        ])
    );
}

#[test]
fn short_variant_list_extends_with_first_entry() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "extruder_variant_list": ["High Flow"]
    }))
    .unwrap();

    options.extend_extruder_variant(3).unwrap();

    assert_eq!(
        options.values()["extruder_variant_list"],
        json!(["High Flow", "High Flow", "High Flow"])
    );
    assert_eq!(options.values()["printer_extruder_id"], json!([1, 2, 3]));
    assert_eq!(
        options.values()["printer_extruder_variant"],
        json!(["High Flow", "High Flow", "High Flow"])
    );
}

#[test]
fn long_variant_list_truncates_to_extruder_count() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "extruder_variant_list": ["A", "B", "C"]
    }))
    .unwrap();

    options.extend_extruder_variant(2).unwrap();

    assert_eq!(options.values()["extruder_variant_list"], json!(["A", "B"]));
    assert_eq!(options.values()["printer_extruder_id"], json!([1, 2]));
    assert_eq!(
        options.values()["printer_extruder_variant"],
        json!(["A", "B"])
    );
}

#[test]
fn comma_separated_variants_generate_flattened_ids_and_variants() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "extruder_variant_list": ["Standard,High Flow", "Direct Drive, Hardened"]
    }))
    .unwrap();

    options.extend_extruder_variant(2).unwrap();

    assert_eq!(options.values()["printer_extruder_id"], json!([1, 1, 2, 2]));
    assert_eq!(
        options.values()["printer_extruder_variant"],
        json!(["Standard", "High Flow", "Direct Drive", " Hardened"])
    );
}

#[test]
fn boost_split_edge_cases_preserve_boundary_empty_tokens() {
    for (variant, expected) in [
        ("", json!([""])),
        (",", json!(["", ""])),
        (",A", json!(["", "A"])),
        ("A,", json!(["A", ""])),
        ("A,,B", json!(["A", "B"])),
    ] {
        let mut options: SliceOptions = serde_json::from_value(json!({
            "extruder_variant_list": [variant]
        }))
        .unwrap();

        options.extend_extruder_variant(1).unwrap();

        assert_eq!(options.values()["printer_extruder_variant"], expected);
        assert_eq!(
            options.values()["printer_extruder_id"],
            json!(vec![1; expected.as_array().unwrap().len()])
        );
    }
}

#[test]
fn existing_generated_arrays_are_replaced() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "extruder_variant_list": ["A,B"],
        "printer_extruder_id": [99],
        "printer_extruder_variant": ["stale"]
    }))
    .unwrap();

    options.extend_extruder_variant(1).unwrap();

    assert_eq!(options.values()["printer_extruder_id"], json!([1, 1]));
    assert_eq!(
        options.values()["printer_extruder_variant"],
        json!(["A", "B"])
    );
}

#[test]
fn zero_extruders_produces_empty_arrays() {
    let mut options: SliceOptions = serde_json::from_value(json!({
        "extruder_variant_list": ["A"],
        "printer_extruder_id": [1],
        "printer_extruder_variant": ["A"]
    }))
    .unwrap();

    options.extend_extruder_variant(0).unwrap();

    assert_eq!(options.values()["extruder_variant_list"], json!([]));
    assert_eq!(options.values()["printer_extruder_id"], json!([]));
    assert_eq!(options.values()["printer_extruder_variant"], json!([]));
}

#[test]
fn invalid_variant_list_values_return_invalid_input() {
    for invalid_value in [json!("A"), json!(["A", 2]), json!([])] {
        let mut options: SliceOptions = serde_json::from_value(json!({
            "extruder_variant_list": invalid_value,
            "printer_extruder_id": [7],
            "printer_extruder_variant": ["existing"],
            "unrelated": "preserved"
        }))
        .unwrap();

        let error = options.extend_extruder_variant(1).unwrap_err();

        assert!(matches!(error, SliceError::InvalidInput(_)));
        assert_eq!(options.values()["printer_extruder_id"], json!([7]));
        assert_eq!(
            options.values()["printer_extruder_variant"],
            json!(["existing"])
        );
        assert_eq!(options.values()["unrelated"], json!("preserved"));
    }
}
