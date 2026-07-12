use super::super::*;
use serde_json::json;

#[test]
fn expands_relative_correction_vector_to_missing_scalar_axes() {
    let options: SliceOptions = serde_json::from_value(json!({
        "relative_correction": [1.2, 1.3]
    }))
    .unwrap();

    assert_eq!(options.values()["relative_correction_x"], json!(1.2));
    assert_eq!(options.values()["relative_correction_y"], json!(1.2));
    assert_eq!(options.values()["relative_correction_z"], json!(1.3));
}

#[test]
fn expands_material_correction_vector_with_upstream_indexing() {
    let options: SliceOptions = serde_json::from_value(json!({
        "material_correction": [2.1, 2.2, 2.3]
    }))
    .unwrap();

    assert_eq!(options.values()["material_correction_x"], json!(2.1));
    assert_eq!(options.values()["material_correction_y"], json!(2.1));
    assert_eq!(options.values()["material_correction_z"], json!(2.2));
}

#[test]
fn preserves_existing_scalar_correction_values() {
    let options: SliceOptions = serde_json::from_value(json!({
        "relative_correction": [1.2, 1.3],
        "relative_correction_x": 9.0,
        "relative_correction_z": 8.0
    }))
    .unwrap();

    assert_eq!(options.values()["relative_correction_x"], json!(9.0));
    assert_eq!(options.values()["relative_correction_y"], json!(1.2));
    assert_eq!(options.values()["relative_correction_z"], json!(8.0));
}

#[test]
fn absent_correction_vectors_do_not_create_scalar_axes() {
    let options: SliceOptions = serde_json::from_value(json!({
        "future_orca_key": true
    }))
    .unwrap();

    assert!(!options.values().contains_key("relative_correction_x"));
    assert!(!options.values().contains_key("relative_correction_y"));
    assert!(!options.values().contains_key("relative_correction_z"));
    assert!(!options.values().contains_key("material_correction_x"));
    assert!(!options.values().contains_key("material_correction_y"));
    assert!(!options.values().contains_key("material_correction_z"));
}

#[test]
fn all_existing_scalar_axes_do_not_parse_invalid_vector() {
    let options: SliceOptions = serde_json::from_value(json!({
        "material_correction": ["not numeric"],
        "material_correction_x": 4.0,
        "material_correction_y": 5.0,
        "material_correction_z": 6.0
    }))
    .unwrap();

    assert_eq!(options.values()["material_correction_x"], json!(4.0));
    assert_eq!(options.values()["material_correction_y"], json!(5.0));
    assert_eq!(options.values()["material_correction_z"], json!(6.0));
}

#[test]
fn accepts_numeric_string_vectors_and_scalar_when_only_index_zero_is_needed() {
    let string_vector: SliceOptions = serde_json::from_value(json!({
        "relative_correction": "1.4;1.5",
        "relative_correction_z": 9.0
    }))
    .unwrap();
    assert_eq!(string_vector.values()["relative_correction_x"], json!(1.4));
    assert_eq!(string_vector.values()["relative_correction_y"], json!(1.4));
    assert_eq!(string_vector.values()["relative_correction_z"], json!(9.0));

    let comma_vector: SliceOptions = serde_json::from_value(json!({
        "material_correction": "2.4,2.5"
    }))
    .unwrap();
    assert_eq!(comma_vector.values()["material_correction_x"], json!(2.4));
    assert_eq!(comma_vector.values()["material_correction_y"], json!(2.4));
    assert_eq!(comma_vector.values()["material_correction_z"], json!(2.5));

    let scalar: SliceOptions = serde_json::from_value(json!({
        "relative_correction": 3.4,
        "relative_correction_z": 7.0
    }))
    .unwrap();
    assert_eq!(scalar.values()["relative_correction_x"], json!(3.4));
    assert_eq!(scalar.values()["relative_correction_y"], json!(3.4));
    assert_eq!(scalar.values()["relative_correction_z"], json!(7.0));
}

#[test]
fn ignores_invalid_unneeded_vector_tail_when_only_index_zero_is_needed() {
    let options: SliceOptions = serde_json::from_value(json!({
        "relative_correction": [1.7, "not numeric"],
        "relative_correction_z": 9.0
    }))
    .unwrap();

    assert_eq!(options.values()["relative_correction_x"], json!(1.7));
    assert_eq!(options.values()["relative_correction_y"], json!(1.7));
    assert_eq!(options.values()["relative_correction_z"], json!(9.0));
}

#[test]
fn rejects_invalid_non_finite_or_too_short_needed_correction_vectors() {
    for (key, value, index) in [
        ("relative_correction", json!(["bad", 1.0]), 0),
        ("relative_correction", json!([1.0]), 1),
        ("relative_correction", json!(3.0), 1),
        ("relative_correction", json!("1.0;inf"), 1),
        ("material_correction", json!([]), 0),
        ("material_correction", json!({"value": 1.0}), 0),
    ] {
        let error = serde_json::from_value::<SliceOptions>(json!({ key: value })).unwrap_err();
        let message = error.to_string();
        assert!(message.contains(key), "{message}");
        assert!(message.contains(&format!("index {index}")), "{message}");
        assert!(message.contains("finite number"), "{message}");
    }
}

#[test]
fn legacy_sla_normalization_preserves_existing_composite_behavior() {
    let options: SliceOptions = serde_json::from_value(json!({
        "relative_correction": [1.2, 1.3],
        "thumbnail_size": "256x256",
        "wiping_volumes_matrix": [0, 140, 141, 0]
    }))
    .unwrap();

    assert_eq!(options.values()["relative_correction_x"], json!(1.2));
    assert_eq!(options.values()["relative_correction_y"], json!(1.2));
    assert_eq!(options.values()["relative_correction_z"], json!(1.3));
    assert_eq!(options.values()["thumbnails"], json!("256x256/PNG"));
    assert_eq!(
        options.values()["wiping_volumes_use_custom_matrix"],
        json!(true)
    );
}
